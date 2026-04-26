use std::convert::Infallible;

use bytes::Bytes;
use http::Method;
use http_body_util::Full;
use hyper::{Request, Response};
use pd_http::{Body, Client};
use pkcs8::SecretDocument;

#[tokio::test]
async fn test_execute_signed() {
    let json_data = sonic_rs::json!({"key": "value"});
    let svc = tower::service_fn(move |request: Request<_>| {
        assert_eq!(request.method(), Method::POST);
        assert_eq!(request.uri().path(), "/inbox");
        assert_eq!(request.headers().get("Host").unwrap(), "my-server.com");
        assert_eq!(
            request.headers().get("Content-Type").unwrap(),
            "application/activity+json"
        );
        assert_eq!(
            request.headers().get("Digest").unwrap(),
            "SHA-256=X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE="
        );
        let signature_header = request
            .headers()
            .get("Signature")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(signature_header.contains("keyId=\"https://my-server.com/users/alice#main-key\""));
        assert!(signature_header.contains("headers=\"host date content-type digest\""));

        let json_data = json_data.clone();
        async move { Ok::<_, Infallible>(Response::new(Full::new(Bytes::from(json_data.to_string())))) }
    });

    let client = Client::builder().service(svc);

    let request = Request::builder()
        .method(Method::POST)
        .uri("https://my-server.com/inbox")
        .header("Host", "my-server.com")
        .header("Content-Type", "application/activity+json")
        .header(
            "Digest",
            "SHA-256=X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=",
        )
        .body(Body::empty())
        .unwrap();

    let private_key_pem = include_str!("../../pd-signature/tests/key/private_rsa.pem");
    let (_tag, document) = SecretDocument::from_pem(private_key_pem).unwrap();

    let response = client
        .execute_signed(
            request,
            "https://my-server.com/users/alice#main-key",
            document.as_bytes(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success());
}
