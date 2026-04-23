use http::{Method, Request};
use pd_http::Client;
use reqwest::Body;
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
    let client = Client::builder().build().unwrap();
    let request = Request::builder()
        .method(Method::GET)
        .uri(url)
        .body(Body::default())
        .unwrap();
    let response = client.execute(request).await.unwrap();

    assert!(response.status().is_success());
    let bytes = response.bytes().await.unwrap();
    assert_eq!(bytes.len(), fake_data.len());
    assert_eq!(bytes.as_ref(), fake_data.as_slice())
}
