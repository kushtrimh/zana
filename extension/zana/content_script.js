const apiUrl = 'https://api.zanareads.com/books';

const host = window.location.host;
const hostsConfig = {
    'dukagjinibooks.com': dukagjini,
};

const hostConfig = hostsConfig[host];
hostConfig.init();

browser.runtime.onMessage.addListener((message) => {
    const isbn = hostConfig.retrieveIsbn();
    if (hostConfig) {
        if (hostConfig.queryBookData) {
            if (isbn) {
                update(isbn);
            }
        }
    }
});

function update(isbn) {
    const responsePromises = retrieveBookData(isbn);
    Promise.all(responsePromises)
        .then(responses => {
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
            return {
                isbn: isbn,
                body: error,
                status: 500,
                type: query.type,
            };
        }));
}
