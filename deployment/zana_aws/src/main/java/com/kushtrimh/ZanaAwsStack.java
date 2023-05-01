package com.kushtrimh;

import software.amazon.awscdk.Duration;
import software.amazon.awscdk.Stack;
import software.amazon.awscdk.StackProps;
import software.amazon.awscdk.services.apigateway.AccessLogFormat;
import software.amazon.awscdk.services.apigateway.CfnAccount;
import software.amazon.awscdk.services.apigateway.Cors;
import software.amazon.awscdk.services.apigateway.CorsOptions;
import software.amazon.awscdk.services.apigateway.EndpointConfiguration;
import software.amazon.awscdk.services.apigateway.EndpointType;
import software.amazon.awscdk.services.apigateway.LambdaIntegration;
import software.amazon.awscdk.services.apigateway.LogGroupLogDestination;
import software.amazon.awscdk.services.apigateway.MethodLoggingLevel;
import software.amazon.awscdk.services.apigateway.RestApi;
import software.amazon.awscdk.services.apigateway.StageOptions;
import software.amazon.awscdk.services.certificatemanager.Certificate;
import software.amazon.awscdk.services.cloudfront.BehaviorOptions;
import software.amazon.awscdk.services.cloudfront.CachePolicy;
import software.amazon.awscdk.services.cloudfront.CacheQueryStringBehavior;
import software.amazon.awscdk.services.cloudfront.Distribution;
import software.amazon.awscdk.services.cloudfront.ResponseHeadersCorsBehavior;
import software.amazon.awscdk.services.cloudfront.ResponseHeadersPolicy;
import software.amazon.awscdk.services.cloudfront.origins.HttpOrigin;
import software.amazon.awscdk.services.iam.Effect;
import software.amazon.awscdk.services.iam.ManagedPolicy;
import software.amazon.awscdk.services.iam.PolicyStatement;
import software.amazon.awscdk.services.iam.Role;
import software.amazon.awscdk.services.iam.ServicePrincipal;
import software.amazon.awscdk.services.lambda.AutoScalingOptions;
import software.amazon.awscdk.services.lambda.Code;
import software.amazon.awscdk.services.lambda.Function;
import software.amazon.awscdk.services.lambda.LambdaInsightsVersion;
import software.amazon.awscdk.services.lambda.LayerVersion;
import software.amazon.awscdk.services.lambda.Runtime;
import software.amazon.awscdk.services.lambda.UtilizationScalingOptions;
import software.amazon.awscdk.services.logs.LogGroup;
import software.amazon.awscdk.services.logs.RetentionDays;
import software.amazon.awscdk.services.route53.ARecord;
import software.amazon.awscdk.services.route53.HostedZone;
import software.amazon.awscdk.services.route53.HostedZoneAttributes;
import software.amazon.awscdk.services.route53.RecordTarget;
import software.amazon.awscdk.services.route53.targets.CloudFrontTarget;
import software.amazon.awscdk.services.ssm.StringParameter;
import software.constructs.Construct;

import java.util.Arrays;
import java.util.List;
import java.util.Map;

/**
 * @author Kushtrim Hajrizi
 */
public class ZanaAwsStack extends Stack {

    public ZanaAwsStack(final Construct scope, final String id) {
        this(scope, id, null, "test");
    }

    public ZanaAwsStack(final Construct scope, final String id, final StackProps props, final String zanaEnv) {
        super(scope, id, props);

        // Retrieve CORS configuration, which allows only certain origins to access the API
        var corsAllowedOrigins = Arrays.stream(StringParameter.valueForStringParameter(this,
                                String.format("/zana/%s/cors-allow-origins", zanaEnv))
                        .split(","))
                .toList();

        var booksDataLambda = createZanaBooksDataLambda(zanaEnv);
        var restApi = createZanaRestApi(zanaEnv, booksDataLambda, corsAllowedOrigins);
        var distribution = createCloudFrontDistribution(zanaEnv, restApi, corsAllowedOrigins);

        // Hosted zone configuration
        var hostedZoneId = StringParameter.valueForStringParameter(this,
                String.format("/zana/%s/hosted-zone-id", zanaEnv));
        var hostedZoneName = StringParameter.valueForStringParameter(this,
                String.format("/zana/%s/hosted-zone-name", zanaEnv));

        var hostedZone = HostedZone.fromHostedZoneAttributes(this, "zana-hosted-zone", HostedZoneAttributes
                .builder()
                .hostedZoneId(hostedZoneId)
                .zoneName(hostedZoneName)
                .build());

        ARecord.Builder.create(this, "zana-api-domain-record")
                .zone(hostedZone)
                .recordName("api")
                .target(RecordTarget.fromAlias(new CloudFrontTarget(distribution)))
                .build();
    }

