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

// Initialize the host configuration.
// This should define any needed function, and add any needed event listener.
hostConfig.init();

browser.runtime.onMessage.addListener(() => {
    // In case when an SPA is used, each valid XHR request that is intercept by the background script will trigger an
    // update, causing the data to be retrieved again for a different book.
    update();
});

// Once the page is loaded, update the page with the book data.
update();

function update() {
    const isbn = hostConfig.retrieveIsbn();
    if (hostConfig) {
        if (hostConfig.queryBookData) {
            if (isbn) {
                notify(isbn);
            }
        }
    }
}

function notify(isbn) {
    // Display a loading indicator, as it was defined in the host configuration.
    hostConfig.loading();

    const responsePromises = retrieveBookData(isbn);
    Promise.all(responsePromises)
        .then(responses => {
            // Dispatch a custom event to the host module listeners, which will handle the response
            // and display the needed HTML elements.
            const event = new CustomEvent(hostConfig.eventName, {
                detail: {
                    responses: responses,
                }
            });
            dispatchEvent(event);
        });
}

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
