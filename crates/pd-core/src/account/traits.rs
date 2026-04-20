use error_stack::Report;

use crate::account::model::{AccountResolutionError, AccountResource};

pub trait AccountResolver {
    fn resolve_account(
        &self,
        username: &str,
        domain: &str,
    ) -> impl Future<Output = Result<Option<AccountResource>, Report<AccountResolutionError>>>;
}
