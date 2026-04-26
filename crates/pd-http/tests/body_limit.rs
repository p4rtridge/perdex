use std::{convert::Infallible, io::Write};

use bytes::Bytes;
use flate2::{Compression, write::GzEncoder};
use http::header::CONTENT_ENCODING;
use http_body_util::Full;
use hyper::{Request, Response};
use pd_http::{Body, Client};

#[tokio::test]
async fn gzip() {
    let raw_body = vec![b'A'; 5 * 1024]; // 5 KiB
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&raw_body).unwrap();
    let compressed_body = encoder.finish().unwrap();

    assert!(compressed_body.len() < 1024);

    let svc = tower::service_fn(move |req: Request<_>| {
        assert_eq!(req.uri().path_and_query().unwrap(), "/path");
        let body = compressed_body.clone();

        async move {
            let response = Response::builder()
                .header(CONTENT_ENCODING, "gzip")
                .body(Full::<Bytes>::new(body.into()))
                .unwrap();

            Ok::<_, Infallible>(response)
        }
    });
    let client = Client::builder().body_limit(Some(1024)).service(svc);

    let req = Request::builder()
        .uri("https://example.com/path")
        .body(Body::empty())
        .unwrap();
    let response = client.execute(req).await.unwrap();

    let body = response.bytes().await;

    assert!(body.is_err())
}
