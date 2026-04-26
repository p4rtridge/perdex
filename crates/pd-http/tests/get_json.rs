use std::convert::Infallible;

use bytes::Bytes;
use http_body_util::Full;
use hyper::{Request, Response};
use pd_http::{Body, Client};

#[tokio::test]
async fn get_json() {
    let json_data = sonic_rs::json!({"key": "value"});
    let json_data_clone = json_data.clone();
    let svc = tower::service_fn(move |request: Request<_>| {
        assert_eq!(request.uri().path(), "/json");
        let json_data = json_data.clone();
        async move { Ok::<_, Infallible>(Response::new(Full::new(Bytes::from(json_data.to_string())))) }
    });

    let client = Client::builder().service(svc);

    let request = Request::builder()
        .uri("https://example.com/json")
        .body(Body::empty())
        .unwrap();

    let response = client.execute(request).await.unwrap();

    assert!(response.status().is_success());
    let deserialized: sonic_rs::Value = response.json().await.unwrap();
    assert_eq!(deserialized, json_data_clone);
}
