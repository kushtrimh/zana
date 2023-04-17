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

    let bookDetailsNode = document.querySelector('#book-details .container');

    // Container
    let dataContainer = document.createElement("div");
    dataContainer.className = "row dukagjinibooks-container";

    // Ratings
    let ratingsElement = document.createElement("div");
    ratingsElement.className = "dukagjinibooks-ratings";
    ratingsElement.textContent = responses[0].body.rating.average_rating;
    dataContainer.appendChild(ratingsElement);

    let starElement = document.createElement('div');
    starElement.innerHTML = `
        <span class="b-rating-star flex-grow-1 b-rating-star-full"><span class="b-rating-icon"><svg viewBox="0 0 16 16" width="1em" height="1em" focusable="false" role="img" aria-label="star fill" xmlns="http://www.w3.org/2000/svg" fill="currentColor" class="bi-star-fill b-icon bi"><g><path d="M3.612 15.443c-.386.198-.824-.149-.746-.592l.83-4.73L.173 6.765c-.329-.314-.158-.888.283-.95l4.898-.696L7.538.792c.197-.39.73-.39.927 0l2.184 4.327 4.898.696c.441.062.612.636.283.95l-3.523 3.356.83 4.73c.078.443-.36.79-.746.592L8 13.187l-4.389 2.256z"></path></g></svg></span></span>
    `;
    starElement.className = "star-color";
    ratingsElement.appendChild(starElement);

    // Description
    let descriptionElement = document.createElement("div");
    descriptionElement.textContent = responses[0].body.data.description;
    dataContainer.appendChild(descriptionElement);

    bookDetailsNode.prepend(dataContainer);
}
