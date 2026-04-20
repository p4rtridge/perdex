use error_stack::Report;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
pub enum AccountResolutionError {
    /// The account was not found
    #[error("Account not found")]
    NotFound,

    /// An error occurred during resolution
    #[error("Failed to resolve account: {0}")]
    ResolutionError(String),
}

pub trait AccountResolver {
    fn resolve_account(
        &self,
        username: &str,
        domain: &str,
    ) -> impl Future<Output = Result<Option<AccountResource>, Report<AccountResolutionError>>>;
}
