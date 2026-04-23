use std::io::Write;

use flate2::{Compression, write::GzEncoder};
use http::{Method, Request, header::CONTENT_ENCODING};
use pd_http::Client;
use reqwest::Body;
use wiremock::{Mock, MockServer};

#[tokio::test]
async fn fail_on_content_length_exceeded() {
    let mock_server = MockServer::start().await;

    let fake_data = vec![0u8; 1024];
    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/bytes"))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_bytes(fake_data.clone()))
        .mount(&mock_server)
        .await;

    let url = format!("{}/bytes", &mock_server.uri());
    let client = Client::builder().body_limit(Some(512)).build().unwrap();
    let request = Request::builder()
        .method(Method::GET)
        .uri(url)
        .body(Body::default())
        .unwrap();
    let response = client.execute(request).await.unwrap().bytes().await;

    assert!(response.is_err());
    let error = response.err().unwrap();
    assert_eq!(
        error.current_context().to_string(),
        "Response body exceeds the configured limit of 512 bytes"
    );
}

#[tokio::test]
async fn fail_on_body_limit_exceeded() {
    let mock_server = MockServer::start().await;

    let raw_body = vec![b'A'; 5000];
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&raw_body).unwrap();
    let compressed_body = encoder.finish().unwrap();

    Mock::given(wiremock::matchers::method("GET"))
        .respond_with(
            wiremock::ResponseTemplate::new(200)
                .insert_header(CONTENT_ENCODING, "gzip")
                .set_body_bytes(compressed_body),
        )
        .mount(&mock_server)
        .await;

    let url = format!("{}/bytes", &mock_server.uri());
    let client = Client::builder().body_limit(Some(1024)).build().unwrap();
    let request = Request::builder()
        .method(Method::GET)
        .uri(url)
        .body(Body::default())
        .unwrap();
    let response = client.execute(request).await.unwrap();

    assert_eq!(response.content_length(), None);

    let response = response.bytes().await;
    assert!(response.is_err());
    let error = response.err().unwrap();
    assert_eq!(
        error.current_context().to_string(),
        "Response body exceeds the configured limit of 1024 bytes"
    );
}
