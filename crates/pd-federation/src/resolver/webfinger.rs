use error_stack::{Report, ResultExt};
use pd_core::account::{
    model::{AccountResolutionError, AccountResource},
    traits::AccountResolver,
};
use pd_http::{
    Body, Client,
    http::{HeaderValue, Method, Request, StatusCode, header::ACCEPT},
};
use typed_builder::TypedBuilder;

use crate::ap_type::webfinger::Resource;

const ACCEPT_JRD_VALUE: HeaderValue = HeaderValue::from_static("application/jrd+json");
const MAX_REDIRECTS: usize = 3;

#[derive(Clone, TypedBuilder)]
pub struct Webfinger {
    http_client: Client,
}

impl AccountResolver for Webfinger {
    async fn resolve_account(
        &self,
        username: &str,
        domain: &str,
    ) -> Result<Option<AccountResource>, Report<AccountResolutionError>> {
        let original_acct = format!("acct:{}@{domain}", urlencoding::encode(username));

        let mut acct_buf: String;
        let mut acct = original_acct.as_str();
        let mut username = username;
        let mut domain = domain;

        let mut remaining_redirects = MAX_REDIRECTS;
        let links = loop {
            let webfinger_uri = format!("https://{domain}/.well-known/webfinger?resource={acct}");

            let request = Request::builder()
                .method(Method::GET)
                .header(ACCEPT, ACCEPT_JRD_VALUE)
                .uri(webfinger_uri)
                .body(Body::default())
                .expect("Failed to build WebFinger request");
            let response = self
                .http_client
                .execute(request)
                .await
                .change_context_lazy(|| {
                    AccountResolutionError::ResolutionError(
                        "Failed to execute WebFinger request".to_string(),
                    )
                })?;
            if matches!(response.status(), StatusCode::NOT_FOUND | StatusCode::GONE) {
                return Ok(None);
            }

            let resource = response.json::<Resource>().await.change_context_lazy(|| {
                AccountResolutionError::ResolutionError(
                    "Failed to parse WebFinger response".to_string(),
                )
            })?;
            if resource.subject == acct {
                break resource.links;
            }

            if remaining_redirects == 0 {
                return Ok(None);
            }

            acct_buf = resource.subject;
            acct = acct_buf.as_str();

            let Some(username_domain) = acct
                .strip_prefix("acct:")
                .and_then(|acct| acct.split_once('@'))
            else {
                return Ok(None);
            };
            (username, domain) = username_domain;

            remaining_redirects -= 1;
        };

        let Some(uri) = links
            .into_iter()
            .find_map(|link| (link.rel == "self").then_some(link.href?))
        else {
            return Ok(None);
        };

        let account = Some(AccountResource {
            uri,
            username: username.to_string(),
            domain: domain.to_string(),
        });
        Ok(account)
    }
}
