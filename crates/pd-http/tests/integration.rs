use std::io::Write;

use flate2::{Compression, write::GzEncoder};
use futures_util::StreamExt;
use http::{Method, header::CONTENT_ENCODING};
use pd_http::{HttpClient, Request};
use wiremock::{Mock, MockServer};

#[tokio::test]
async fn get_bytes() {
    let mock_server = MockServer::start().await;

    let fake_data = vec![0u8; 1024];
    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/bytes"))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_bytes(fake_data.clone()))
        .mount(&mock_server)
        .await;

    let url = format!("{}/bytes", &mock_server.uri());
    let client = HttpClient::builder().build().unwrap();
    let request = Request::new(Method::GET, url.parse().unwrap());
    let response = client.execute(request).await.unwrap();

    assert!(response.status().is_success());
    let bytes = response.bytes().await.unwrap();
    assert_eq!(bytes.len(), fake_data.len());
    assert_eq!(bytes.as_ref(), fake_data.as_slice())
}

#[tokio::test]
async fn get_stream() {
    let mock_server = MockServer::start().await;

    let fake_data = vec![5u8; 4096];
    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/stream"))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_bytes(fake_data.clone()))
        .mount(&mock_server)
        .await;

    let url = format!("{}/stream", &mock_server.uri());
    let client = HttpClient::builder().build().unwrap();
    let request = Request::new(Method::GET, url.parse().unwrap());
    let mut stream = client.execute(request).await.unwrap().stream().await;

    let mut downloaded = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        downloaded.extend_from_slice(&chunk);
    }

    assert_eq!(downloaded.len(), fake_data.len());
    assert_eq!(downloaded.as_slice(), fake_data.as_slice());
}

#[tokio::test]
async fn get_json() {
    let mock_server = MockServer::start().await;

    let json_data = sonic_rs::json!({
        "name": "Alice",
        "age": 30,
        "is_student": false
    });
    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/json"))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(&json_data))
        .mount(&mock_server)
        .await;

    let url = format!("{}/json", &mock_server.uri());
    let client = HttpClient::builder().build().unwrap();
    let request = Request::new(Method::GET, url.parse().unwrap());
    let response = client.execute(request).await.unwrap();

    assert!(response.status().is_success());
    let deserialized: sonic_rs::Value = response.json().await.unwrap();
    assert_eq!(deserialized, json_data);
}

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
    let client = HttpClient::builder().body_limit(Some(512)).build().unwrap();
    let request = Request::new(Method::GET, url.parse().unwrap());
    let response = client.execute(request).await.unwrap().bytes().await;

    assert!(response.is_err());
    let error = response.err().unwrap();
    assert_eq!(error.to_string(), "Content length exceeds body limit");
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
    let client = HttpClient::builder()
        .body_limit(Some(1024))
        .build()
        .unwrap();
    let request = Request::new(Method::GET, url.parse().unwrap());
    let response = client.execute(request).await.unwrap();

    assert_eq!(response.content_length(), None);

    let response = response.bytes().await;
    assert!(response.is_err());
    let error = response.err().unwrap();
    assert_eq!(error.to_string(), "Response body exceeds body limit");
}
