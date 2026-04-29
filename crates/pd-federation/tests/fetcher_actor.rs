use std::convert::Infallible;

use bytes::Bytes;
use http::{Request, Response, StatusCode, header::CONTENT_TYPE};
use http_body_util::Full;
use pd_core::account::traits::AccountFetcher;
use pd_federation::{fetcher::Fetcher, resolver::webfinger::Webfinger};
use pd_http::Client;

#[tokio::test]
async fn basic() {
    let svc = tower::service_fn(|request: Request<_>| async move {
        match request.uri().path_and_query().unwrap().as_str() {
            "/users/partridge" => {
                let body = include_str!("../../../test-fixtures/ap/partridge_actor.json");

                Ok::<_, Infallible>(
                    Response::builder()
                        .header(CONTENT_TYPE, "application/activity+json")
                        .body(Full::<Bytes>::new(body.into()))
                        .unwrap(),
                )
            }
            path if path.starts_with("/.well-known/webfinger?") => Ok::<_, Infallible>(
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Full::default())
                    .unwrap(),
            ),
            path => panic!("HTTP client hit unexpected route: {path}"),
        }
    });

    let http_client = Client::builder().service(svc);
    let webfinger = Webfinger::builder()
        .http_client(http_client.clone())
        .build();

    let fetcher = Fetcher::builder()
        .http_client(http_client)
        .resolver(webfinger)
        .build();

    let actor = fetcher
        .fetch_account("https://mastodon.com/users/partridge", None)
        .await
        .expect("Failed to fetch actor")
        .unwrap();

    assert_eq!(actor.uri, "https://mastodon.com/users/partridge");
    assert_eq!(actor.username, "partridge");
    assert_eq!(actor.domain, "mastodon.com");
}
