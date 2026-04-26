use std::convert::Infallible;

use bytes::Bytes;
use futures_util::StreamExt;
use http_body_util::Full;
use hyper::{Request, Response};
use pd_http::{Body, Client};

#[tokio::test]
async fn get_stream() {
    let fake_data = vec![5u8; 4096];
    let fake_data_clone = fake_data.clone();
    let svc = tower::service_fn(move |request: Request<_>| {
        assert_eq!(request.uri().path(), "/stream");
        let fake_data = fake_data.clone();
        async move { Ok::<_, Infallible>(Response::new(Full::new(Bytes::from(fake_data)))) }
    });

    let client = Client::builder().service(svc);

    let request = Request::builder()
        .uri("https://example.com/stream")
        .body(Body::empty())
        .unwrap();

    let mut stream = client.execute(request).await.unwrap().stream().await;

    let mut downloaded = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        downloaded.extend_from_slice(&chunk);
    }

    assert_eq!(downloaded.len(), fake_data_clone.len());
    assert_eq!(downloaded.as_slice(), fake_data_clone.as_slice());
}
