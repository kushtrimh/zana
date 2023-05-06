const fs = require('fs');
const {TextEncoder, TextDecoder} = require('util');
global.TextEncoder = TextEncoder;
global.TextDecoder = TextDecoder;

const {JSDOM} = require('jsdom');
const {dukagjiniBooks} = require('../../addon/modules/dukagjinibooks');

const html = fs.readFileSync('./tests/dukagjinibooks/dom.html', 'utf8').toString();

function createDefaultSuccessResponses() {
    return [
        {
            'isbn': '9780261102385',
            'body': {
                'data': {
                    'page_count': 1567,
                    'description': 'Description 1',
                    'provider_link': 'http://localhost/link/1'
                },
                'rating': {
                    'average_rating': 4.5,
                    'ratings_count': 147
                }
            },
            'status': 200,
            'type': 'googlebooks'
        },
        {
            'isbn': '9780261102385',
            'body': {
                'data': {
                    'page_count': 123,
                    'description': 'Description 2',
                    'provider_link': 'http://localhost/link/2'
                },
                'rating': {
                    'average_rating': 4.515625,
                    'ratings_count': 64
                }
            },
            'status': 200,
            'type': 'openlibrary'
        }
    ];
}

function initializeAndSendSuccessfulEvent(responses) {
    dukagjiniBooks.init();
    const event = new CustomEvent(dukagjiniBooks.eventName, {
        detail: {
            responses: responses || createDefaultSuccessResponses(),
        }
    });
    dispatchEvent(event);
}

function displayErrorMessageForStatusCodes(statusCodes, expectedMessage) {
    let responses = createDefaultSuccessResponses();
    responses[0].status = statusCodes[0];
    responses[1].status = statusCodes[1];
    initializeAndSendSuccessfulEvent(responses);

    const message = document.querySelector('.dukagjinibooks-missing-data-container');
    expect(message.textContent).toBe(expectedMessage);
}

describe('dukagjini books', () => {

    beforeEach(() => {
        global.browser = {
            runtime: {
                getURL: jest.fn(),
            }
        }

        const dom = new JSDOM(html, {});
        global.document.body.innerHTML = dom.window.document.body.innerHTML;
        global.window = dom.window;
    });

    it('should return isbn', () => {
        expect(dukagjiniBooks.retrieveIsbn()).toBe('9780261102385');
    })

    it('should display loading screen', () => {
        const loadingBeforeInit = document.querySelector('.dukagjinibooks-loading-container');
        dukagjiniBooks.loading();
        const loadingAfterInit = document.querySelector('.dukagjinibooks-loading-container');

        expect(loadingBeforeInit).toBeNull();
        expect(loadingAfterInit).not.toBeNull();
    })

    it('should remove loading screen when event is triggered', () => {
        dukagjiniBooks.loading();
        initializeAndSendSuccessfulEvent();
        const loading = document.querySelector('.dukagjinibooks-loading-container');
        expect(loading).toBeNull();
    })

    it('should display book description', () => {
        initializeAndSendSuccessfulEvent();

        const description = document.querySelector('.dukagjinibooks-description');
        expect(description.textContent).toBe('Description 1');
    })

    it('should display book page count', () => {
        initializeAndSendSuccessfulEvent();

        const pageCount = document.querySelector('.dukagjinibooks-metadata-container');
        expect(pageCount.textContent.trim()).toBe('Number of pages: 1567');
    })

    it('should display book ratings count', () => {
        initializeAndSendSuccessfulEvent();

        const ratingsCount = document.querySelectorAll('.dukagjinibooks-rating-data');
        expect(ratingsCount.length).toBe(2);

        const firstRating = ratingsCount[0];
        const firstRatingText = firstRating.textContent;
        expect(firstRatingText).toContain('147 Reviews');
        expect(firstRatingText).toContain('4.50 Average');

        const secondRating = ratingsCount[1];
        const secondRatingText = secondRating.textContent;
        expect(secondRatingText).toContain('64 Reviews');
        expect(secondRatingText).toContain('4.52 Average');
    })

    it('should display book provider link', () => {
        initializeAndSendSuccessfulEvent();

        const providerLinks = document.querySelectorAll('.dukagjinibooks-rating-external-link');

        expect(providerLinks.length).toBe(2);
        expect(providerLinks[0].href).toBe('http://localhost/link/1');
        expect(providerLinks[1].href).toBe('http://localhost/link/2');
    })

    it('should not display number of pages when its missing', () => {
        let responses = createDefaultSuccessResponses();
        responses[0].body.data.page_count = null;
        responses[1].body.data.page_count = 0;
        initializeAndSendSuccessfulEvent(responses);

        const pageCount = document.querySelector('.dukagjinibooks-metadata-container');
        expect(pageCount).toBeNull();
    })

    it('should not display description when its missing', () => {
        let responses = createDefaultSuccessResponses();
        responses[0].body.data.description = null;
        responses[1].body.data.description = '';
        initializeAndSendSuccessfulEvent(responses);

        const description = document.querySelector('.dukagjinibooks-description');
        expect(description).toBeNull();
    })

    it('should not display ratings when they are missing', () => {
        let responses = createDefaultSuccessResponses();
        responses[0].body.rating = null;
        responses[1].body.rating = null;
        initializeAndSendSuccessfulEvent(responses);

        const ratingsCount = document.querySelectorAll('.dukagjinibooks-rating-data');
        expect(ratingsCount.length).toBe(0);
    })

    it('should display message that book data is not found when 404 is present together with 500 status code', () => {
        displayErrorMessageForStatusCodes([404, 500], 'Zana did not find any data for this book.');
    })

    it('should display message that book data is not found when 404 is present together with 429 status code', () => {
        displayErrorMessageForStatusCodes([404, 429], 'Zana did not find any data for this book.');
    })

    it('should display message that the rate limit has been reached if 429 is included in status codes', () => {
        displayErrorMessageForStatusCodes([429, 500], 'Zana has reached the maximum number of requests. Please try again later.');
    })

    it('should display message that book the extension is having trouble retrieving data for other status codes', () => {
        displayErrorMessageForStatusCodes([500, 503], 'Zana is having trouble retrieving book data at the moment. Please try again later.');
    })

    it('should display number of pages from second response if first response has them as 0', () => {
        let responses = createDefaultSuccessResponses();
        responses[0].body.data.page_count = 0;
        responses[1].body.data.page_count = 123;
        initializeAndSendSuccessfulEvent(responses);

        const pageCount = document.querySelector('.dukagjinibooks-metadata-container');
        expect(pageCount.textContent.trim()).toBe('Number of pages: 123');
    })

    it('should display description from second response if first response has it as empty string', () => {
        let responses = createDefaultSuccessResponses();
        responses[0].body.data.description = '';
        responses[1].body.data.description = 'Description 2';
        initializeAndSendSuccessfulEvent(responses);

        const description = document.querySelector('.dukagjinibooks-description');
        expect(description.textContent).toBe('Description 2');
    })
})
