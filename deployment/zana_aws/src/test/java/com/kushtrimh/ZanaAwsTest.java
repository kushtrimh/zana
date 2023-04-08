package com.kushtrimh;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import software.amazon.awscdk.App;
import software.amazon.awscdk.assertions.Capture;
import software.amazon.awscdk.assertions.Match;
import software.amazon.awscdk.assertions.Template;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * @author Kushtrim Hajrizi
 */
public class ZanaAwsTest {

    private static final String ENV = "test";

    private Template template;

    @BeforeEach
    public void init() {
        var app = new App();
        var stack = new ZanaAwsStack(app, ENV);
        template = Template.fromStack(stack);
    }

    @Test
    public void zanaLambda_LambdaIsConfiguredProperly() {
        template.hasResourceProperties("AWS::Lambda::Function", getZanaLambdaBaseProps());
    }

    @Test
    public void zanaLambda_LambdaHasRoleThatCanAccessRequiredServices() {
        var roleCapture = new Capture();

        var resource = template.findResources("AWS::IAM::ManagedPolicy", Map.of(
                "Properties", Map.of(
                        "PolicyDocument", Map.of(
                                "Statement", List.of(
                                        Map.of(
                                                "Action", "ssm:GetParameter",
                                                "Effect", "Allow",
                                                "Resource", "arn:aws:ssm:*:*:parameter/zana/*"),
                                        Map.of(
                                                "Action", "kms:Decrypt",
                                                "Effect", "Allow",
                                                "Resource", "*"))))));

        var role = template.findResources("AWS::IAM::Role", Map.of(
                "Properties", Match.objectLike(Map.of(
                        "AssumeRolePolicyDocument", Map.of(
                                "Statement", List.of(Map.of(
                                        "Action", "sts:AssumeRole",
                                        "Effect", "Allow",
                                        "Principal", Map.of(
                                                "Service", "lambda.amazonaws.com"))),
                                "Version", "2012-10-17"),
                        "Description", Match.anyValue(),
                        "ManagedPolicyArns", List.of(
                                Map.of("Fn::Join", List.of(
                                        Match.anyValue(),
                                        List.of(
                                                Match.anyValue(),
                                                Match.anyValue(),
                                                ":iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"))),
                                Map.of("Fn::Join", List.of(
                                        Match.anyValue(),
                                        List.of(
                                                Match.anyValue(),
                                                Match.anyValue(),
                                                ":iam::aws:policy/CloudWatchLambdaInsightsExecutionRolePolicy"))),
                                Map.of("Ref", resource.keySet().iterator().next()))))));

        template.hasResourceProperties("AWS::Lambda::Function", merge(getZanaLambdaBaseProps(), Map.of(
                "Role", Map.of(
                        "Fn::GetAtt", List.of(roleCapture, "Arn"))
        )));

        assertEquals(role.keySet().iterator().next(), roleCapture.asString());
    }

    @Test
    public void zanaLambda_LambdaExtensionsAreConfiguredProperly() {
        var extensions = new Capture();

        template.hasResourceProperties("AWS::Lambda::Function", merge(getZanaLambdaBaseProps(), Map.of(
                "Layers", extensions
        )));

        var extensionsList = extensions.asArray();
        var parameters = new ArrayList<>();
        for (var extension : extensionsList) {
            var parameterId = ((Map<String, Object>) extension).get("Ref").toString();
            var parameter = template.findParameters(parameterId);
            parameters.add(parameter.get(parameterId).get("Default"));
        }

        assertEquals(2, parameters.size());
        assertTrue(parameters.contains("/zana/test/lambda-ssm-extension-arn"));
        assertTrue(parameters.contains("/zana/test/lambda-insights-extension-arn"));
    }

    @Test
    public void zanaLambda_LambdaLogRetentionIsConfiguredProperly() {
        var logRetention = new Capture();

        String lambdaId = getZanaLambdaId();
        template.hasResourceProperties("Custom::LogRetention", Map.of(
                "LogGroupName", Map.of(
                        "Fn::Join", List.of(
                                "",
                                List.of(
                                        "/aws/lambda/",
                                        Map.of("Ref", lambdaId)))),
                "RetentionInDays", logRetention));

        assertEquals(731, logRetention.asNumber().intValue());
    }

    @Test
    public void zanaLambda_LambdaAliasIsConfiguredProperly() {
        template.hasResourceProperties("AWS::Lambda::Alias", getZanaLambdaAliasBaseProps());
    }

