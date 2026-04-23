use http::{Method, Request};
use pd_http::Client;
use reqwest::Body;
use wiremock::{Mock, MockServer};

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
    let client = Client::builder().build().unwrap();
    let request = Request::builder()
        .method(Method::GET)
        .uri(url)
        .body(Body::default())
        .unwrap();
    let response = client.execute(request).await.unwrap();

    assert!(response.status().is_success());
    let deserialized: sonic_rs::Value = response.json().await.unwrap();
    assert_eq!(deserialized, json_data);
}
