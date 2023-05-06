if (typeof browser === 'undefined') {
    // In case the browser API is not available, switch to Chrome API.
    browser = chrome;
}

function cleanHost(host) {
    return host.replace('www.', '');
}

// List of URLs to be intercepted
let targets = [
    'https://dukagjinibooks.com/api/products/*'
];

// List of hosts that are supported.
// Each host can have a pattern that is used to match the URLs further in case the `targets` definition is too broad.
// If the `targets` definition is enough to match the URLs, the `skipPatternCheck` can be set to `true` to skip the pattern check.
let hostsConfig = {
    'dukagjinibooks.com': {
        pattern: /^https:\/\/dukagjinibooks\.com\/api\/products\/\d+$/,
        skipPatternCheck: false,
    }
}

function listener(details) {
    const url = details.url;
    const host = cleanHost(new URL(url).host);
    const hostConfig = hostsConfig[host];
    if (hostConfig) {
        if (hostConfig.skipPatternCheck || hostConfig.pattern.test(url)) {
            // Sends a message to the content script to notify it that the URL has been intercepted.
            browser.tabs.sendMessage(details.tabId, {});
        }
    }
}

browser.webRequest.onCompleted.addListener(
    listener,
    {urls: targets}
);
