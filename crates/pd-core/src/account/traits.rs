use error_stack::Report;

use crate::account::model::{
    AccountFetchError, AccountResolutionError, AccountResource, RemoteAccountProfile,
};

pub trait AccountResolver {
    fn resolve_account(
        &self,
        username: &str,
        domain: &str,
    ) -> impl Future<Output = Result<Option<AccountResource>, Report<AccountResolutionError>>>;
}

pub trait AccountFetcher {
    fn fetch_account(
        &self,
        url: &str,
        acct: Option<(&str, &str)>,
    ) -> impl Future<Output = Result<Option<RemoteAccountProfile>, Report<AccountFetchError>>>;
}
