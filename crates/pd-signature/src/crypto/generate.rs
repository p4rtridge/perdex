use error_stack::{Report, ResultExt};
use pd_blocking;
use rsa::{
    RsaPrivateKey,
    pkcs8::{EncodePrivateKey, EncodePublicKey},
};

#[derive(Debug, thiserror::Error)]
pub enum GenerateError {
    #[error("Failed to generate RSA key pair")]
    Generation,

    #[error("Failed to encode key to DER format")]
    Encoding,

    #[error("Blocking task failed")]
    Blocking,
}

pub struct KeyPairDer {
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

/// Generates a new 4096-bit RSA Key Pair and returns them in DER format.
///
/// Because RSA key generation is computationally expensive, this function executes
/// on a dedicated blocking thread pool for cryptography tasks (`pd_blocking::crypto`).
pub async fn generate_rsa_keypair() -> Result<KeyPairDer, Report<GenerateError>> {
    let keypair = pd_blocking::crypto(|| {
        let mut rng = rand::rng();
        RsaPrivateKey::new(&mut rng, 4096)
    })
    .await
    .change_context(GenerateError::Blocking)?
    .change_context(GenerateError::Generation)?;

    let private_key_der = keypair
        .to_pkcs8_der()
        .change_context(GenerateError::Encoding)?
        .to_bytes()
        .to_vec();

    let public_key_der = keypair
        .to_public_key()
        .to_public_key_der()
        .change_context(GenerateError::Encoding)?
        .to_vec();

    Ok(KeyPairDer {
        private_key: private_key_der,
        public_key: public_key_der,
    })
}