    private Distribution createCloudFrontDistribution(String zanaEnv, RestApi restApi, List<String> corsAllowedOrigins) {
        var certificateArn = StringParameter.valueForStringParameter(this,
                String.format("/zana/%s/certificate-arn", zanaEnv));
        var apiDomain = StringParameter.valueForStringParameter(this, String.format("/zana/%s/api-host", zanaEnv));

        String apiDomainName = restApi.getRestApiId() + ".execute-api." + this.getRegion() + ".amazonaws.com";
        return Distribution.Builder.create(this, "zana-distribution")
                .certificate(Certificate.fromCertificateArn(this, "zana-certificate", certificateArn))
                .domainNames(List.of(apiDomain))
                .defaultBehavior(BehaviorOptions.builder()
                        .cachePolicy(CachePolicy.Builder.create(this, "zana-distribution-cache-policy")
                                .comment("Caching policy for Zana books API")
                                .defaultTtl(Duration.hours(6))
                                .maxTtl(Duration.hours(12))
                                .enableAcceptEncodingGzip(true)
                                .queryStringBehavior(CacheQueryStringBehavior.all())
                                .build())
                        .responseHeadersPolicy(ResponseHeadersPolicy.Builder.create(this, "zana-distribution-response-header-policy")
                                .corsBehavior(ResponseHeadersCorsBehavior.builder()
                                        .accessControlAllowHeaders(Cors.DEFAULT_HEADERS)
                                        .accessControlAllowMethods(List.of("GET"))
                                        .accessControlAllowOrigins(corsAllowedOrigins)
                                        .accessControlAllowCredentials(false)
                                        .originOverride(true)
                                        .build())
                                .build())
                        .origin(HttpOrigin.Builder.create(apiDomainName).originPath("/prod").build())
                        .build())
                .enableLogging(true)
                .logFilePrefix("zana-distribution-access-logs/")
                .build();
    }

