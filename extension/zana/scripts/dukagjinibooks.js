function dukagjiniBooksIsbn() {
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

function dukagjiniBooks() {
    let bookItems = document.querySelectorAll('.book-item-specifics h5 span');
    let isbnItem;

    for (let bookItem of bookItems) {
        if (bookItem.textContent.includes('ISBN')) {
            isbnItem = bookItem;
            break;
        }
    }

    if (isbnItem) {
        let isbn = isbnItem.parentElement.textContent.split(':')[1].trim();

        let url = 'https://api.zanareads.com/books?type=googlebooks&isbn=' + isbn;
        fetch(url)
            .then(response => response.json())
            .then(response => {
                let bookItem = document.createElement('div');
                bookItem.innerHTML = `
                <div class="book-item-specifics">
                    <h5>Ratings</h5>
                    <div class="book-item-details">
                        <div class="book-item-detail">
                            <span>Data: </span>
                            <p>${response.rating.average_rating}</p>
                            <p>${response.rating.ratings_count}</p>
                            <p>${response.data.page_count}</p>
                            <p>${response.data.description}</p>
                        </div>
                    </div>
                </div>
            `;
                isbnItem.parentElement.parentElement.parentElement.appendChild(bookItem);
            });
    }
}
