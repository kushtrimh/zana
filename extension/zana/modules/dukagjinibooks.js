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
    console.log(responses);
    let bookDetailsNode = document.querySelector('#book-details .container');

    let dataContainer = document.createElement("div");
    dataContainer.className = "row";
    dataContainer.innerHTML = `
        <p>${responses[0].body.data.description}</p>
    `;

    bookDetailsNode.prepend(dataContainer);
}
