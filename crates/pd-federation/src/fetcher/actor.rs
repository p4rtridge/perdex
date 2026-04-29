use error_stack::{Report, ResultExt};

use pd_core::account::{
    model::{AccountFetchError, RemoteAccountProfile},
    traits::{AccountFetcher, AccountResolver},
};
use url::Url;

use crate::{ap_type::actor::Actor, fetcher::Fetcher};

impl<R> AccountFetcher for Fetcher<R>
where
    R: AccountResolver,
{
    async fn fetch_account(
        &self,
        url: &str,
        acct: Option<(&str, &str)>,
    ) -> Result<Option<RemoteAccountProfile>, Report<AccountFetchError>> {
        let mut actor_url = Url::parse(url)
            .change_context(AccountFetchError::FetchError("Failed to parse url input"))?;

        let Some(mut actor) = self
            .fetch_ap_resource::<Actor>(actor_url.as_str())
            .await
            .change_context(AccountFetchError::FetchError(
                "Failed to fetch actor resource",
            ))?
        else {
            return Ok(None);
        };

        let mut domain = actor_url.host_str().ok_or(AccountFetchError::FetchError(
            "Missing host component in url input",
        ))?;
        let try_resolver = acct.is_none_or(|acct| acct != (&actor.preferred_username, domain));

        let domain_buf;
        let used_resolver = if try_resolver {
            let resolved_account = self
                .resolver
                .resolve_account(&actor.preferred_username, domain)
                .await
                .change_context(AccountFetchError::FetchError("Failed to resolve account"))?;

            match resolved_account {
                Some(resource) => {
                    if resource.uri == actor.id {
                        actor.preferred_username = resource.username;
                        domain_buf = resource.domain;
                        domain = &domain_buf;
                        true
                    } else {
                        return Err(Report::new(AccountFetchError::FetchError(
                            "Resolved account URI does not match Actor ID",
                        )));
                    }
                }
                _ => {
                    // Fall back to `{preferredUsername}@{domain}`
                    false
                }
            }
        } else {
            false
        };

        // TODO: add comment about why we need to re-parse the Actor ID as a URL if we used the resolver
        if !used_resolver && actor.id != actor_url.as_str() {
            actor_url = Url::parse(&actor.id).change_context(AccountFetchError::FetchError(
                "Failed to parse Actor ID as URL",
            ))?;
            domain = actor_url.host_str().ok_or(AccountFetchError::FetchError(
                "Missing host component in Actor ID",
            ))?;
        }

        let remote_account = RemoteAccountProfile {
            uri: actor.id,
            domain: domain.to_string(),
            username: actor.preferred_username,
            display_name: actor.name,
            public_key: actor.public_key.public_key_pem, // TODO: handle missing public key or unsupported formats
            avatar_url: None, // TODO: extract from Actor's `icon` property
            summary: None,    // TODO: extract from Actor's `summary` property
        };
        Ok(Some(remote_account))
    }
}
