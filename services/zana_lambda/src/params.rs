use crate::http::{FailureResponse, ResponseError};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

// TODO: Test with httpmock, no other way of testing it as of right now

const AWS_TOKEN_HEADER: &str = "X-Aws-Parameters-Secrets-Token";

#[derive(Deserialize)]
struct ParameterResponse {
    #[serde(rename(deserialize = "Parameter"))]
    parameter: Parameter,
}

#[derive(Deserialize)]
struct Parameter {
    #[serde(rename(deserialize = "Value"))]
    value: String,
}

#[async_trait]
pub trait ParamStore {
    async fn parameter(&self, name: &str, with_decryption: bool)
        -> Result<String, FailureResponse>;
}

#[derive(Debug)]
pub struct AWSParamStore {
    param_store_get_url: String,
    client: Client,
    token: String,
    env: String,
}

impl AWSParamStore {
    pub fn new(param_store_url: &str, token: &str, zana_env: &str) -> Self {
        Self {
            param_store_get_url: String::from(param_store_url),
            client: Client::new(),
            token: String::from(token),
            env: String::from(zana_env),
        }
    }
}

#[async_trait]
impl ParamStore for AWSParamStore {
    async fn parameter(
        &self,
        name: &str,
        with_decryption: bool,
    ) -> Result<String, FailureResponse> {
        let with_decryption = with_decryption.to_string();
        let query_params: Vec<(&str, &str)> = vec![
            ("name", name),
            ("label", &self.env),
            ("withDecryption", with_decryption.as_str()),
        ];

        let response = self
            .client
            .get(&self.param_store_get_url)
            .header(AWS_TOKEN_HEADER, &self.token)
            .query(&query_params)
            .send()
            .await
            .map_err(|err| {
                log::error!("could not send request to parameter store {}", err);
                FailureResponse::new(&ResponseError::ServiceError)
            })?;

        let response_status = response.status().as_u16();
        if response_status != 200 {
            log::error!(
                "received response with {} from parameter store: {}",
                response_status,
                response.text().await.map_err(|err| {
                    log::error!("could not retrieve text from parameter response {}", err);
                    FailureResponse::new(&ResponseError::ServiceError)
                })?
            );
            Err(FailureResponse::new(&ResponseError::ServiceError))
        } else {
            let parameter_response: ParameterResponse = response.json().await.map_err(|err| {
                log::error!("could not convert parameter response from json {}", err);
                FailureResponse::new(&ResponseError::ServiceError)
            })?;
            return Ok(parameter_response.parameter.value);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::http::ResponseError;
    use crate::params::{AWSParamStore, ParamStore, AWS_TOKEN_HEADER};
    use httpmock::prelude::*;
    use httpmock::Mock;
    use httpmock::MockServer;

    const TEST_ENV: &str = "test";
    const TOKEN: &str = "token-1234";

    fn create_mock<'a>(
        server: &'a MockServer,
        name: &str,
        with_decryption: bool,
        status_code: u16,
        response_body: &str,
    ) -> Mock<'a> {
        server.mock(|when, then| {
            when.method(GET)
                .query_param("name", name)
                .query_param("label", TEST_ENV)
                .query_param("withDecryption", with_decryption.to_string().as_str())
                .header(AWS_TOKEN_HEADER, TOKEN);
            then.status(status_code)
                .header("Content-Type", "application/json")
                .body(response_body);
        })
    }

    fn create_param_store(server: &MockServer) -> AWSParamStore {
        AWSParamStore::new(
            &format!("http://{}", &server.address().to_string()),
            TOKEN,
            TEST_ENV,
        )
    }

    async fn assert_fail(param_name: &str, decrypt: bool, status_code: u16, response_body: &str) {
        let server = MockServer::start();
        let m = create_mock(&server, param_name, decrypt, status_code, &response_body);
        let param_store = create_param_store(&server);
        let response = param_store.parameter(param_name, decrypt).await;

        m.assert();
        match response {
            Ok(_) => panic!("successful response returned when error expected"),
            Err(err) => assert_eq!(ResponseError::ServiceError.to_string(), err.error),
        }
    }

    #[tokio::test]
    async fn return_error_when_request_cant_complete() {
        let param_store = AWSParamStore::new("http://localhost/wrong/url/here", TOKEN, TEST_ENV);
        let response = param_store.parameter("test/param-name", false).await;
        match response {
            Ok(_) => panic!("successful response returned when error expected"),
            Err(err) => assert_eq!(ResponseError::ServiceError.to_string(), err.error),
        }
    }

    #[tokio::test]
    async fn return_error_when_response_is_not_200() {
        let param_name = "test/param-name";
        let decrypt = false;

        for status_code in [201, 300, 400, 401, 403, 404, 429, 500, 503] {
            assert_fail(param_name, decrypt, status_code, "Error returned").await
        }
    }

    #[tokio::test]
    async fn return_error_when_response_is_not_json() {
        let param_name = "test/param-name";
        let decrypt = false;

        assert_fail(param_name, decrypt, 200, "This is not JSON").await
    }

    #[tokio::test]
    async fn get_parameter_from_param_store() {
        let param_name = "test/param-name";
        let decrypt = false;

        let response_body = "{\"Parameter\":{\"ARN\":\"arn:aws:ssm:us-east-2:111122223333:parameter/test/param-name\",\"DataType\":\"text\",\"LastModifiedDate\":1582657288.8,\"Name\":\"test/param-name\",\"Type\":\"SecureString\",\"Value\":\"param-value\",\"Version\":3}}";

        let server = MockServer::start();
        let m = create_mock(&server, param_name, decrypt, 200, &response_body);
        let param_store = create_param_store(&server);
        let response = param_store
            .parameter(param_name, decrypt)
            .await
            .expect("value expected from parameter store");

        m.assert();
        assert_eq!("param-value", response);
    }
}
