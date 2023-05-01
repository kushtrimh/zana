# Contributing to Zana

Thank you for taking time to contribute to Zana.

In this guide you will get an overview of the contribution workflow from opening an issue, creating a PR, reviewing, and merging the PR.
The guide will also show you how to set up your local environment for making changes to, building and testing Zana.

By participating to Zana, you are expected to uphold our [Code of Conduct](./CODE_OF_CONDUCT.md).

### Table of Contents

- [Questions and discussions](#questions-and-discussions)
- [Reporting a Bug](#reporting-a-bug)
- [Suggesting Enhancements](#suggesting-enhancements)
- [Local Setup and Development](#local-setup-and-development)
- [Style Guides](#style-guides)
  - [Git Commit Messages](#git-commit-messages)
  - [Services/Release (Rust) Style Guides](#services-style-guides)
  - [Extension (JavaScript) Style Guides](#extension-style-guides)
  - [Deployment (Java) Style Guides](#deployment-style-guides)
- [Your First Contribution](#your-first-contribution)
- [Pull Requests](#pull-requests)
- [Adding Support For a New Bookstore](#adding-support-for-a-new-bookstore)
- [How To Handle Breaking Changes](#how-to-handle-breaking-changes)

## Questions and discussions

If you have any question, suggestion or you are interested to discuss Zana, feel free to add a comment with your question or suggestion at the open
[Zana Q&A](https://github.com/kushtrimh/zana/discussions/17) discussion, or create a new discussion yourself at our [discussions](https://github.com/kushtrimh/zana/discussions) page.

## Reporting a Bug

If you come across any problem with Zana, and you want to report an issue, you can follow this guide.
Before reporting a new bug however, please check the [issues](https://github.com/kushtrimh/zana/issues) page in case
a similar bug has already been reported.

To create a bug issue, you can go to the [issues](https://github.com/kushtrimh/zana/issues) page and click on the `New issue` button.
You will be redirected to another page with a list of issues templates, and there you can choose the `Bug report` template.

For the title of the bug, please give something short that best describes your problem.
On the bug report content pane, a list of headings will show the expected information.
Please make sure to add as many details as you can. 
Steps to reproduce, console log errors, screenshots and browser type and version are particularly helpful to resolve the issue faster.

You can preview the bug issue template [here](.github/ISSUE_TEMPLATE/bug_report.md).

If you simply have a question and Zana, please do not raise a new bug issue for it, instead check our 
[questions and discussions](#questions-and-discussions) section.

## Suggesting Enhancements














### Required tools

- [Rust 1.68.0](https://www.rust-lang.org/) (`services`, `release`)
- [Cargo Lambda 0.17.1](https://www.cargo-lambda.info/) (`services`)
- [Node.js v18.14.2](https://nodejs.org/en) (`extension`, `deployment`)
- [Web-ext 7.6.1](https://github.com/mozilla/web-ext) (`extension`)
- [AWS CLI 2.10.3](https://aws.amazon.com/cli/) (`deployment`)
- [AWS CDK + AWS CDK CLI 2.67.0](https://docs.aws.amazon.com/cdk/v2/guide/cli.html) (`deployment`)
- [Java 17](https://www.oracle.com/java/technologies/javase/jdk17-archive-downloads.html) (`deployment`)

### Services

The services are Rust crates (libraries and binaries) that make the clients that talk to third-party APIs, and the AWS Lambda function
that serves the data retrieved from the clients in a generic format.

#### Zana service

The `zana` service is a Rust crate that provides functionality to retrieve book information from third-party APIs.
It does so by providing a generic clients for each third-party API, and a generic data model that is used to represent the data retrieved from the APIs.
Even though currently it is used only by `zana_lambda`, it is built as a separate library crate so that it can be used easily later on in other Rust binary crates
built for other cloud providers or as a generic HTTP service.

You can build `zana` service by running the following command in the `services/zana` directory

```bash
cargo build
```
or test it by running
```bash
cargo test
```

#### Zana Lambda

`zana_lambda` is a Rust binary crate that contains the AWS Lambda function that serves the data retrieved from third-party APIs.
It uses the `zana` library crate to make clients for each supported third-party API.

##### Environment variables

###### Required

- `ZANA_ENV`, environment that is used on AWS Parameter Store queries as part of the key, to provide support for same parameters on multiple environments.

###### Set automatically AWS Lambda runtime
- `AWS_SESSION_TOKEN`, token used to access AWS Parameter Store.
- `PARAMETERS_SECRETS_EXTENSION_HTTP_PORT`, [port to be used by the AWS Parameter Store lambda extension](https://docs.aws.amazon.com/secretsmanager/latest/userguide/retrieving-secrets_lambda.html#retrieving-secrets_lambda_env-var).

###### Optional

- `ZANA_GOOGLE_BOOKS_URL`, use if you do not want to fetch the URL from AWS Parameter Store.
- `ZANA_GOOGLE_BOOKS_KEY`, use if you do not want to fetch the API key from AWS Parameter Store.
- `ZANA_OPENLIBRARY_URL`, use if you do not want to fetch the URL from AWS Parameter Store.

##### AWS Parameter Store support

[AWS Parameter Store](https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-parameter-store.html) is used
to retrieve parameters required to initialize clients for different third-party APIs, if those parameters are not provided as
environment variables.

`zana_lambda` uses the following parameter names to query parameter from the AWS Parameter Store. `prod` represents the environment, and it should change
based on the environment that is used.
- `/zana/prod/google-books-url`
- `/zana/prod/google-books-key`
- `/zana/prod/openlibrary-url`

Value of the environment variable `ZANA_ENV` is used as part of the key when retrieving parameter values,
in order to provide support for multiple environments at the same time.

##### Adding to AWS Parameter Store for production

```sh
aws ssm put-parameter --name "/zana/prod/google-books-url" --value "VALUE-HERE" --type String
aws ssm put-parameter --name "/zana/prod/google-books-key" --value "VALUE-HERE" --type SecureString
aws ssm put-parameter --name "/zana/prod/openlibrary-url" --value "VALUE-HERE" --type String
```

##### Running locally

The easiest way to test `zana_lambda` locally, is with [cargo lambda](https://www.cargo-lambda.info/guide/getting-started.html).

Once `cargo lambda` is installed, you need to create a `.env` file in the `services/zana_lambda` directory.
This file should contain the following variables when running locally

```
AWS_SESSION_TOKEN=token-example-123
PARAMETERS_SECRETS_EXTENSION_HTTP_PORT=2773
ZANA_ENV=local
ZANA_GOOGLE_BOOKS_URL=https://www.googleapis.com
ZANA_GOOGLE_BOOKS_KEY=<YOUR-GOOGLE-BOOKS-KEY>
ZANA_OPENLIBRARY_URL=https://openlibrary.org
```

`AWS_SESSION_TOKEN` and `PARAMETERS_SECRETS_EXTENSION_HTTP_PORT` are variables which are provided by the Lambda Runtime
on other environment, and they're required to communicate with _AWS Parameter Store_.
Since the _Parameter Store_ is not used when testing locally, those 2 environment variables do not have any function on local environment.

Additional environment variables (not required on other envs) are added to replace values that would be fetched from the _Parameter Store_.
`ZANA_GOOGLE_BOOKS_URL`, `ZANA_GOOGLE_BOOKS_KEY` and `ZANA_OPENLIBRARY_URL` are meant as a replacement for _Parameter Store_ values when running locally.

Those can be utilized on other environments as well, if you do not want to use _Parameter Store_ to fetch those values.

##### Starting the lambda locally

To start the lambda locally, change your directory to `services/zana_lambda` and run
```sh
cargo build
```

After the build is completed run
```sh
cargo lambda watch --env-file .env
```
where `.env` is the file containing the environment variables mentioned above.

##### Query using any HTTP client

The lambda should start on port `9000` by default.

_Request for OpenLibrary_
```
GET http://localhost:9000/lambda-url/zana_lambda?type=openlibrary&isbn=9781591026419
```

_Request for Google Books_
```
GET http://localhost:9000/lambda-url/zana_lambda?type=googlebooks&isbn=9781591026419
```

### Deployment

AWS CDK with _Java_ is used to define the infrastructure, which is primarily built around serverless services provided by AWS.
The deployment is done automatically once a PR is merged into the `main` branch, using _GitHub Actions_.

![Zana AWS Architecture](./docs/zana_aws.drawio.png)

#### Useful commands

Change your directory to `deployment/zana_aws` to run the following commands

* `mvn package`     compile and run tests
* `cdk ls`          list all stacks in the app
* `cdk synth`       emits the synthesized CloudFormation template
* `cdk deploy`      deploy this stack to your default AWS account/region
* `cdk diff`        compare deployed stack with current state
* `cdk docs`        open CDK documentation

#### What is required before the deployment

- To have a hosted zone in `Route 53`
- Certificate for *.yourdomain.com in us-east-1, and to create records for it in `Route 53`
- Parameters set into `AWS Parameter Store`

#### Required parameters on Parameter store

`prod` is used by default if not environment is specified.

- `/zana/prod/google-books-url` - type: `String`, URL for Google Books API
- `/zana/prod/google-books-key` - type: `SecureString`, API Key for Google Books API
- `/zana/prod/openlibrary-url` - type: `String`, URL for OpenLibrary API

- `/zana/prod/certificate-arn` - type: `String`, Certificate ARN from AWS Certificate Manager
- `/zana/prod/api-host` - type: `String`, API host name to be used by zana
- `/zana/prod/hosted-zone-id` - type: `String`, Route53 Hosted zone ID
- `/zana/prod/hosted-zone-name` - type: `String`, Route54 Hosted zone name
- `/zana/prod/cors-allow-origins` - type: `String`, Comma seperated string of allowed origins
- `/zana/prod/lambda-ssm-extension-arn` - type: `String`, ARN for Lambda SSM extension
- `/zana/prod/lambda-insights-extension-arn` - type: `String`, ARN for Lambda Insights extension
