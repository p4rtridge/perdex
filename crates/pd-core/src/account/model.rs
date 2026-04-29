use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AccountResolutionError {
    /// The account was not found
    #[error("Account not found")]
    NotFound,

    /// An error occurred during resolution
    #[error("Failed to resolve account: {0}")]
    ResolutionError(&'static str),
}

/// Description of a resolved account
#[derive(Debug, Deserialize, Serialize)]
pub struct AccountResource {
    /// The `self` link (the account's URI)
    pub uri: String,
    /// The username part of the canonical `acct:` URI
    pub username: String,
    /// The domain part of the canonical `acct:` URI
    pub domain: String,
}

#[derive(Debug, Error)]
pub enum AccountFetchError {
    /// An error occurred during account fetching
    #[error("Failed to fetch remote account: {0}")]
    FetchError(&'static str),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RemoteAccountProfile {
    pub uri: String,
    pub domain: String,
    pub username: String,
    pub display_name: Option<String>,
    pub summary: Option<String>,
    pub avatar_url: Option<String>,
    pub public_key: String,
}
