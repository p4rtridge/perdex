use pd_signature::crypto::{self, generate::generate_rsa_keypair};

#[tokio::test]
async fn test_generate_rsa_keypair_valid_der() {
    let result = generate_rsa_keypair()
        .await
        .expect("Failed to generate keypair");

    let _ =
        crypto::parse::private_key(&result.private_key).expect("Failed to parse private key DER");
    let _ = crypto::parse::public_key(&result.public_key).expect("Failed to parse public key DER");
}
