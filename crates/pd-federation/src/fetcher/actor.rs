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
    async fn fetch_account(
        &self,
        url: &str,
        acct: Option<(&str, &str)>,
    ) -> Result<Option<RemoteAccountProfile>, Report<AccountFetchError>> {
        let Some(mut actor) = self.fetch_ap_resource::<Actor>(url).await.change_context(
            AccountFetchError::FetchError("Failed to fetch actor resource"),
        )?
        else {
            return Ok(None);
        };

        let canonical_url = Url::parse(&actor.id).change_context(AccountFetchError::FetchError(
            "Failed to parse Actor ID as URL",
        ))?;
        let mut domain = canonical_url
            .host_str()
            .ok_or(AccountFetchError::FetchError(
                "Missing host component in Actor ID",
            ))?;
        let try_resolver = acct.is_none_or(|acct| acct != (&actor.preferred_username, domain));

        let domain_buf;
        if try_resolver {
            let resolved_account = self
                .resolver
                .resolve_account(&actor.preferred_username, domain)
                .await
                .change_context(AccountFetchError::FetchError("Failed to resolve account"))?;

            // If None then we fall back to `{preferredUsername}@{domain}`
            if let Some(resource) = resolved_account {
                if resource.uri != actor.id {
                    return Err(Report::new(AccountFetchError::FetchError(
                        "Resolved account URI does not match Actor ID",
                    )));
                }

                // Canonicalize preferred username and domain
                actor.preferred_username = resource.username;
                domain_buf = resource.domain;
                domain = &domain_buf;
            }
        };

        // if !used_resolver && actor.id != actor_url.as_str() {
        //     actor_url = Url::parse(&actor.id).change_context(AccountFetchError::FetchError(
        //         "Failed to parse Actor ID as URL",
        //     ))?;
        //     domain = actor_url.host_str().ok_or(AccountFetchError::FetchError(
        //         "Missing host component in Actor ID",
        //     ))?;
        // }

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
