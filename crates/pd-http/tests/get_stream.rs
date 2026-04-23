use futures_util::StreamExt;
use http::{Method, Request};
use pd_http::Client;
use reqwest::Body;
use wiremock::{Mock, MockServer};

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
    let client = Client::builder().build().unwrap();
    let request = Request::builder()
        .method(Method::GET)
        .uri(url)
        .body(Body::default())
        .unwrap();
    let mut stream = client.execute(request).await.unwrap().stream().await;

    let mut downloaded = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        downloaded.extend_from_slice(&chunk);
    }

    assert_eq!(downloaded.len(), fake_data.len());
    assert_eq!(downloaded.as_slice(), fake_data.as_slice());
}
