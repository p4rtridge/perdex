use std::convert::Infallible;

use bytes::Bytes;
use http::{Request, Response, StatusCode, header::CONTENT_TYPE};
use http_body_util::Full;
use pd_core::account::traits::AccountFetcher;
use pd_federation::{
    ap_type::webfinger::{Link, Resource},
    fetcher::Fetcher,
    resolver::webfinger::Webfinger,
};
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

#[tokio::test]
async fn check_ap_id_authority() {
    let svc = tower::service_fn(|request: Request<_>| async move {
        assert_ne!(request.uri().host(), Some("example.com"));

        match request.uri().path_and_query().unwrap().as_str() {
            "/users/partridge" => {
                let mut body =
                    include_str!("../../../test-fixtures/ap/partridge_actor.json").to_string();
                // Replace the actor ID with a different domain to trigger SSRF protection
                body = body.replace(
                    "https://mastodon.com/users/partridge",
                    "https://example.com/users/partridge",
                );

                Ok::<_, Infallible>(
                    Response::builder()
                        .header(CONTENT_TYPE, "application/activity+json")
                        .body(Full::<Bytes>::new(body.into()))
                        .unwrap(),
                )
            }
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

    // The fetcher should reject the response because the @id domain does not match the requested domain
    let result = fetcher
        .fetch_account("https://mastodon.com/users/partridge", None)
        .await;

    assert!(
        result.is_err(),
        "Expected fetch_account to fail due to authority mismatch"
    );
}

#[tokio::test]
async fn check_ap_content_type() {
    let svc = tower::service_fn(|request: Request<_>| async move {
        match request.uri().path_and_query().unwrap().as_str() {
            "/users/partridge" => {
                let body = include_str!("../../../test-fixtures/ap/partridge_actor.json");

                Ok::<_, Infallible>(
                    Response::builder()
                        // Intentionally missing CONTENT_TYPE or using wrong one
                        .header(CONTENT_TYPE, "text/html")
                        .body(Full::<Bytes>::new(body.into()))
                        .unwrap(),
                )
            }
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

    let result = fetcher
        .fetch_account("https://mastodon.com/users/partridge", None)
        .await;

    assert!(
        result.is_err(),
        "Expected fetch_account to fail due to invalid content type"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("Invalid Content-Type in response") || err_msg.contains("Content-Type")
    );
}

#[tokio::test]
async fn fetch_account_with_custom_acct() {
    let svc = tower::service_fn(|request: Request<_>| async move {
        let jrd_body = include_str!("../../../test-fixtures/ap/partridge_jrd.json");
        let jrd_body = sonic_rs::to_string(&Resource {
            subject: "acct:partridge@example.com".to_string(),
            ..sonic_rs::from_slice(jrd_body.as_bytes()).unwrap()
        })
        .unwrap();

        match (
            request.uri().authority().unwrap().as_str(),
            request.uri().path_and_query().unwrap().as_str(),
        ) {
            ("mastodon.com", "/.well-known/webfinger?resource=acct:partridge@mastodon.com")
            | ("example.com", "/.well-known/webfinger?resource=acct:partridge@example.com") => {
                Ok::<_, Infallible>(
                    Response::builder()
                        .header(CONTENT_TYPE, "application/activity+json")
                        .body(Full::<Bytes>::new(jrd_body.into()))
                        .unwrap(),
                )
            }
            ("mastodon.com", "/users/partridge") => {
                let body = include_str!("../../../test-fixtures/ap/partridge_actor.json");

                Ok::<_, Infallible>(
                    Response::builder()
                        .header(CONTENT_TYPE, "application/activity+json")
                        .body(Full::<Bytes>::new(body.into()))
                        .unwrap(),
                )
            }
            path => panic!("HTTP client hit unexpected route: {path:?}"),
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
    assert_eq!(actor.domain, "example.com");
}

#[tokio::test]
async fn fetch_account_with_redirect() {
    let svc = tower::service_fn(|request: Request<_>| async move {
        match (
            request.uri().authority().unwrap().as_str(),
            request.uri().path_and_query().unwrap().as_str(),
        ) {
            ("example.com", "/users/partridge") => Ok::<_, Infallible>(
                Response::builder()
                    .status(StatusCode::MOVED_PERMANENTLY)
                    .header(
                        http::header::LOCATION,
                        "https://mastodon.com/users/partridge",
                    )
                    .body(Full::default())
                    .unwrap(),
            ),
            ("mastodon.com", "/users/partridge") => {
                let body = include_str!("../../../test-fixtures/ap/partridge_actor.json");

                Ok::<_, Infallible>(
                    Response::builder()
                        .header(CONTENT_TYPE, "application/activity+json")
                        .body(Full::<Bytes>::new(body.into()))
                        .unwrap(),
                )
            }
            ("mastodon.com", "/.well-known/webfinger?resource=acct:partridge@mastodon.com") => {
                let jrd_body = include_str!("../../../test-fixtures/ap/partridge_jrd.json");
                let jrd_body = sonic_rs::to_string(&Resource {
                    subject: "acct:partridge@mastodon.com".to_string(),
                    ..sonic_rs::from_slice(jrd_body.as_bytes()).unwrap()
                })
                .unwrap();

                Ok::<_, Infallible>(
                    Response::builder()
                        .header(CONTENT_TYPE, "application/activity+json")
                        .body(Full::<Bytes>::new(jrd_body.into()))
                        .unwrap(),
                )
            }
            path => panic!("HTTP client hit unexpected route: {path:?}"),
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
        .fetch_account("https://example.com/users/partridge", None)
        .await
        .expect("Failed to fetch actor")
        .unwrap();

    assert_eq!(actor.uri, "https://mastodon.com/users/partridge");
    assert_eq!(actor.username, "partridge");
    assert_eq!(actor.domain, "mastodon.com");
}

#[tokio::test]
async fn ignore_fake_webfinger_acct() {
    let svc = tower::service_fn(|request: Request<_>| async move {
        match (
            request.uri().authority().unwrap().as_str(),
            request.uri().path_and_query().unwrap().as_str(),
        ) {
            ("mastodon.com", "/users/partridge") => {
                let body = include_str!("../../../test-fixtures/ap/partridge_actor.json");

                Ok::<_, Infallible>(
                    Response::builder()
                        .header(CONTENT_TYPE, "application/activity+json")
                        .body(Full::<Bytes>::new(body.into()))
                        .unwrap(),
                )
            }
            ("mastodon.com", "/.well-known/webfinger?resource=acct:partridge@mastodon.com") => {
                let jrd_body = include_str!("../../../test-fixtures/ap/partridge_jrd.json");
                let jrd_body = sonic_rs::to_string(&Resource {
                    subject: "acct:partridge@whitehouse.gov".to_string(),
                    ..sonic_rs::from_slice(jrd_body.as_bytes()).unwrap()
                })
                .unwrap();

                Ok::<_, Infallible>(
                    Response::builder()
                        .header(CONTENT_TYPE, "application/activity+json")
                        .body(Full::<Bytes>::new(jrd_body.into()))
                        .unwrap(),
                )
            }
            ("whitehouse.gov", "/.well-known/webfinger?resource=acct:partridge@whitehouse.gov") => {
                let jrd_body = include_str!("../../../test-fixtures/ap/partridge_jrd.json");
                let jrd_body = sonic_rs::to_string(&Resource {
                    subject: "acct:partridge@whitehouse.gov".to_string(),
                    links: vec![Link {
                        rel: "self".to_string(),
                        r#type: Some("application/activity+json".to_string()),
                        href: Some("https://whitehouse.gov/users/partridge".to_string()),
                    }],
                    ..sonic_rs::from_slice(jrd_body.as_bytes()).unwrap()
                })
                .unwrap();

                Ok::<_, Infallible>(
                    Response::builder()
                        .header(CONTENT_TYPE, "application/activity+json")
                        .body(Full::<Bytes>::new(jrd_body.into()))
                        .unwrap(),
                )
            }
            path => panic!("HTTP client hit unexpected route: {path:?}"),
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
        .await;

    assert!(
        actor.is_err(),
        "Expected fetch_account to fail due to fake webfinger acct"
    );
}
