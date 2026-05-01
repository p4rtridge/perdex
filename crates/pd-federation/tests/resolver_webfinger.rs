use std::convert::Infallible;

use bytes::Bytes;
use http_body_util::{Empty, Full};
use hyper::{Request, Response};
use pd_core::account::traits::AccountResolver;
use pd_federation::resolver::webfinger::Webfinger;
use pd_http::Client;

use pd_federation::ap_type::webfinger::Resource;

#[tokio::test]
async fn basic() {
    let svc = tower::service_fn(|request: Request<_>| {
        assert_eq!(request.uri().path(), "/.well-known/webfinger");

        let mut resource = request.uri().query().unwrap().split('=');
        assert_eq!(resource.next(), Some("resource"));
        let resource = resource.next().unwrap();
        assert_eq!(resource, "acct:partridge@example.org");

        let base = include_bytes!("../../../test-fixtures/ap/partridge_jrd.json");
        let body = sonic_rs::json!(&Resource {
            subject: "acct:partridge@example.org".to_string(),
            ..sonic_rs::from_slice(base).unwrap()
        });

        async move { Ok::<_, Infallible>(Response::new(Full::new(Bytes::from(body.to_string())))) }
    });

    let client = Client::builder().service(svc);

    let webfinger = Webfinger::builder().http_client(client).build();

    let resource = webfinger
        .resolve_account("partridge", "example.org")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(resource.username, "partridge");
    assert_eq!(resource.domain, "example.org");
}

#[tokio::test]
async fn redirect_unbounded() {
    let svc = tower::service_fn(|request: Request<_>| {
        assert_eq!(request.uri().path(), "/.well-known/webfinger");

        let mut resource = request.uri().query().unwrap().split('=');
        assert_eq!(resource.next(), Some("resource"));
        let resource = resource.next().unwrap();

        let Some(count) = resource
            .strip_prefix("acct:partridge_")
            .and_then(|suffix| suffix.strip_suffix("@example.org"))
            .and_then(|count| count.parse::<u32>().ok())
        else {
            panic!("Unexpected resource format");
        };

        let base = include_bytes!("../../../test-fixtures/ap/partridge_jrd.json");
        let body = sonic_rs::json!(&Resource {
            subject: format!("acct:partridge_{}@example.org", count + 1),
            ..sonic_rs::from_slice(base).unwrap()
        });

        async move { Ok::<_, Infallible>(Response::new(Full::new(Bytes::from(body.to_string())))) }
    });

    let client = Client::builder().service(svc);

    let webfinger = Webfinger::builder().http_client(client).build();

    let resource = webfinger
        .resolve_account("partridge_0", "example.org")
        .await
        .unwrap();
    assert!(
        resource.is_none(),
        "Expected None due to too many redirects, got {resource:?}"
    );
}

#[tokio::test]
async fn not_found_returns_none() {
    let svc = tower::service_fn(|request: Request<_>| {
        assert_eq!(request.uri().path(), "/.well-known/webfinger");

        let mut resource = request.uri().query().unwrap().split('=');
        assert_eq!(resource.next(), Some("resource"));
        let resource = resource.next().unwrap();
        assert_eq!(resource, "acct:not_found@example.org");

        async move {
            Ok::<_, Infallible>(
                Response::builder()
                    .status(404)
                    .body(Empty::<Bytes>::new())
                    .unwrap(),
            )
        }
    });

    let client = Client::builder().service(svc);

    let webfinger = Webfinger::builder().http_client(client).build();

    let resource = webfinger
        .resolve_account("not_found", "example.org")
        .await
        .unwrap();

    assert!(
        resource.is_none(),
        "Expected None for not found resource, got {resource:?}"
    );
}

#[tokio::test]
async fn bad_json_returns_error() {
    let svc = tower::service_fn(|request: Request<_>| {
        assert_eq!(request.uri().path(), "/.well-known/webfinger");

        let mut resource = request.uri().query().unwrap().split('=');
        assert_eq!(resource.next(), Some("resource"));
        let resource = resource.next().unwrap();
        assert_eq!(resource, "acct:bad_json@example.org");

        async move {
            Ok::<_, Infallible>(
                Response::builder()
                    .status(200)
                    .body(Full::new(Bytes::from("{ bad json }")))
                    .unwrap(),
            )
        }
    });

    let client = Client::builder().service(svc);

    let webfinger = Webfinger::builder().http_client(client).build();

    let resource = webfinger.resolve_account("bad_json", "example.org").await;

    assert!(
        resource.is_err(),
        "Expected error for bad JSON resource, got {resource:?}"
    );
}

#[tokio::test]
async fn invalid_acct_returns_none() {
    let svc = tower::service_fn(|request: Request<_>| {
        assert_eq!(request.uri().path(), "/.well-known/webfinger");

        let base = include_bytes!("../../../test-fixtures/ap/partridge_jrd.json");
        let body = sonic_rs::json!(&Resource {
            subject: "invalid".to_string(),
            ..sonic_rs::from_slice(base).unwrap()
        });

        async move { Ok::<_, Infallible>(Response::new(Full::new(Bytes::from(body.to_string())))) }
    });

    let client = Client::builder().service(svc);

    let webfinger = Webfinger::builder().http_client(client).build();

    let resource = webfinger
        .resolve_account("invalid", "example.org")
        .await
        .unwrap();

    assert!(
        resource.is_none(),
        "Expected None for invalid resource, got {resource:?}"
    );
}

#[tokio::test]
async fn no_self_link_returns_none() {
    let svc = tower::service_fn(|request: Request<_>| {
        assert_eq!(request.uri().path(), "/.well-known/webfinger");

        let mut resource = request.uri().query().unwrap().split('=');
        assert_eq!(resource.next(), Some("resource"));
        let resource = resource.next().unwrap();
        assert_eq!(resource, "acct:no_self_link@example.org");

        let base = include_bytes!("../../../test-fixtures/ap/partridge_jrd.json");
        let mut body: Resource = sonic_rs::from_slice(base).unwrap();
        body.subject = resource.to_string();
        body.links.clear();

        async move {
            Ok::<_, Infallible>(Response::new(Full::new(Bytes::from(
                sonic_rs::to_string(&body).unwrap(),
            ))))
        }
    });

    let client = Client::builder().service(svc);

    let webfinger = Webfinger::builder().http_client(client).build();

    let resource = webfinger
        .resolve_account("no_self_link", "example.org")
        .await
        .unwrap();

    assert!(
        resource.is_none(),
        "Expected None for no_self_link resource, got {resource:?}"
    );
}

#[tokio::test]
async fn network_error_returns_error() {
    let client = Client::builder().build();

    let webfinger = Webfinger::builder().http_client(client).build();

    let resource = webfinger.resolve_account("partridge", "127.0.0.1:0").await;
    assert!(resource.is_err());
}
