use crate::http::{FailureResponse, ResponseError};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub const AWS_TOKEN_HEADER: &str = "X-Aws-Parameters-Secrets-Token";

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
