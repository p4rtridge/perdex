use std::{collections::HashMap, net::SocketAddr};

use axum::{Json, Router, extract::Query, http::StatusCode, response::IntoResponse, routing};
use axum_server::tls_rustls::RustlsConfig;
use pd_core::federation::domain::account::AccountResolver;
use pd_federation::{ap_types::webfinger::Resource, resolver::webfinger::Webfinger};
use pd_http::Client;

async fn load_tls_config() -> RustlsConfig {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let cert = std::fs::read("tests/certs/cert.pem").expect("Failed to read TLS certificate");
    let key = std::fs::read("tests/certs/key.pem").expect("Failed to read TLS private key");
    RustlsConfig::from_pem(cert, key)
        .await
        .expect("Failed to create TLS config")
}

async fn prepare_mock_server() -> u16 {
    let app = Router::new().route(
        "/.well-known/webfinger",
        routing::get(|Query(params): Query<HashMap<String, String>>| async move {
            let resource = params.get("resource").cloned().unwrap_or_default();

            if resource.starts_with("acct:not_found@") {
                return (StatusCode::NOT_FOUND, "Not Found").into_response();
            }

            if resource.starts_with("acct:bad_json@") {
                return (StatusCode::OK, "{ bad json }").into_response();
            }

            let base = include_bytes!("datas/webfinger/partridge_jrd.json");

            if resource.starts_with("acct:invalid_syntax@") {
                // Return subject that does not have acct: or @ domain format
                let body = sonic_rs::json!(&Resource {
                    subject: "invalid_syntax".to_string(),
                    ..sonic_rs::from_slice(base).unwrap()
                });
                return Json(body).into_response();
            }

            if resource.starts_with("acct:no_self_link@") {
                let mut res: Resource = sonic_rs::from_slice(base).unwrap();
                res.subject = resource;
                res.links.clear(); // Removing all links, so no rel="self"
                return Json(res).into_response();
            }

            let resource_buf = resource.strip_prefix("acct:partridge_");

            let body: sonic_rs::Value;
            if let Some(count) = resource_buf
                .and_then(|suffix| suffix.split('@').next())
                .and_then(|count| count.parse::<u32>().ok())
            {
                let domain = resource_buf.unwrap().split('@').nth(1).unwrap();
                body = sonic_rs::json!(&Resource {
                    subject: format!("acct:partridge_{}@{domain}", count + 1),
                    ..sonic_rs::from_slice(base).unwrap()
                });
            } else {
                body = sonic_rs::json!(&Resource {
                    subject: resource,
                    ..sonic_rs::from_slice(base).unwrap()
                });
            }
            Json(body).into_response()
        }),
    );

    let tls_config = load_tls_config().await;
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let handle = axum_server::Handle::new();
    let handle_clone = handle.clone();
    tokio::spawn(async move {
        axum_server::bind_rustls(addr, tls_config)
            .handle(handle_clone)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    let mut bounded_port = 0;
    for _ in 0..10 {
        if let Some(addr) = handle.listening().await {
            bounded_port = addr.port();
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert!(
        bounded_port != 0,
        "Failed to bind to a port for the mock server"
    );
    bounded_port
}

fn create_client() -> Webfinger {
    let client = Client::builder()
        .accept_invalid_certs(true)
        .build()
        .unwrap();
    Webfinger::builder().http_client(client).build()
}

#[tokio::test]
async fn basic() {
    let mock_port = prepare_mock_server().await;
    let webfinger = create_client();
    let domain = format!("127.0.0.1:{}", mock_port);
    let resource = webfinger
        .resolve_account("partridge", &domain)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(resource.username, "partridge");
    assert_eq!(resource.domain, domain);
}

#[tokio::test]
async fn redirect_unbounded() {
    let mock_port = prepare_mock_server().await;
    let webfinger = create_client();
    let domain = format!("127.0.0.1:{}", mock_port);
    let resource = webfinger
        .resolve_account("partridge_0", &domain)
        .await
        .unwrap();
    assert!(resource.is_none());
}

#[tokio::test]
async fn not_found_returns_none() {
    let mock_port = prepare_mock_server().await;
    let webfinger = create_client();
    let resource = webfinger
        .resolve_account("not_found", &format!("127.0.0.1:{}", mock_port))
        .await
        .unwrap();
    assert!(resource.is_none());
}

#[tokio::test]
async fn bad_json_returns_error() {
    let mock_port = prepare_mock_server().await;
    let webfinger = create_client();
    let resource = webfinger
        .resolve_account("bad_json", &format!("127.0.0.1:{}", mock_port))
        .await;
    assert!(resource.is_err());
}

#[tokio::test]
async fn invalid_acct_returns_none() {
    let mock_port = prepare_mock_server().await;
    let webfinger = create_client();
    let resource = webfinger
        .resolve_account("invalid_syntax", &format!("127.0.0.1:{}", mock_port))
        .await
        .unwrap();
    assert!(resource.is_none());
}

#[tokio::test]
async fn no_self_link_returns_none() {
    let mock_port = prepare_mock_server().await;
    let webfinger = create_client();
    let resource = webfinger
        .resolve_account("no_self_link", &format!("127.0.0.1:{}", mock_port))
        .await
        .unwrap();
    assert!(resource.is_none());
}

#[tokio::test]
async fn network_error_returns_error() {
    let webfinger = create_client();
    // To trigger a fast network error, we use a closed local port or an invalid scheme.
    let resource = webfinger.resolve_account("partridge", "127.0.0.1:0").await;
    assert!(resource.is_err());
}
