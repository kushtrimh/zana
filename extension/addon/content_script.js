if (typeof browser === 'undefined') {
    // In case the browser API is not available, switch to Chrome API.
    browser = chrome;
}

function cleanHost(host) {
    return host.replace('www.', '');
}

const apiUrl = 'https://api.zanareads.com/books';

const host = cleanHost(window.location.host);

// List of supported hosts.

const hostsConfig = {
    'dukagjinibooks.com': dukagjiniBooks,
};
const hostConfig = hostsConfig[host];

browser.runtime.onMessage.addListener(() => {
    // In case when an SPA is used, each valid XHR request that is intercept by the background script will trigger an
    // update, causing the data to be retrieved again for a different book.
    update();
});
// Once the page is loaded, update the page with the book data.

zanaInit();
window.addEventListener('load', function () {
    update();
})

function update() {
    const isbn = hostConfig.isOnBookPage();
    if (hostConfig) {
        if (hostConfig) {
            if (isbn) {
                notify(isbn);
            } else {
                notifyForOtherPages();
            }
        }
    }
}

function notify(isbn) {
    const event = new CustomEvent(ZANA_BOOK_EVENT, {
        detail: {
            isbn: isbn,
        }
    });
    dispatchEvent(event);
}

function notifyForOtherPages() {
    const event = new CustomEvent(ZANA_NON_BOOK_EVENT, {});
    dispatchEvent(event);
}
