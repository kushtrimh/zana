/*
 This module is used to handle any functionality related with dukagjinibooks.com.
 */

// Host configuration
let dukagjiniBooks = {};

// Required function used to retrieve the ISBN, which is used to query the book data.
dukagjiniBooks.isOnBookPage = function () {
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
    return null;
}

// Used to make this module available for unit testing
if (typeof module !== 'undefined') {
    module.exports = {
        dukagjiniBooks: dukagjiniBooks
    }
}

