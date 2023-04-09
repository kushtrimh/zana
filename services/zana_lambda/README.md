# zana_lambda

## Environment variables

### Required

- `ZANA_ENV`, environment that is used on AWS Parameter Store queries as part of the key, to provide support for same parameters on multiple environments.

### Set automatically AWS Lambda runtime
- `AWS_SESSION_TOKEN`, token used to access AWS Parameter Store.
- `PARAMETERS_SECRETS_EXTENSION_HTTP_PORT`, [port to be used by the AWS Parameter Store lambda extension](https://docs.aws.amazon.com/secretsmanager/latest/userguide/retrieving-secrets_lambda.html#retrieving-secrets_lambda_env-var).

### Optional

- `ZANA_GOOGLE_BOOKS_URL`, use if you do not want to fetch the URL from AWS Parameter Store.
- `ZANA_GOOGLE_BOOKS_KEY`, use if you do not want to fetch the API key from AWS Parameter Store.
- `ZANA_OPENLIBRARY_URL`, use if you do not want to fetch the URL from AWS Parameter Store.

## AWS Parameter Store support

[AWS Parameter Store](https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-parameter-store.html) is used
to provide retrieve parameters required to initialize clients for different providers, if those parameters are not provided as
environment variables.

`zana_lambda` uses the following parameter names to query parameter from the parameter store. `prod` represents the environment, and it should change
based on the environment that is used.
- `/zana/prod/google-books-url`
- `/zana/prod/google-books-key`
- `/zana/prod/openlibrary-url`

Values of environment variable `ZANA_ENV` is used as part of the key when retrieving parameter values, 
in order to provide support for multiple environments at the same time.

### Add to AWS Parameter Store for production

```sh
aws ssm put-parameter --name "/zana/prod/google-books-url" --value "VALUE-HERE" --type String
aws ssm put-parameter --name "/zana/prod/google-books-key" --value "VALUE-HERE" --type SecureString
aws ssm put-parameter --name "/zana/prod/openlibrary-url" --value "VALUE-HERE" --type String
```

## Running locally

The easiest way to test `zana_lambda` locally, is to install
[cargo lambda](https://www.cargo-lambda.info/guide/getting-started.html).

Once _cargo lambda_ is installed, you need to create a `.env` file.
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
on other environment, and they're required to communicate with _Parameter Store_.
Since _Parameter Store_ is not used when testing locally, those 2 environment variables do not have any function on local environment.

Additional environment variables (not required on other envs) are added to replace values that would be fetched from _Parameter Store_.
`ZANA_GOOGLE_BOOKS_URL`, `ZANA_GOOGLE_BOOKS_KEY` and `ZANA_OPENLIBRARY_URL` are meant as a replacement for _Parameter Store_ values when running locally.

Those can be utilized on other environments as well.

### Starting lambda locally

To start the lambda locally

```sh
cargo build
```

After the build is completed run 
```sh
cargo lambda watch --env-file .env
```
where `.env` is the file containing the environment variables mentioned above.

### Query using any HTTP client

The lambda should start on port `9000` by default.

_OpenLibrary_
```
GET http://localhost:9000/lambda-url/zana_lambda?type=openlibrary&isbn=9781591026419
```

_Google Books_
```
GET http://localhost:9000/lambda-url/zana_lambda?type=googlebooks&isbn=9781591026419
```
