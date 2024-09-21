const ZANA_BOOK_EVENT = 'ZanaBookEvent';
const ZANA_NON_BOOK_EVENT = 'ZanaNonBookEvent';

const dataProviders = {
    googlebooks: {
        label: 'Google Books',
        poweredByImage: {
            url: browser.runtime.getURL('images/powered_by_googlebooks.png'),
            width: 60,
        },
    },
    openlibrary: {
        label: 'OpenLibrary',
        poweredByImage: {
            url: browser.runtime.getURL('images/powered_by_openlibrary.svg'),
            width: 100,
        },
    }
}

// Add an event listener to the custom event dispatched by the content script.
window.addEventListener(ZANA_BOOK_EVENT, handle);
window.addEventListener(ZANA_NON_BOOK_EVENT, handleOtherPages);

let zanaContext = {
    currentISBN: '',
    cache: []
}

function zanaInit() {
    let bodyElement = document.querySelector('body');

    let zanaContainer = document.querySelector('.zana-main-container');
    if (zanaContainer) {
        return;
    }

    zanaContainer = document.createElement('div');
    zanaContainer.className = 'zana-main-container';

    let zanaButton = document.createElement('div');
    zanaButton.className = 'zana-main-button';

    let panelContainer = document.createElement('div');
    panelContainer.className = 'zana-main-panel-container';

    let panel = document.createElement('div');
    panel.className = 'zana-main-panel';
    panel.addEventListener('click', function() {
        this.style.visibility = 'hidden';
        this.parentElement.style.visibility = 'hidden';
    })

    zanaButton.addEventListener('mouseover', zanaHoverHandle);

    panelContainer.append(panel);
    zanaContainer.append(zanaButton);
    bodyElement.append(panelContainer);
    bodyElement.append(zanaContainer);
}

function zanaHoverHandle() {
    let panelContainer =  document.querySelector('.zana-main-panel-container');
    let panel = document.querySelector('.zana-main-panel');
    panelContainer.style.visibility = 'visible';
    panel.style.visibility = 'visible';

    // Show loading screen
    zanaLoading();

    // Get data from Zana service
    const responsePromises = retrieveBookData(zanaContext.currentISBN);
    Promise.all(responsePromises)
        .then(responses => {
            // Remove loading screen
            panel.replaceChildren();
            // Display the needed HTML elements.
            displayBookMetadata(responses);
        });
}

// Send requests to Zana service
function retrieveBookData(isbn) {
    const queries = [
        {
            url: apiUrl + '?type=googlebooks&isbn=' + isbn,
            type: 'googlebooks',
        },
        {
            url: apiUrl + '?type=openlibrary&isbn=' + isbn,
            type: 'openlibrary',
        }
    ];

    return queries.map(query => fetch(query.url)
        .then(response => {
            return response.json().then(responseBody => {
                return {
                    isbn: isbn,
                    body: responseBody,
                    status: response.status,
                    type: query.type,
                };
            });
        })
        .catch(error => {
            console.error('Zana: Error while retrieving book data for isbn: ' + isbn);
            console.error(error);
            return {
                isbn: isbn,
                body: error,
                status: 500,
                type: query.type,
            };
        }));
}

function handle(event) {
    let zanaContainer = document.querySelector('.zana-main-container');
    zanaContainer.style.visibility = 'visible';

    let zanaPanel = document.querySelector('.zana-main-panel');
    zanaPanel.style.visibility = 'hidden';

    zanaContext.currentISBN = event.detail.isbn;

}

function handleOtherPages() {
    let zanaContainer = document.querySelector('.zana-main-container');
    zanaContainer.style.visibility = 'hidden';

    let zanaPanel = document.querySelector('.zana-main-panel');
    zanaPanel.style.visibility = 'hidden';
}

function zanaLoading() {
    let panel = document.querySelector('.zana-main-panel');
    panel.replaceChildren();

    let loadingContainer = document.createElement('div');
    loadingContainer.className = 'zana-loading-container';

    let loadingImage = document.createElement('img');
    loadingImage.src = browser.runtime.getURL('images/main/loading.gif');
    loadingImage.className = 'zana-loading-image';
    loadingImage.alt = 'Loading...';

    loadingContainer.appendChild(loadingImage);
    panel.append(loadingContainer);
}

