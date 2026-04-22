use std::future;

use const_oid::db::{rfc5912::RSA_ENCRYPTION, rfc8410::ID_ED_25519};
use http::{Method, Request, Uri};
use pd_signature::cavage::{self, BoxError};
use pkcs8::{Document, PrivateKeyInfo, SecretDocument, der::Encode, spki::AlgorithmIdentifier};

#[must_use]
fn get_rsa_public_key() -> Vec<u8> {
    let pem = include_str!("key/public_rsa.pem");
    let (_tag, document) = Document::from_pem(pem).unwrap();
    document.as_bytes().to_vec()
}

#[must_use]
fn get_rsa_pkcs8_private_key() -> Vec<u8> {
    let pem = include_str!("key/private_rsa.pem");
    let (_tag, document) = SecretDocument::from_pem(pem).unwrap();
    let private_key_info = PrivateKeyInfo {
        algorithm: AlgorithmIdentifier {
            oid: RSA_ENCRYPTION,
            parameters: None,
        },
        private_key: document.as_bytes(),
        public_key: None,
    };
    private_key_info.to_der().unwrap()
}

#[must_use]
fn get_ed25519_public_key() -> Vec<u8> {
    let pem = include_str!("key/public_ed25519.pem");
    let (_tag, document) = Document::from_pem(pem).unwrap();
    document.as_bytes().to_vec()
}

#[must_use]
fn get_ed25519_pkcs8_private_key() -> Vec<u8> {
    let pem = include_str!("key/private_ed25519.pem");
    let (_tag, document) = SecretDocument::from_pem(pem).unwrap();
    let private_key_info = PrivateKeyInfo {
        algorithm: AlgorithmIdentifier {
            oid: ID_ED_25519,
            parameters: None,
        },
        private_key: document.as_bytes(),
        public_key: None,
    };
    private_key_info.to_der().unwrap()
}

#[must_use]
pub fn get_request() -> Request<()> {
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

    let signed_req = cavage::sig::sign(req, "Test", &get_rsa_pkcs8_private_key())
        .await
        .unwrap();

    cavage::sig::verify(&signed_req, |key_id| {
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

    let signed_req = cavage::sig::sign(req, "Test", &get_ed25519_pkcs8_private_key())
        .await
        .unwrap();

    cavage::sig::verify(&signed_req, |key_id| {
        assert_eq!(key_id, "Test");
        future::ready(Ok::<_, BoxError>(public_key.clone()))
    })
    .await
    .unwrap();
}
