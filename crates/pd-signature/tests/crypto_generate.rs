use pd_signature::crypto::generate::generate_rsa_keypair;
use pkcs8::{
    DecodePrivateKey, Document, PrivateKeyInfo, SecretDocument, SubjectPublicKeyInfoRef,
    der::Decode,
};

#[tokio::test]
async fn test_generate_rsa_keypair_valid_der() {
    let result = generate_rsa_keypair()
        .await
        .expect("Failed to generate keypair");

    let private_key_raw = SecretDocument::from_pkcs8_der(&result.private_key)
        .expect("Failed to parse private key DER");
    let _ = private_key_raw.decode_msg::<PrivateKeyInfo<'_>>().unwrap();

    let public_key =
        Document::from_der(&result.public_key).expect("Failed to parse public key DER");
    let _ = public_key
        .decode_msg::<SubjectPublicKeyInfoRef<'_>>()
        .unwrap();
}
