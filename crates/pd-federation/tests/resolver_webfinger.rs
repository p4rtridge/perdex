use std::{collections::HashMap, net::SocketAddr};

use axum::{Json, Router, extract::Query, routing};
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
            let base = include_bytes!("datas/webfinger/partridge_jrd.json");

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
            Json(body)
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

#[tokio::test]
async fn basic() {
    let mock_port = prepare_mock_server().await;

    let client = Client::builder()
        .accept_invalid_certs(true)
        .build()
        .unwrap();
    let webfinger = Webfinger::builder().http_client(client).build();

    let domain = format!("127.0.0.1:{}", mock_port);
    let resource = webfinger
        .resolve_account("partridge", &domain)
        .await
        .expect("Failed to resolve account")
        .unwrap();

    assert_eq!(resource.username, "partridge");
    assert_eq!(resource.domain, domain);
}

#[tokio::test]
async fn redirect_unbounded() {
    let mock_port = prepare_mock_server().await;

    let client = Client::builder()
        .accept_invalid_certs(true)
        .build()
        .unwrap();
    let webfinger = Webfinger::builder().http_client(client).build();

    let domain = format!("127.0.0.1:{}", mock_port);
    let resource = webfinger
        .resolve_account("partridge_0", &domain)
        .await
        .expect("Failed to resolve account");

    assert!(
        resource.is_none(),
        "Expected no subject for unbounded redirect, but got {:?}",
        resource
    );
}