    @Test
    public void zanaLambda_LambdaAutoScalingIsConfiguredProperly() {
        var lambdaAlias = template.findResources("AWS::Lambda::Alias", Map.of(
                "Properties", Match.objectLike(getZanaLambdaAliasBaseProps())));
        var aliasId = lambdaAlias.keySet().iterator().next();

        var scalingTarget = template.findResources("AWS::ApplicationAutoScaling::ScalableTarget", Map.of(
                "Properties", Map.of(
                        "MaxCapacity", 20,
                        "MinCapacity", 1,
                        "ResourceId", Match.objectLike(Map.of(
                                "Fn::Join", List.of(
                                        "",
                                        Match.arrayWith(List.of(
                                                "function:",
                                                Map.of("Fn::Select", Match.arrayWith(List.of(
                                                        Map.of("Fn::Split", List.of(
                                                                ":",
                                                                Map.of("Ref", aliasId))))
                                                ))))))),
                        "RoleARN", Match.anyValue(),
                        "ScalableDimension", "lambda:function:ProvisionedConcurrency",
                        "ServiceNamespace", "lambda")));
        var scalingTargetId = scalingTarget.keySet().iterator().next();

        template.hasResourceProperties("AWS::ApplicationAutoScaling::ScalingPolicy", Map.of(
                "PolicyName", Match.anyValue(),
                "PolicyType", "TargetTrackingScaling",
                "ScalingTargetId", Map.of(
                        "Ref", scalingTargetId),
                "TargetTrackingScalingPolicyConfiguration", Map.of(
                        "PredefinedMetricSpecification", Map.of(
                                "PredefinedMetricType", "LambdaProvisionedConcurrencyUtilization"),
                        "TargetValue", 0.5)
        ));
    }

    @Test
    public void restApi_LogGroupIsConfiguredProperly() {
        template.hasResourceProperties("AWS::Logs::LogGroup", Map.of(
                "RetentionInDays", 731));
    }

    @Test
    public void restApi_RestApiGatewayIsConfiguredProperly() {
        template.hasResourceProperties("AWS::ApiGateway::RestApi", getRestApiGatewayBaseProps());
    }

    @Test
    public void restApi_RestApiGatewayDeploymentIsConfiguredProperly() {
        template.hasResourceProperties("AWS::ApiGateway::Deployment", Map.of(
                "RestApiId", Map.of(
                        "Ref", getRestApiGatewayId())));
    }

    @Test
    public void restApi_RestApiGatewayStageIsConfiguredProperly() {
        String restApiGatewayId = getRestApiGatewayId();
        var deployment = template.findResources("AWS::ApiGateway::Deployment", Map.of(
                "Properties", Map.of(
                        "RestApiId", Map.of(
                                "Ref", restApiGatewayId))));
        var deploymentId = deployment.keySet().iterator().next();

        var logGroup = template.findResources("AWS::Logs::LogGroup", Map.of(
                "Properties", Map.of(
                        "RetentionInDays", 731)));
        var logGroupId = logGroup.keySet().iterator().next();

        template.hasResourceProperties("AWS::ApiGateway::Stage", Map.of(
                "RestApiId", Map.of(
                        "Ref", restApiGatewayId),
                "AccessLogSetting", Map.of(
                        "DestinationArn", Map.of(
                                "Fn::GetAtt", List.of(logGroupId, "Arn")),
                        "Format", Match.anyValue()),
                "DeploymentId", Map.of(
                        "Ref", deploymentId),
                "MethodSettings", List.of(
                        Map.of(
                                "CachingEnabled", false,
                                "DataTraceEnabled", false,
                                "HttpMethod", "*",
                                "LoggingLevel", "INFO",
                                "MetricsEnabled", true,
                                "ResourcePath", "/*",
                                "ThrottlingBurstLimit", 500,
                                "ThrottlingRateLimit", 1000
                        )),
                "StageName", "prod"));
    }

    @Test
    public void restApi_RestApiGatewayResourceIsConfiguredProperly() {
        template.hasResourceProperties("AWS::ApiGateway::Resource", getRestApiGatewayResourceBaseProps());
    }

    @Test
    public void restApi_RestApiGatewayMethodIsConfiguredProperly() {
        var restApiGatewayId = getRestApiGatewayId();

        var restApiResource = template.findResources("AWS::ApiGateway::Resource", Map.of(
                "Properties", Match.objectLike(getRestApiGatewayResourceBaseProps())));
        var restApiResourceId = restApiResource.keySet().iterator().next();

        template.hasResourceProperties("AWS::ApiGateway::Method", Map.of(
                "HttpMethod", "GET",
                "ResourceId", Map.of(
                        "Ref", restApiResourceId),
                "RestApiId", Map.of(
                        "Ref", restApiGatewayId),
                "AuthorizationType", "NONE",
                "Integration", Map.of(
                        "IntegrationHttpMethod", "POST",
                        "TimeoutInMillis", 29000,
                        "Type", "AWS_PROXY",
                        "Uri", Match.anyValue())));
    }

