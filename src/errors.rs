use http::Error as HttpError;
use hyper::{Error as HyperError, StatusCode};
use serde_json::Error as JsonError;
use thiserror::Error as ThisError;
use tokio::time::Elapsed;
use url::ParseError as UrlParseError;

fn strs_to_str(strs: &Vec<&'static str>) -> String {
    strs.join(", ")
}

/// High-level error for all operations.
#[derive(ThisError, Debug)]
pub enum Error {
    /// The Consul endpoint given to configure a client was invalid.
    #[error("failed to parse Consul endpoint: {0:?}")]
    InvalidConsulEndpoint(#[from] UrlParseError),
    /// JSON serialization error during building a request.
    #[error("failed to serialize request body to JSON: {0:?}")]
    InvalidRequestBody(JsonError),
    /// Unable to construct a valid HTTP request for an operation.
    #[error("failed to build request: {0:?}")]
    InvalidRequest(HttpError),
    /// Error occurred during the sending of a request to Consul.
    #[error("request error: {0}")]
    RequestError(#[from] HyperError),
    /// Request timed out.
    #[error("request timed out: {0}")]
    RequestTimedOut(#[from] Elapsed),
    /// Error occurred while parsing a response from Consul.
    #[error("unexpected response: {0}")]
    ResponseError(#[from] ResponseError),
}

/// High-level error for responses.
#[derive(ThisError, Debug)]
pub enum ResponseError {
    /// The HTTP status code for the response was unexpected.
    #[error("unexpected status code: {0}")]
    UnexpectedStatus(StatusCode),
    /// The response from Consul was missing expected headers or they were invalid.
    #[error("missing or invalid response headers: {}", strs_to_str(.0))]
    InvalidHeaders(Vec<&'static str>),
    /// Failed to consume/read the entire body of the response.
    #[error("failed to consume response: {0}")]
    BodyConsumeFailure(#[from] HyperError),
    /// The response body was not JSON or did not match the expected JSON structure.
    #[error("invalid JSON payload: {0}")]
    InvalidPayload(#[from] JsonError),
}
