/*!
Provides functionality to query parameters needed to initialize clients.

Provides support for environment variables and AWS Parameter Store to fetch parameters.
AWS Parameter Store is used via the [AWS Parameter and Secrets Lambda extension](https://docs.aws.amazon.com/systems-manager/latest/userguide/ps-integration-lambda-extensions.html).
*/
use crate::http::{FailureResponse, ResponseError};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::env;

/// Name of the header that will contain the value of the secret token used to communicate
/// with AWS Parameter Store.
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

/// A trait that describes implementations of a parameter container from which parameters
/// can be retrieved.
///
/// This trait provides a way to fetch parameter from the parameter store or environment variables, with a flag for
/// decryption support.
/// An failure error response is returned if an error occurs during communication with the parameter store.
#[async_trait]
pub trait ParamStore {
    /// Returns parameter from parameter store.
    ///
    /// If the parameter needs to be decrypted then `with_decryption` should
    /// be set as true. Use false otherwise.
    async fn parameter(&self, name: &str, with_decryption: bool)
        -> Result<String, FailureResponse>;

    /// Returns parameter from environment variables.
    ///
    /// If the parameter needs to be decrypted then `with_decryption` should
    /// be set as true. Use false otherwise.
    async fn parameter_from_env(
        &self,
        env_variable: &str,
        name: &str,
        with_decryption: bool,
    ) -> Result<String, FailureResponse>;
}

/// Parameter store implementation for AWS Parameter Store.
///
/// Supports fetching parameters and secure parameters from AWS Parameter Store based on label and name.
#[derive(Debug)]
pub struct AWSParamStore {
    param_store_get_url: String,
    client: Client,
    token: String,
    env: String,
}

impl AWSParamStore {
    /// Returns new AWS Parameter Store for specific environment
    ///
    /// * `param_store_url` is the url of AWS Parameter Store, as specified on AWS Parameter Store Lambda extension.
    /// * `token` is a required AWS token that is used for communication with AWS Parameter Store.
    /// * `zana_env` will be used as a label to seperate different parameters based on different environments.
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
    /// Fetches parameter from AWS Parameter Store.
    ///
    /// `zana_env` is used as the label when retrieving parameters.
    /// For any during communication error with the parameter store, response deserialization problems,
    /// or responses that are not 200, an error is returned.
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
                tracing::error!("could not send request to parameter store {}", err);
                FailureResponse::new(&ResponseError::ServiceError)
            })?;

        let response_status = response.status().as_u16();
        if response_status != 200 {
            tracing::error!(
                "received response with {} from parameter store: {}",
                response_status,
                response.text().await.map_err(|err| {
                    tracing::error!("could not retrieve text from parameter response {}", err);
                    FailureResponse::new(&ResponseError::ServiceError)
                })?
            );
            Err(FailureResponse::new(&ResponseError::ServiceError))
        } else {
            let parameter_response: ParameterResponse = response.json().await.map_err(|err| {
                tracing::error!("could not convert parameter response from json {}", err);
                FailureResponse::new(&ResponseError::ServiceError)
            })?;
            return Ok(parameter_response.parameter.value);
        }
    }

    /// Tries to fetch parameter from environment variables, and falls back to AWS Parameter Store
    /// if environment variable does not exist.
    ///
    /// `zana_env` is used as the label when retrieving parameters.
    /// For any during communication error with the parameter store, response deserialization problems,
    /// or responses that are not 200, an error is returned.
    async fn parameter_from_env(
        &self,
        env_variable: &str,
        name: &str,
        with_decryption: bool,
    ) -> Result<String, FailureResponse> {
        match env::var(env_variable) {
            Ok(value) => Ok(value),
            Err(_) => {
                tracing::debug!("parameter {} not found as env variable. Retrieving from parameter store with {}/decryption: {}", env_variable, name, with_decryption);
                self.parameter(name, false).await
            }
        }
    }
}