    @Test
    public void restApi_RestApiGatewayCloudWatchRoleIsConfiguredProperly() {
        template.hasResourceProperties("AWS::IAM::Role", getRestApiGatewayCloudWatchRoleBaseProps());
    }

    @Test
    public void restApi_RestApiGatewayAccountIsConfiguredProperly() {
        var restApiGatewayCloudWatchRole = template.findResources("AWS::IAM::Role", Map.of(
                "Properties", Match.objectLike(getRestApiGatewayCloudWatchRoleBaseProps())));
        var restApiGatewayCloudWatchRoleId = restApiGatewayCloudWatchRole.keySet().iterator().next();

        template.hasResourceProperties("AWS::ApiGateway::Account", Map.of(
                "CloudWatchRoleArn", Map.of(
                        "Fn::GetAtt", List.of(restApiGatewayCloudWatchRoleId, "Arn"))));
    }

    @Test
    public void cloudFrontDistribution_CachePolicyIsConfiguredProperly() {
        template.hasResourceProperties("AWS::CloudFront::CachePolicy", getCloudFrontDistributionCachePolicyBaseProps());
    }

    @Test
    public void cloudFrontDistribution_DistributionIsConfiguredProperly() {
        var aliasCapture = new Capture();
        var certificateCapture = new Capture();

        var cachePolicy = template.findResources("AWS::CloudFront::CachePolicy", Map.of(
                "Properties", Match.objectLike(getCloudFrontDistributionCachePolicyBaseProps())));
        var cachePolicyId = cachePolicy.keySet().iterator().next();

        template.hasResourceProperties("AWS::CloudFront::Distribution", Map.of(
                        "DistributionConfig", Map.of(
                                "Aliases", List.of(Map.of("Ref", aliasCapture)),
                                "DefaultCacheBehavior", Map.of(
                                        "CachePolicyId", Map.of(
                                                "Ref", cachePolicyId),
                                        "Compress", true,
                                        "TargetOriginId", Match.anyValue(),
                                        "ViewerProtocolPolicy", "allow-all"),
                                "Enabled", true,
                                "HttpVersion", "http2",
                                "IPV6Enabled", true,
                                "Logging", Match.objectLike(Map.of(
                                        "Bucket", Match.anyValue(),
                                        "Prefix", "zana-distribution-access-logs/")),
                                "Origins", List.of(
                                        Map.of(
                                                "CustomOriginConfig", Map.of(
                                                        "OriginProtocolPolicy", "https-only",
                                                        "OriginSSLProtocols", List.of("TLSv1.2")),
                                                "DomainName", Match.anyValue(),
                                                "Id", Match.anyValue(),
                                                "OriginPath", "/prod"
                                        )
                                ),
                                "ViewerCertificate", Map.of(
                                        "AcmCertificateArn", Map.of(
                                                "Ref", certificateCapture),
                                        "MinimumProtocolVersion", "TLSv1.2_2021",
                                        "SslSupportMethod", "sni-only"
                                )
                        )
                )
        );

        var aliasParameterId = aliasCapture.asString();
        var aliasParameter = template.findParameters(aliasParameterId);

        var certificateParameterId = certificateCapture.asString();
        var certificateParameter = template.findParameters(certificateParameterId);

        assertEquals("/zana/test/api-host", aliasParameter.get(aliasParameterId).get("Default"));
        assertEquals("/zana/test/certificate-arn", certificateParameter.get(certificateParameterId).get("Default"));
    }