function displayBookMetadata(responses) {
    let panel = document.querySelector('.zana-main-panel');

    let dataContainer = document.createElement('div');
    dataContainer.className = 'row zana-container';

    // Keep only the successful responses
    let validResponses = responses.filter(response => response.status === 200);

    if (validResponses.length === 0) {
        let statusCodes = responses.map(response => response.status);
        let message;
        if (statusCodes.includes(404)) {
            message = 'did not find any data for this book.';
        } else if (statusCodes.includes(429)) {
            message = 'has reached the maximum number of requests. Please try again later.';
        } else {
            message = 'is having trouble retrieving book data at the moment. Please try again later.';
        }

        let missingDataElement = createMissingDataElement(message);
        dataContainer.appendChild(missingDataElement);
        panel.append(dataContainer);
        console.log('Zana response: ', responses);
        return;
    }

    addRatings(validResponses, dataContainer);
    addBookMetadata(validResponses, dataContainer);
    panel.append(dataContainer);
}

function createMissingDataElement(message) {
    let missingDataContainer = document.createElement('div');
    missingDataContainer.className = 'zana-missing-data-container';
    missingDataContainer.textContent = message;

    let missingDataAppName = document.createElement('span');
    missingDataAppName.className = 'zana-missing-data-app-name';
    missingDataAppName.textContent = 'Zana ';

    missingDataContainer.prepend(missingDataAppName);
    return missingDataContainer;
}

function addBookMetadata(responses, dataContainer) {
    let numberOfPages;
    let description;
    // If page count is available from the first provider, then use it, otherwise continue to the next provider
    // until an available page count is found. If no page count is found, then it is not displayed.
    // Same logic is applied for the description.
    for (let response of responses) {
        const data = response.body.data;
        if (data.page_count && !numberOfPages) {
            numberOfPages = data.page_count;
        }
        if (data.description && !description) {
            description = data.description;
        }
    }

    if (numberOfPages) {
        const numberOfPagesElement = createBookMetadata(numberOfPages);
        dataContainer.appendChild(numberOfPagesElement);
    }
    if (description) {
        const descriptionElement = createDescription(description);
        dataContainer.appendChild(descriptionElement);
    }
}

function addRatings(responses, dataContainer) {
    let ratingsContainer = document.createElement('div');
    ratingsContainer.className = 'zana-ratings-container';

    // Available ratings are displayed in the order they are received from the providers.
    // If no ratings are available, then the ratings container is not displayed.
    for (let response of responses) {
        let rating = response.body.rating;
        if (rating) {
            let ratingsElement = createRatings(response.type, rating, response.body.data.provider_link);
            ratingsContainer.appendChild(ratingsElement);
        }
    }
    dataContainer.appendChild(ratingsContainer);
}

function createBookMetadata(numberOfPages) {
    let metadataContainer = document.createElement('div');
    metadataContainer.className = 'zana-metadata-container';
    metadataContainer.textContent = `
            Number of pages: ${numberOfPages}
        `;
    return metadataContainer;
}

function createDescription(description) {
    let descriptionElement = document.createElement('div');
    descriptionElement.className = 'zana-description';
    descriptionElement.textContent = description;
    return descriptionElement;
}

