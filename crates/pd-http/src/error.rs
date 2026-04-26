use simdutf8::basic::Utf8Error;
use thiserror::Error;
use tower::BoxError;

#[derive(Debug, Error)]
pub enum HttpError {
    /// Errors related to reading the HTTP response body
    #[error("Failed to read response body: {0}")]
    BodyRead(#[source] BoxError),

    /// Errors related to executing the HTTP request
    #[error("Failed to execute HTTP request: {0}")]
    RequestExecution(#[source] BoxError),

    /// Errors related to deserializing the response body as JSON
    #[error("Failed to deserialize response body as JSON: {0}")]
    JsonDeserialization(#[source] sonic_rs::Error),

    /// Errors related to signing the HTTP request using HTTP Signatures
    #[error("Failed to sign HTTP request")]
    Signature(#[source] BoxError),

    /// Errors related to streaming the response body
    #[error("Failed to read response body stream: {0}")]
    StreamRead(#[source] BoxError),

    /// Errors related to decoding the response body as UTF-8 text
    #[error("Failed to decode response body as UTF-8 text: {0}")]
    TextDecoding(#[source] Utf8Error),

    /// Errors related to converting between different header types
    #[error("Failed to convert headers: {0}")]
    HeaderConversion(#[source] BoxError),
}

pub type Result<T> = std::result::Result<T, HttpError>;
