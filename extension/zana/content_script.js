// Put all the javascript code here, that you want to execute after page load.
const apiUrl = 'https://api.zanareads.com/books';

const host = window.location.host;
const hosts = {
    'dukagjinibooks.com': {
        isbn: dukagjiniBooksIsbn,
        successHandler: dukagjiniBooks,
        failureHandler: null,
        queryBookData: true,
    }
};

const hostConfig = hosts[host];
if (hostConfig) {
    if (hostConfig.queryBookData) {
        const isbn = hostConfig.isbn();
        const queries = [
            apiUrl + '?type=googlebooks&isbn=' + isbn,
            apiUrl + '?type=openlibrary&isbn=' + isbn
        ];

        const promises =queries.map(url => fetch(url).then(response => response.json()));
        Promise.all(promises)
            .then(responses => console.log(responses));

    }
    // hostConfig.func();
}



