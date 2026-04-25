use std::future;

use http::{Method, Request, Uri};
use pd_signature::cavage::{BoxError, sig::SigExt};
use pkcs8::{Document, SecretDocument};

#[must_use]
fn get_rsa_public_key() -> Vec<u8> {
    let pem = include_str!("key/public_rsa.pem");
    let (_tag, document) = Document::from_pem(pem).unwrap();
    document.as_bytes().to_vec()
}

#[must_use]
fn get_rsa_private_key() -> Vec<u8> {
    let pem = include_str!("key/private_rsa.pem");
    let (_tag, document) = SecretDocument::from_pem(pem).unwrap();
    document.as_bytes().to_vec()
}

#[must_use]
fn get_ed25519_public_key() -> Vec<u8> {
    let pem = include_str!("key/public_ed25519.pem");
    let (_tag, document) = Document::from_pem(pem).unwrap();
    document.as_bytes().to_vec()
}

#[must_use]
fn get_ed25519_private_key() -> Vec<u8> {
    let pem = include_str!("key/private_ed25519.pem");
    let (_tag, document) = SecretDocument::from_pem(pem).unwrap();
    document.as_bytes().to_vec()
}

#[must_use]
fn get_request() -> Request<()> {
    Request::builder()
        .method(Method::POST)
        .uri(Uri::from_static("/foo?param=value&pet=dog"))
        .header("Host", "example.com")
        .header("Date", "Sun, 05 Jan 2014 21:31:40 GMT")
        .header("Content-Type", "application/json")
        .header(
            "Digest",
            "SHA-256=X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=",
        )
        .header("Content-Length", "18")
        .body(())
        .unwrap()
}

#[tokio::test]
async fn with_rsa() {
    let req = get_request();
    let public_key = get_rsa_public_key();

    let signed_req = req.sign("Test", &get_rsa_private_key()).await.unwrap();

    signed_req
        .verify(|key_id| {
            assert_eq!(key_id, "Test");
            future::ready(Ok::<_, BoxError>(public_key.clone()))
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn with_ed25519() {
    let req = get_request();
    let public_key = get_ed25519_public_key();

    let signed_req = req.sign("Test", &get_ed25519_private_key()).await.unwrap();

    signed_req
        .verify(|key_id| {
            assert_eq!(key_id, "Test");
            future::ready(Ok::<_, BoxError>(public_key.clone()))
        })
        .await
        .unwrap();
}

#[cfg(feature = "mock-time-test")]
#[tokio::test]
async fn with_rsa_expire() {
    use std::time::{Duration, SystemTime};

    let req = get_request();

    let signed_req = req.sign("Test", &get_rsa_private_key()).await.unwrap();

    cavage::sig::MOCK_TIME.with(|t: &std::cell::RefCell<Option<SystemTime>>| {
        *t.borrow_mut() = Some(SystemTime::now() + Duration::from_mins(30))
    });

    let _ = signed_req
        .verify(|_key_id| {
            #[allow(unreachable_code)]
            future::ready(unreachable!() as Result<_, BoxError>)
        })
        .await
        .unwrap_err();
}
