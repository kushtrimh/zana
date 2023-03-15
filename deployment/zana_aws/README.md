# Zana AWS

## Useful commands

 * `mvn package`     compile and run tests
 * `cdk ls`          list all stacks in the app
 * `cdk synth`       emits the synthesized CloudFormation template
 * `cdk deploy`      deploy this stack to your default AWS account/region
 * `cdk diff`        compare deployed stack with current state
 * `cdk docs`        open CDK documentation

Enjoy!

## What is required before

- To have a hosted zone already
- Certificate for *.zanareads.com in us-east-1, and create records for it in Route53
- Params in param store

## Required parameters on Parameter store

Add all parameters with the following label `prod`

- `/zana/prod/google-books-url` - type: `String`, Google Books API URL
- `/zana/prod/google-books-key` - type: `SecureString`, Google Books API key
- `/zana/prod/openlibrary-url` - type: `String`, OpenLibrary URL

- `/zana/prod/certificate-arn` - type: `String`, Certificate ARN from AWS Certificate Manager
- `/zana/prod/api-domain` - type: `String`, API Domain name to be used by zana
- `/zana/prod/hosted-zone-id` - type: `String`, Route53 Hosted zone ID
- `/zana/prod/hosted-zone-name` - type: `String`, Route54 Hosted zone name
- `/zana/prod/cors-allow-origins` - type: `String`, Allowed origins
- `/zana/prod/lambda-ssm-extension-arn` - type: `String`, ARN for Lambda SSM extension
