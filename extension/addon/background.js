if (typeof browser === 'undefined') {
    browser = chrome;
}

let targets = [
    "https://dukagjinibooks.com/api/products/*"
];

let hostsConfig = {
    'dukagjinibooks.com': {
        pattern: /^https:\/\/dukagjinibooks\.com\/api\/products\/\d+$/,
        skipPatternCheck: false,
    }
}

function listener(details) {
    const url = details.url;
    const host = new URL(url).host;
    const hostConfig = hostsConfig[host];
    if (hostConfig) {
        if (hostConfig.skipPatternCheck || hostConfig.pattern.test(url)) {
            browser.tabs.sendMessage(details.tabId, {});
        }
    }
}

browser.webRequest.onCompleted.addListener(
    listener,
    {urls: targets}
);
