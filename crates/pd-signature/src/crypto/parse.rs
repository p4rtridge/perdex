use super::SigningKey as SigningKeyTrait;
use const_oid::db::{rfc5912::RSA_ENCRYPTION, rfc8410::ID_ED_25519};
use error_stack::Report;
use pkcs8::{DecodePrivateKey, PrivateKeyInfo, SecretDocument};
use ring::signature::{Ed25519KeyPair, RsaKeyPair};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Malformed PKCS#8 document")]
    MalformedPkcs8(#[from] pkcs8::Error),

    #[error("Malformed der structure: {0}")]
    MalformedDer(#[from] pkcs8::der::Error),

    #[error("Key rejected: {0}")]
    KeyRejected(#[from] ring::error::KeyRejected),

    #[error("Malformed key")]
    MalformedKey,

    #[error("Unsupported key type")]
    UnsupportedKeyType,
}

pub enum SigningKey {
    Rsa(RsaKeyPair),
    Ed25519(Ed25519KeyPair),
}

impl SigningKeyTrait for SigningKey {
    type Output = Vec<u8>;

    fn sign(&self, payload: &[u8]) -> Self::Output {
        match self {
            Self::Ed25519(key) => key.sign(payload).as_ref().to_vec(),
            Self::Rsa(key) => SigningKeyTrait::sign(key, payload),
        }
    }
}

/// Parse a private key from its PKCS#8 DER form.
///
/// Currently supported algorithms:
/// - RSA
/// - Ed25519
pub fn private_key(key: &[u8]) -> Result<SigningKey, Report<ParseError>> {
    let document = SecretDocument::from_pkcs8_der(key).map_err(ParseError::MalformedPkcs8)?;
    let private_key_raw: PrivateKeyInfo<'_> =
        document.decode_msg().map_err(ParseError::MalformedDer)?;

    let signing_key = match private_key_raw.algorithm.oid {
        RSA_ENCRYPTION => SigningKey::Rsa(
            RsaKeyPair::from_der(private_key_raw.private_key).map_err(ParseError::KeyRejected)?,
        ),
        ID_ED_25519 => SigningKey::Ed25519(
            Ed25519KeyPair::from_seed_and_public_key(
                private_key_raw.private_key,
                private_key_raw.public_key.ok_or(ParseError::MalformedKey)?,
            )
            .map_err(ParseError::KeyRejected)?,
        ),
        _ => return Err(Report::new(ParseError::UnsupportedKeyType)),
    };
    Ok(signing_key)
}
