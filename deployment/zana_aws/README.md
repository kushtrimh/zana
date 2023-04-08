# Zana AWS

## Useful commands

 * `mvn package`     compile and run tests
 * `cdk ls`          list all stacks in the app
 * `cdk synth`       emits the synthesized CloudFormation template
 * `cdk deploy`      deploy this stack to your default AWS account/region
 * `cdk diff`        compare deployed stack with current state
 * `cdk docs`        open CDK documentation

## What is required before

- To have a hosted zone in Route53
- Certificate for *.yourdomain.com in us-east-1, and create records for it in Route53
- Params in param store

## Required parameters on Parameter store

In this case `prod` is the env used.

- `/zana/prod/google-books-url` - type: `String`, Google Books API URL
- `/zana/prod/google-books-key` - type: `SecureString`, Google Books API key
- `/zana/prod/openlibrary-url` - type: `String`, OpenLibrary URL

- `/zana/prod/certificate-arn` - type: `String`, Certificate ARN from AWS Certificate Manager
- `/zana/prod/api-host` - type: `String`, API host name to be used by zana
- `/zana/prod/hosted-zone-id` - type: `String`, Route53 Hosted zone ID
- `/zana/prod/hosted-zone-name` - type: `String`, Route54 Hosted zone name
- `/zana/prod/cors-allow-origins` - type: `String`, Comma seperated string of allowed origins
- `/zana/prod/lambda-ssm-extension-arn` - type: `String`, ARN for Lambda SSM extension
- `/zana/prod/lambda-insights-extension-arn` - type: `String`, ARN for Lambda Insights extension
