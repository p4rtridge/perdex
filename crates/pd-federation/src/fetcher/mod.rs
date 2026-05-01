use std::str::FromStr;

use error_stack::{Report, ResultExt};
use http::{
    Method, Request, StatusCode,
    header::{ACCEPT, CONTENT_TYPE},
};
use mime::Mime;
use pd_core::account::traits::AccountResolver;
use pd_http::{Body, Client};
use serde::de::DeserializeOwned;
use thiserror::Error;
use typed_builder::TypedBuilder;

use crate::ap_type::jsonld::{self, RdfNode};

pub mod actor;

pub const ACCEPT_ACTIVITY_VALUE: &str = "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\", application/activity+json";

#[derive(Debug, Error)]
pub enum FetcherError {
    #[error("HTTP error: {0}")]
    HttpError(&'static str),
    #[error("Parse error: {0}")]
    ParseError(String),
}

#[derive(Clone, TypedBuilder)]
pub struct Fetcher<R>
where
    R: AccountResolver,
{
    http_client: Client,
    resolver: R,
}

impl<R> Fetcher<R>
where
    R: AccountResolver,
{
    /// Fetches an ActivityPub resource from the given URL and attempts to parse it as type `T`.
    ///
    /// The authority of '@id' must match the server authority to prevent SSRF attacks.
    async fn fetch_ap_resource<T>(&self, url: &str) -> Result<Option<T>, Report<FetcherError>>
    where
        T: DeserializeOwned + RdfNode,
    {
        let request = Request::builder()
            .method(Method::GET)
            .header(ACCEPT, ACCEPT_ACTIVITY_VALUE)
            .uri(url)
            .body(Body::empty())
            .expect("Failed to build fetch account request");

        let response = self
            .http_client
            .execute(request)
            .await
            .change_context(FetcherError::HttpError("Failed to fetch resource"))?;

        if matches!(response.status(), StatusCode::NOT_FOUND | StatusCode::GONE) {
            return Ok(None);
        }

        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .ok_or(Report::new(FetcherError::HttpError(
                "Missing Content-Type header",
            )))?
            .to_str()
            .change_context(FetcherError::HttpError(
                "Failed to get Content-Type header string",
            ))?;
        let content_type = Mime::from_str(content_type).change_context(FetcherError::HttpError(
            "Failed to parse Content-Type header",
        ))?;

        let is_activity_json = content_type
            .essence_str()
            .eq_ignore_ascii_case("application/activity+json");

        let is_json_ld_activitystreams = content_type
            .essence_str()
            .eq_ignore_ascii_case("application/ld+json")
            && content_type
                .get_param("profile")
                .is_some_and(|profile_urls| {
                    profile_urls
                        .as_str()
                        .split_whitespace()
                        .any(|url| url == "https://www.w3.org/ns/activitystreams")
                });

        if !is_activity_json && !is_json_ld_activitystreams {
            return Err(Report::new(FetcherError::HttpError(
                "Invalid Content-Type in response",
            )));
        }

        // Validate that the authority of the response URI matches the server authority to prevent SSRF attacks
        let server_authority = response
            .authority()
            .ok_or(Report::new(FetcherError::HttpError(
                "Missing authority in request URI",
            )))?
            .to_owned();

        let actor = response
            .json::<T>()
            .await
            .change_context(FetcherError::HttpError("Failed to parse Activitypub Actor"))?;

        jsonld::validate_rdf_node(&actor, &server_authority)
            .change_context(FetcherError::HttpError("Invalid RDF node in Actor"))?;

        Ok(Some(actor))
    }
}
