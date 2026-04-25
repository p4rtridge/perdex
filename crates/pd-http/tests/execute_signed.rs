use http::{Method, Request};
use pd_http::Client;
use pkcs8::SecretDocument;
use reqwest::Body;
use wiremock::{Mock, MockServer, matchers::any};

#[tokio::test]
async fn test_execute_signed() {
    let mock_server = MockServer::start().await;

    Mock::given(any())
        .respond_with(wiremock::ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let url = format!("{}/inbox", &mock_server.uri());
    let client = Client::builder().build().unwrap();
    let request = Request::builder()
        .method(Method::POST)
        .uri(url)
        .header("Host", mock_server.address().to_string())
        .header("Content-Type", "application/activity+json")
        .header(
            "Digest",
            "SHA-256=X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=",
        )
        .body(Body::default())
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

    // Verify the request received by wiremock
    let requests = mock_server.received_requests().await;
    assert!(requests.is_some());
    let requests = requests.unwrap();
    assert_eq!(requests.len(), 1);

    let received_req = &requests[0];

    assert!(received_req.headers.contains_key("signature"));
    assert!(received_req.headers.contains_key("date"));

    let signature_header = received_req
        .headers
        .get("signature")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(signature_header.contains("keyId=\"https://my-server.com/users/alice#main-key\""));
    assert!(signature_header.contains("headers=\"host date content-type digest\""));
}
