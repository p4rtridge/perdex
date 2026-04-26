use std::convert::Infallible;

use bytes::Bytes;
use http_body_util::Full;
use hyper::{Request, Response};
use pd_http::{Body, Client};

#[tokio::test]
async fn get_bytes() {
    let fake_data = vec![0u8; 1024];
    let fake_data_clone = fake_data.clone();
    let svc = tower::service_fn(move |request: Request<_>| {
        assert_eq!(request.uri().path(), "/bytes");
        let fake_data = fake_data.clone();
        async move { Ok::<_, Infallible>(Response::new(Full::<Bytes>::new(fake_data.into()))) }
    });

    let client = Client::builder().service(svc);

    let request = Request::builder()
        .uri("https://example.org/bytes")
        .body(Body::empty())
        .unwrap();

    let response = client.execute(request).await.unwrap();

    assert!(response.status().is_success());
    let bytes = response.bytes().await.unwrap();
    assert_eq!(bytes.len(), fake_data_clone.len());
    assert_eq!(bytes.as_ref(), fake_data_clone.as_slice())
}
