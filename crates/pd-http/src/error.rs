use error_stack::Report;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Errors related to building the HTTP client or request
    #[error("Failed to build HTTP client or request")]
    RequestBuild,

    /// Errors related to executing the HTTP request or receiving the response
    #[error("Failed to execute HTTP request")]
    RequestExecution,

    /// Errors related to reading the HTTP response body
    #[error("Failed to read HTTP response body")]
    ResponseRead,

    /// Errors related to exceeding the configured body size limit
    #[error("Response body exceeds the configured limit of {0} bytes")]
    BodyLimitExceeded(usize),

    /// A catch-all error variant for any other unexpected errors
    #[error("An unexpected error occurred: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Report<Error>>;
