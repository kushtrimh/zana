package com.kushtrimh;

import software.amazon.awscdk.App;
import software.amazon.awscdk.Environment;
import software.amazon.awscdk.StackProps;

import java.util.Map;
import java.util.Optional;

public class ZanaAwsApp {
    public static void main(final String[] args) {
        App app = new App();

        var region = Optional.ofNullable(System.getenv("CDK_DEFAULT_REGION"))
                .orElseThrow(() -> new IllegalArgumentException("AWS region environment variable not provided"));
        var account = Optional.ofNullable(System.getenv("CDK_DEFAULT_ACCOUNT"))
                .orElseThrow(() -> new IllegalArgumentException("AWS account environment variable not provided"));
        var zanaEnv = Optional.ofNullable(System.getenv("ZANA_ENV"))
                .orElse("prod");

        new ZanaAwsStack(app, "ZanaAwsStack", StackProps.builder()
                .env(Environment.builder()
                        .account(account)
                        .region(region)
                        .build())
                .tags(Map.of("zanaEnv", zanaEnv))
                .build());

        app.synth();
    }
}

