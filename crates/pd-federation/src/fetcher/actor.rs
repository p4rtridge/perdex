use error_stack::{Report, ResultExt};

use pd_core::{
    PublicKey,
    account::{
        model::{AccountFetchError, RemoteAccountProfile},
        traits::{AccountFetcher, AccountResolver},
    },
};
use url::Url;

use crate::{ap_type::actor::Actor, fetcher::Fetcher, utils::sanitizer::SanitizeExt};

impl<R> AccountFetcher for Fetcher<R>
where
    R: AccountResolver,
{
    /// Fetches an ActivityPub Actor profile from the given URL
    ///
    /// It guarantees the following security checks:
    /// - The authority of the fetched '@id' must match the server authority to prevent SSRF attacks.
    /// - If an `acct` is provided, it must match the fetched `preferredUsername` and domain.
    /// - If the resolver is used, the resolved account's URI must match the Actor ID to prevent impersonation.
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
        // We don't use the resolver if the acct matches the actor's preferredUsername and domain
        // (if acct is provided, it might come from webfinger resolution in previous steps)
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
                        // Canonicalize preferred username and domain
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

        if !used_resolver && actor.id != actor_url.as_str() {
            actor_url = Url::parse(&actor.id).change_context(AccountFetchError::FetchError(
                "Failed to parse Actor ID as URL",
            ))?;
            domain = actor_url.host_str().ok_or(AccountFetchError::FetchError(
                "Missing host component in Actor ID",
            ))?;
        }

        actor.clean_html();

        // Yes, we still haven't handled icon yet
        // TODO: handle icon and avatar (which may be different in some implementations)

        let remote_account = RemoteAccountProfile {
            uri: actor.id,
            domain: domain.to_string(),
            username: actor.preferred_username,
            display_name: actor.name,
            summary: None,
            avatar_url: None,
            public_key: PublicKey {
                id: actor.public_key.id,
                owner: actor.public_key.owner,
                public_key_pem: actor.public_key.public_key_pem,
            },
        };
        Ok(Some(remote_account))
    }
}
