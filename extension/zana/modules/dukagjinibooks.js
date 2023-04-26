let dukagjini = {
    eventName: 'dukagjiniEvent',
    queryBookData: true,
};

dukagjini.retrieveIsbn = function() {
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

dukagjini.init = function() {
    window.addEventListener(dukagjini.eventName, handle);
}

function handle(event) {
    let responses = event.detail.responses;

    // TODO: Check if response if valid

    let bookDetailsNode = document.querySelector('#book-details .container');

    let existingContainer = document.querySelector('.dukagjinibooks-container');
    if (existingContainer) {
        return;
    }
    // Container
    let dataContainer = document.createElement('div');
    dataContainer.className = 'row dukagjinibooks-container';

    // Ratings
    let ratingsContainer = document.createElement('div');
    ratingsContainer.className = 'dukagjinibooks-ratings-container';

    for (let response of responses) {
        let rating = response.body.rating;
        if (rating) {
            let ratingsElement = createRatings(response.type, rating);
            ratingsContainer.appendChild(ratingsElement);
        } else {
            // TODO: add message that no rating was found
        }
    }
    dataContainer.appendChild(ratingsContainer);

    let numberOfPages;
    let description;
    for (let response of responses) {
        const data = response.body.data;
        if (data.page_count && !numberOfPages) {
            numberOfPages = data.page_count;
        }
        if (data.description && !description) {
            description = data.description;
        }
    }

    // Book metadata
    const numberOfPagesElement = createBookMetadata(numberOfPages);
    if (numberOfPagesElement) {
        dataContainer.appendChild(numberOfPagesElement);
    }
    // Description
    // todo: check if description here, then create and add el;ement
    const descriptionElement = createDescription(description);
    if (descriptionElement) {
        dataContainer.appendChild(descriptionElement);
    }

    bookDetailsNode.append(dataContainer);
}

function createBookMetadata(numberOfPages) {
    if (numberOfPages) {
        let metadataContainer = document.createElement('div');
        metadataContainer.className = 'dukagjinibooks-metadata-container';
        metadataContainer.textContent = `
            Number of pages: ${numberOfPages}
        `;
        return metadataContainer;
    }
}

function createDescription(description) {
    if (description) {
        let descriptionElement = document.createElement('div');
        descriptionElement.className = 'dukagjinibooks-description';
        descriptionElement.textContent = description;
        return descriptionElement;
    }
}

function createRatings(type, rating) {
    let ratingsElement = document.createElement('div');
    ratingsElement.className = 'dukagjinibooks-rating';
    // Rating type
    let ratingTypeElement = document.createElement('span');
    ratingTypeElement.className = 'dukagjinibooks-rating-type';

    let ratingTypeImageElement = document.createElement('img');
    ratingTypeImageElement.src = browser.runtime.getURL('images/' + type + '.svg');
    ratingTypeImageElement.className = 'dukagjinibooks-rating-type-image';
    ratingTypeElement.appendChild(ratingTypeImageElement);
    ratingsElement.appendChild(ratingTypeElement);

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
        starsElement.insertAdjacentHTML('beforeend', `
            <span class="b-rating-star">
                <span class="b-rating-icon">
                    <svg viewBox="0 0 16 16" width="1em" height="1em" focusable="false" role="img" aria-label="star" xmlns="http://www.w3.org/2000/svg" fill="currentColor" class="bi-star b-icon bi">
                        <g>
                            <path d="${starPath}">
                            </path>
                        </g>
                    </svg>
                </span>
            </span>
        `)
    }
    return starsElement;
}