    private Function createZanaBooksDataLambda(String zanaEnv) {
        var lambdaSSMExtensionArn = StringParameter.valueForStringParameter(this,
                String.format("/zana/%s/lambda-ssm-extension-arn", zanaEnv));
        var lambdaInsightsExtensionArn = StringParameter.valueForStringParameter(this,
                String.format("/zana/%s/lambda-insights-extension-arn", zanaEnv));

        var lambdaPolicy = ManagedPolicy.Builder.create(this, "zana-lambda-ssm-read-only-access")
                .description("Provides read only access to zana related entries on AWS Parameter Store")
                .statements(
                        List.of(
                                PolicyStatement.Builder.create()
                                        .effect(Effect.ALLOW)
                                        .actions(List.of("ssm:GetParameter"))
                                        .resources(List.of("arn:aws:ssm:*:*:parameter/zana/*"))
                                        .build(),
                                PolicyStatement.Builder.create()
                                        .effect(Effect.ALLOW)
                                        .actions(List.of("kms:Decrypt"))
                                        .resources(List.of("*"))
                                        .build()))
                .build();

        var lambdaRole = Role.Builder.create(this, "zana-books-lambda-role")
                .assumedBy(new ServicePrincipal("lambda.amazonaws.com"))
                .description(
                        "Allows lambda functions to retrieve parameters from AWS SSM. Intended to be by Zana book handler lambdas.")
                .managedPolicies(List.of(
                        ManagedPolicy.fromAwsManagedPolicyName("service-role/AWSLambdaBasicExecutionRole"),
                        ManagedPolicy.fromAwsManagedPolicyName("CloudWatchLambdaInsightsExecutionRolePolicy"),
                        lambdaPolicy))
                .build();

        var booksDataLambda = Function.Builder.create(this, "zana-books-data-handler")
                .runtime(Runtime.PROVIDED_AL2)
                .description("Function that returns book data and ratings.")
                .code(Code.fromAsset("../../services/zana_lambda/target/lambda/zana_lambda/bootstrap.zip"))
                .handler("main")
                .environment(Map.of(
                        "RUST_BACKTRACE", "1",
                        "ZANA_ENV", zanaEnv,
                        "PARAMETERS_SECRETS_EXTENSION_HTTP_PORT", "2773"))
                .role(lambdaRole)
                .timeout(Duration.seconds(30))
                .insightsVersion(LambdaInsightsVersion.fromInsightVersionArn(lambdaInsightsExtensionArn))
                .logRetention(RetentionDays.TWO_YEARS)
                .layers(List.of(
                        LayerVersion.fromLayerVersionArn(
                                this,
                                "zana-books-lambda-ssm-extension",
                                lambdaSSMExtensionArn)))
                .build();

        var lambdaAlias = booksDataLambda.addAlias(zanaEnv);

        var lambdaAutoScaling = AutoScalingOptions.builder()
                .maxCapacity(20)
                .build();
        var scalingTarget = lambdaAlias.addAutoScaling(lambdaAutoScaling);
        scalingTarget.scaleOnUtilization(UtilizationScalingOptions.builder()
                .utilizationTarget(0.5)
                .build());

        return booksDataLambda;
    }

    private RestApi createZanaRestApi(String zanaEnv, Function booksDataLambda, List<String> corsAllowedOrigins) {
        var restApiLogGroup = new LogGroup(this, "zana-books-api-log-group");

        var restApi = RestApi.Builder.create(this, "zana-books-api")
                .cloudWatchRole(false)
                .deployOptions(StageOptions.builder()
                        .stageName("")
                        .cachingEnabled(false)
                        .metricsEnabled(true)
                        .loggingLevel(MethodLoggingLevel.INFO)
                        .accessLogDestination(new LogGroupLogDestination(restApiLogGroup))
                        .accessLogFormat(AccessLogFormat.jsonWithStandardFields())
                        .throttlingRateLimit(1000)
                        .throttlingBurstLimit(500)
                        .build())
                .defaultCorsPreflightOptions(CorsOptions.builder()
                        .allowHeaders(Cors.DEFAULT_HEADERS)
                        .allowMethods(List.of("GET"))
                        .allowOrigins(corsAllowedOrigins)
                        .build())
                .endpointConfiguration(EndpointConfiguration.builder()
                        .types(List.of(EndpointType.REGIONAL))
                        .build())
                .build();

        var booksDataLambdaIntegration = LambdaIntegration.Builder.create(booksDataLambda)
                .allowTestInvoke(true)
                .timeout(Duration.seconds(29))
                .proxy(true)
                .build();

        var booksResource = restApi.getRoot().addResource("books");
        booksResource.addMethod("GET", booksDataLambdaIntegration);

        // Create account for API Gateway CloudWatch logs
        var apiGatewayCloudWatchRole = Role.Builder.create(this, "zana-api-gateway-cloudwatch-role")
                .assumedBy(new ServicePrincipal("apigateway.amazonaws.com"))
                .description("Allows API Gateways to push logs into CloudWatch.")
                .managedPolicies(List.of(
                        ManagedPolicy.fromAwsManagedPolicyName("service-role/AmazonAPIGatewayPushToCloudWatchLogs")))
                .build();

        CfnAccount.Builder.create(this, "zana-api-gateway-account")
                .cloudWatchRoleArn(apiGatewayCloudWatchRole.getRoleArn())
                .build();

        return restApi;
    }
}
