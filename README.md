# zana

![build](https://github.com/kushtrimh/zana/actions/workflows/build.yml/badge.svg?branch=main)
![deploy](https://github.com/kushtrimh/zana/actions/workflows/deploy.yml/badge.svg?branch=main)
![release](https://github.com/kushtrimh/zana/actions/workflows/release.yml/badge.svg?branch=main)

Zana is a browser extension that adds book information and ratings to supported bookstore websites.
The data is retrieved from a list of third-party book APIs.

## Installation

You can install the extension from the following stores:
- [Firefox](https://addons.mozilla.org/en-US/firefox/addon/zanareads/)
- [Chrome and Edge](https://chrome.google.com/webstore/detail/zana/gpabeacabfcdnclngneckekecoiieodd)

The extension is not yet on _Microsoft Edge Add-ons_ store, but you can install it on _Edge_ from the _Chrome Web Store_.

## Third-party APIs

Zana has the following clients for third-party APIs:
- Client for [Google Books API](https://developers.google.com/books)
- Client for [Open Library API](https://openlibrary.org/developers/api)
 
## Supported bookstores

- [Dukagjini Bookstore](https://dukagjinibooks.com/)

## Questions and discussions

For any questions or discussions, please check the [Questions and Discussions](./CONTRIBUTING.md#questions-and-discussions) guide.

## Contributing

If you're interested in contributing to the project, please check the [CONTRIBUTING.md](CONTRIBUTING.md) file.

To add support for a new bookstore please check the [Adding Support For a New Bookstore](./CONTRIBUTING.md#adding-support-for-a-new-bookstore) guide.

## Local setup, development and building

For setting up, building, and testing Zana locally, please check the [Local Setup and Development](./CONTRIBUTING.md#local-setup-and-development) guide.

## Reporting issues or requesting enhancements

To report a bug issue, please check the [Reporting a Bug](./CONTRIBUTING.md#reporting-a-bug) guide.

To suggest an enhancement, please check the [Suggesting Enhancements](./CONTRIBUTING.md#suggesting-enhancements) guide.

## Project details

Zana is built as a monorepo, and it uses different tools for API clients, the browser extension, deployment, and release handling.
Modules are organized as follows:
- `services` - Rust crates that contain the API clients and the AWS Lambda function binary that serves the data retrieved by the clients.
- `extension` - Browser extension that is built using WebExtensions API.
- `deployment` - AWS CDK project that contains the infrastructure.
- `tools` - Scripts and binaries that help with:
  - Extension local development and packaging for certain platforms.
  - Release management.

Zana is built primarily for Firefox, but it uses browser polyfills to support Chrome and Edge.
The _Firefox_ extension is built with *Manifest v2*, and the _Chrome_ extension with *Manifest v3*.
The whole extension package will be migrated and adapted to *Manifest v3* in the near future.

Support for _Firefox on Android_ is not yet available, but it is planned to be added in the future.

### Infrastructure

Zana is deployed on AWS, and it is built around its serverless services.

![Zana AWS Architecture](./docs/zana_aws.drawio.png)

### Deployment and releases

Each PR that is merged into the `main` branch triggers the deployment pipeline, and with each deployment
a new release is created (_This process may change in the future_).
Except the `extension` which is built and published manually, all other modules are built and deployed automatically.