    @Test
    public void hostZoneARecord_RecordSetIsConfiguredProperly() {
        var hostZoneNameCapture = new Capture();
        var distributionIdCapture = new Capture();
        var hostedZoneIdCapture = new Capture();

        template.hasResourceProperties("AWS::Route53::RecordSet", Map.of(
                "Name", Map.of(
                        "Fn::Join", List.of(
                                "",
                                Match.arrayWith(List.of(
                                        Map.of("Ref", hostZoneNameCapture)
                                ))
                        )
                ),
                "Type", "A",
                "AliasTarget", Map.of(
                        "DNSName", Map.of(
                                "Fn::GetAtt", List.of(
                                        distributionIdCapture,
                                        "DomainName")),
                        "HostedZoneId", Match.anyValue()),
                "HostedZoneId", Map.of(
                        "Ref", hostedZoneIdCapture))
        );

        var hostZoneNameParameterId = hostZoneNameCapture.asString();
        var hostZoneNameParameter = template.findParameters(hostZoneNameParameterId);

        var hostedZoneIdParameterId = hostedZoneIdCapture.asString();
        var hostedZoneIdParameter = template.findParameters(hostedZoneIdParameterId);
        assertEquals("/zana/test/hosted-zone-name", hostZoneNameParameter.get(hostZoneNameParameterId).get("Default"));
        assertEquals("/zana/test/hosted-zone-id", hostedZoneIdParameter.get(hostedZoneIdParameterId).get("Default"));

        var distributionId = distributionIdCapture.asString();
        assertTrue(template.findResources("AWS::CloudFront::Distribution").containsKey(distributionId));
    }

    // Helpers

    private String getZanaLambdaId() {
        var lambda = template.findResources("AWS::Lambda::Function", Map.of(
                "Properties", getZanaLambdaBaseProps()));
        return lambda.keySet().iterator().next();
    }

    private Map<String, Object> getZanaLambdaBaseProps() {
        return Map.of(
                "Code", Match.anyValue(),
                "Environment", Map.of(
                        "Variables", Map.of(
                                "PARAMETERS_SECRETS_EXTENSION_HTTP_PORT", "2773",
                                "RUST_BACKTRACE", "1",
                                "ZANA_ENV", ENV)),
                "Handler", "main",
                "Runtime", "provided.al2");
    }

    private Map<String, Object> getZanaLambdaAliasBaseProps() {
        return Map.of(
                "FunctionName", Map.of(
                        "Ref", getZanaLambdaId()),
                "FunctionVersion", Map.of(
                        "Fn::GetAtt", List.of(Match.anyValue(), "Version")),
                "Name", ENV
        );
    }

    private String getRestApiGatewayId() {
        var restApiGateway = template.findResources("AWS::ApiGateway::RestApi", Map.of(
                "Properties", getRestApiGatewayBaseProps()));
        return restApiGateway.keySet().iterator().next();
    }

    private static Map<String, Object> getRestApiGatewayBaseProps() {
        return Map.of(
                "EndpointConfiguration", Map.of(
                        "Types", List.of("REGIONAL")),
                "Name", Match.anyValue());
    }

    private Map<String, Object> getRestApiGatewayResourceBaseProps() {
        String restApiGatewayId = getRestApiGatewayId();
        return Map.of(
                "ParentId", Map.of(
                        "Fn::GetAtt", List.of(restApiGatewayId, "RootResourceId")),
                "PathPart", "books",
                "RestApiId", Map.of(
                        "Ref", restApiGatewayId));
    }

    private Map<String, Object> getRestApiGatewayCloudWatchRoleBaseProps() {
        return Map.of(
                "AssumeRolePolicyDocument", Map.of(
                        "Statement", List.of(
                                Map.of(
                                        "Action", "sts:AssumeRole",
                                        "Effect", "Allow",
                                        "Principal", Map.of(
                                                "Service", "apigateway.amazonaws.com")))),
                "ManagedPolicyArns", List.of(
                        Match.objectLike(Map.of(
                                "Fn::Join", Match.arrayWith(List.of(
                                        Match.arrayWith(List.of(
                                                ":iam::aws:policy/service-role/AmazonAPIGatewayPushToCloudWatchLogs"
                                        ))
                                ))
                        ))));
    }

    private Map<String, Map<String, Object>> getCloudFrontDistributionCachePolicyBaseProps() {
        return Map.of(
                "CachePolicyConfig", Map.of(
                        "Comment", Match.anyValue(),
                        "DefaultTTL", 21600,
                        "MaxTTL", 43200,
                        "MinTTL", 0,
                        "Name", Match.anyValue(),
                        "ParametersInCacheKeyAndForwardedToOrigin", Map.of(
                                "CookiesConfig", Map.of(
                                        "CookieBehavior", "none"),
                                "EnableAcceptEncodingBrotli", false,
                                "EnableAcceptEncodingGzip", true,
                                "HeadersConfig", Map.of(
                                        "HeaderBehavior", "none"),
                                "QueryStringsConfig", Map.of(
                                        "QueryStringBehavior", "all"))));
    }

    private Map<String, Object> merge(Map<String, Object> props1, Map<String, Object> props2) {
        var merged = new HashMap<>(props1);
        merged.putAll(props2);
        return merged;
    }

}