function createRatings(type, rating, provider_link) {
    let ratingsElement = document.createElement('div');
    ratingsElement.className = 'zana-rating';
    // Rating type
    let ratingTypeElement = document.createElement('span');
    ratingTypeElement.className = 'zana-rating-type';

    // Rating stars
    let stars = createRatingStars(rating.average_rating);
    stars.className = 'zana-stars';
    ratingsElement.appendChild(stars);

    // Rating data
    let ratingDataElement = document.createElement('span');
    ratingDataElement.className = 'zana-rating-data';
    ratingDataElement.textContent = `
            ${rating.ratings_count} Reviews
            (${rating.average_rating.toFixed(2)} Average)
        `;
    ratingsElement.appendChild(ratingDataElement);

    let ratingPoweredByImageElement = document.createElement('img');
    ratingPoweredByImageElement.src = dataProviders[type].poweredByImage.url;
    ratingPoweredByImageElement.className = 'zana-rating-type-image';
    ratingPoweredByImageElement.width = dataProviders[type].poweredByImage.width;
    ratingsElement.appendChild(ratingPoweredByImageElement);

    let ratingExternalLinkElement = document.createElement('a');
    ratingExternalLinkElement.href = provider_link;
    ratingExternalLinkElement.target = '_blank';
    ratingExternalLinkElement.className = 'zana-rating-external-link';
    ratingsElement.appendChild(ratingExternalLinkElement);

    let ratingExternalLinkImageElement = document.createElement('img');
    ratingExternalLinkImageElement.src = browser.runtime.getURL('images/main/external_link.svg');
    ratingExternalLinkImageElement.className = 'zana-rating-external-link-image';
    ratingExternalLinkElement.appendChild(ratingExternalLinkImageElement);

    return ratingsElement;
}

function createRatingStars(averageRating) {
    const rating = Math.round(averageRating);

    let starsElement = document.createElement('div');
    starsElement.className = 'b-rating flex-grow-1';

    let filledStarPath = 'M3.612 15.443c-.386.198-.824-.149-.746-.592l.83-4.73L.173 6.765c-.329-.314-.158-.888.283-.95l4.898-.696L7.538.792c.197-.39.73-.39.927 0l2.184 4.327 4.898.696c.441.062.612.636.283.95l-3.523 3.356.83 4.73c.078.443-.36.79-.746.592L8 13.187l-4.389 2.256z'
    let emptyStarPath = 'M2.866 14.85c-.078.444.36.791.746.593l4.39-2.256 4.389 2.256c.386.198.824-.149.746-.592l-.83-4.73 3.523-3.356c.329-.314.158-.888-.283-.95l-4.898-.696L8.465.792a.513.513 0 0 0-.927 0L5.354 5.12l-4.898.696c-.441.062-.612.636-.283.95l3.523 3.356-.83 4.73zm4.905-2.767l-3.686 1.894.694-3.957a.565.565 0 0 0-.163-.505L1.71 6.745l4.052-.576a.525.525 0 0 0 .393-.288l1.847-3.658 1.846 3.658a.525.525 0 0 0 .393.288l4.052.575-2.906 2.77a.564.564 0 0 0-.163.506l.694 3.957-3.686-1.894a.503.503 0 0 0-.461 0z';

    for (let i = 1; i <= 5; i++) {
        const starPath = i <= rating ? filledStarPath : emptyStarPath;

        // Create the star element
        const starElement = document.createElement('span');
        starElement.classList.add('b-rating-star', 'zana-star');

        const starIcon = document.createElement('span');
        starIcon.className = 'b-rating-icon';

        const svgElement = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
        svgElement.setAttribute('viewBox', '0 0 16 16');
        svgElement.setAttribute('width', '1em');
        svgElement.setAttribute('height', '1em');
        svgElement.setAttribute('focusable', 'false');
        svgElement.setAttribute('role', 'img');
        svgElement.setAttribute('aria-label', 'star');
        svgElement.setAttribute('xmlns', 'http://www.w3.org/2000/svg');
        svgElement.setAttribute('fill', 'currentColor');
        svgElement.classList.add('bi-star', 'b-icon', 'bi');

        const gElement = document.createElementNS('http://www.w3.org/2000/svg', 'g');

        const pathElement = document.createElementNS('http://www.w3.org/2000/svg', 'path');
        pathElement.setAttribute('d', starPath);

        // Add the path element to the group element, and the group element to the SVG element
        gElement.appendChild(pathElement);
        svgElement.appendChild(gElement);

        // Add the SVG element to the icon element, and the icon element to the star element
        starIcon.appendChild(svgElement);
        starElement.appendChild(starIcon);

        // Add the star element to the array
        starsElement.append(starElement);
    }
    return starsElement;
}
