# zana

![build](https://github.com/kushtrimh/zana/actions/workflows/build.yml/badge.svg?branch=main)
![deploy](https://github.com/kushtrimh/zana/actions/workflows/deploy.yml/badge.svg?branch=main)

Zana is a browser extension that adds book information and ratings to supported bookstore websites.
The data is retrieved from a list of third-party book APIs.

## Installation

You can install the extension from the following stores:
- [Firefox](https://addons.mozilla.org/en-US/firefox/addon/zana/)
- [Chrome](https://chrome.google.com/webstore/detail/zana/dfjgjgjgjgjgjgjgjgjgjgjgjgjgjgj)
- [Edge](https://microsoftedge.microsoft.com/addons/detail/zana/dfjgjgjgjgjgjgjgjgjgjgjgjgjgjg)

The extension is not yet on _Microsoft Edge Add-ons_ store, but you can install it from the _Chrome Web Store_.

## Third-party APIs

Zana has the following clients for third-party APIs:
- Client for [Google Books API](https://developers.google.com/books)
- Client for [Open Library API](https://openlibrary.org/developers/api)
 
## Supported bookstores

- [Dukagjini Bookstore](https://dukagjinibooks.com/)

## Questions and discussions



## Contributing

If you're interested in contributing to the project, please check the [CONTRIBUTING.md](CONTRIBUTING.md) file.

## Local setup, development and building

TODO: Add link

## Reporting issues or requesting enhancements

TODO: Add link

## Project details

Zana is built as a monorepo, and it uses different tools for API clients, the browser extension, deployment, and release handling.
Modules are organized as follows:
- `services` - Rust crates that contain the API clients and the AWS Lambda function binary that serves the data retrieved by the clients.
- `extension` - Browser extension that is built using WebExtensions API.
- `deployment` - AWS CDK project that contains the infrastructure.
- `release` - Rust binary crate that helps with release management.

Zana is built primarily for Firefox, but it uses browser polyfills to support Chrome and Edge.
It _still_ does not support Firefox for Android, but it is planned to be supported in the future.

### Infrastructure

Zana uses AWS as the cloud provider, and is built around its serverless services.

![Zana AWS Architecture](./docs/zana_aws.drawio.png)

### Deployment and releases

Each PR that is merged into the `main` branch triggers the deployment pipeline, and with each deployment
a new release is created. Except the `extension` all other modules are built and deployed automatically.
The extension is built manually, and then submitted into the browser stores as an update.
