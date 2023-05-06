/*
 This module is used to handle any functionality related with dukagjinibooks.com.
 */

// Host configuration
let dukagjiniBooks = {
    eventName: 'dukagjiniEvent',
    queryBookData: true,
};

// Required function used to retrieve the ISBN, which is used to query the book data.
dukagjiniBooks.retrieveIsbn = function () {
    let bookItems = document.querySelectorAll('.book-item-specifics h5 span');
    let isbnItem;
    for (let bookItem of bookItems) {
        if (bookItem.textContent.includes('ISBN')) {
            isbnItem = bookItem;
            break;
        }
    }
    if (isbnItem) {
        return isbnItem.parentElement.textContent.split(':')[1].trim();
    }
}

// Required function used to display a loading indicator.
dukagjiniBooks.loading = function () {
    let bookDetailsNode = document.querySelector('#book-details .container');

    let loadingContainer = document.createElement('div');
    loadingContainer.className = 'dukagjinibooks-loading-container';

    let loadingImage = document.createElement('img');
    loadingImage.src = browser.runtime.getURL('images/dukagjini/loading.gif');
    loadingImage.className = 'dukagjinibooks-loading-image';
    loadingImage.alt = 'Loading...';

    loadingContainer.appendChild(loadingImage);
    bookDetailsNode.append(loadingContainer);
}

// Required function used to initialize any needed function and event listener.
dukagjiniBooks.init = function () {

    // Add an event listener to the custom event dispatched by the content script.
    window.addEventListener(dukagjiniBooks.eventName, handle);

    let dukagjiniProviderData = {
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

    function createMissingDataElement(message) {
        let missingDataContainer = document.createElement('div');
        missingDataContainer.className = 'dukagjinibooks-missing-data-container';
        missingDataContainer.textContent = message;

        let missingDataAppName = document.createElement('span');
        missingDataAppName.className = 'dukagjinibooks-missing-data-app-name';
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
        ratingsContainer.className = 'dukagjinibooks-ratings-container';

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

    function handle(event) {
        let loadingElement = document.querySelector('.dukagjinibooks-loading-container');
        if (loadingElement) {
            loadingElement.remove();
        }

        let bookDetailsNode = document.querySelector('#book-details .container');

        let existingContainer = document.querySelector('.dukagjinibooks-container');
        if (existingContainer) {
            // Container already exists, don't add it again
            return;
        }
        let dataContainer = document.createElement('div');
        dataContainer.className = 'row dukagjinibooks-container';

        // Keep only the successful responses
        let responses = event.detail.responses;
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
            bookDetailsNode.append(dataContainer);
            console.log('Zana response: ', event.detail.responses);
            return;
        }

        addRatings(validResponses, dataContainer);
        addBookMetadata(validResponses, dataContainer);
        bookDetailsNode.append(dataContainer);
    }

    function createBookMetadata(numberOfPages) {
        let metadataContainer = document.createElement('div');
        metadataContainer.className = 'dukagjinibooks-metadata-container';
        metadataContainer.textContent = `
            Number of pages: ${numberOfPages}
        `;
        return metadataContainer;
    }

    function createDescription(description) {
        let descriptionElement = document.createElement('div');
        descriptionElement.className = 'dukagjinibooks-description';
        descriptionElement.textContent = description;
        return descriptionElement;
    }

    function createRatings(type, rating, provider_link) {
        let ratingsElement = document.createElement('div');
        ratingsElement.className = 'dukagjinibooks-rating';
        // Rating type
        let ratingTypeElement = document.createElement('span');
        ratingTypeElement.className = 'dukagjinibooks-rating-type';

        // Rating stars
        let stars = createRatingStars(rating.average_rating);
        stars.className = 'dukagjinibooks-stars';
        ratingsElement.appendChild(stars);

        // Rating data
        let ratingDataElement = document.createElement('span');
        ratingDataElement.className = 'dukagjinibooks-rating-data';
        ratingDataElement.textContent = `
            ${rating.ratings_count} Reviews
            (${rating.average_rating.toFixed(2)} Average)
        `;
        ratingsElement.appendChild(ratingDataElement);

        let ratingPoweredByImageElement = document.createElement('img');
        ratingPoweredByImageElement.src = dukagjiniProviderData[type].poweredByImage.url;
        ratingPoweredByImageElement.className = 'dukagjinibooks-rating-type-image';
        ratingPoweredByImageElement.width = dukagjiniProviderData[type].poweredByImage.width;
        ratingsElement.appendChild(ratingPoweredByImageElement);

        let ratingExternalLinkElement = document.createElement('a');
        ratingExternalLinkElement.href = provider_link;
        ratingExternalLinkElement.target = '_blank';
        ratingExternalLinkElement.className = 'dukagjinibooks-rating-external-link';
        ratingsElement.appendChild(ratingExternalLinkElement);

        let ratingExternalLinkImageElement = document.createElement('img');
        ratingExternalLinkImageElement.src = browser.runtime.getURL('images/dukagjini/external_link.svg');
        ratingExternalLinkImageElement.className = 'dukagjinibooks-rating-external-link-image';
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
            starElement.classList.add('b-rating-star', 'dukagjinibooks-star');

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
}

// Used to make this module available for unit testing
if (typeof module !== 'undefined') {
    module.exports = {
        dukagjiniBooks: dukagjiniBooks
    }
}

