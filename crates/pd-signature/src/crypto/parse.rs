use super::SigningKey as SigningKeyTrait;
use const_oid::db::{rfc5912::RSA_ENCRYPTION, rfc8410::ID_ED_25519};
use error_stack::Report;
use pkcs8::{
    DecodePrivateKey, Document, PrivateKeyInfo, SecretDocument, SubjectPublicKeyInfoRef,
    der::Decode,
};
use ring::signature::{
    ED25519, Ed25519KeyPair, RSA_PKCS1_2048_8192_SHA256, RsaKeyPair, UnparsedPublicKey,
    VerificationAlgorithm,
};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Malformed PKCS#8 document: {0}")]
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

/// Parse a public key from its PKCS#8 DER form
///
/// Currently supported algorithms:
/// - RSA
/// - Ed25519 (PKCS#8 v2 only)
pub fn public_key(der: &[u8]) -> Result<UnparsedPublicKey<Vec<u8>>, Report<ParseError>> {
    let document = Document::from_der(der).map_err(ParseError::MalformedDer)?;
    let spki = document
        .decode_msg::<SubjectPublicKeyInfoRef<'_>>()
        .map_err(ParseError::MalformedDer)?;

    let algorithm: &dyn VerificationAlgorithm = match spki.algorithm.oid {
        RSA_ENCRYPTION => &RSA_PKCS1_2048_8192_SHA256,
        ID_ED_25519 => &ED25519,
        _ => return Err(Report::new(ParseError::UnsupportedKeyType)),
    };

    let raw_bytes = spki
        .subject_public_key
        .as_bytes()
        .ok_or(ParseError::MalformedKey)?
        .to_vec();
    Ok(UnparsedPublicKey::new(algorithm, raw_bytes))
}

/// Parse a private key from its PKCS#8 DER form
///
/// Currently supported algorithms:
/// - RSA
/// - Ed25519 (PKCS#8 v2 only)
pub fn private_key(der: &[u8]) -> Result<SigningKey, Report<ParseError>> {
    let document = SecretDocument::from_pkcs8_der(der).map_err(ParseError::MalformedPkcs8)?;
    let private_key_raw = document
        .decode_msg::<PrivateKeyInfo<'_>>()
        .map_err(ParseError::MalformedDer)?;

    let signing_key = match private_key_raw.algorithm.oid {
        RSA_ENCRYPTION => SigningKey::Rsa(
            RsaKeyPair::from_pkcs8(private_key_raw.private_key).map_err(ParseError::KeyRejected)?,
        ),
        ID_ED_25519 => SigningKey::Ed25519(
            Ed25519KeyPair::from_pkcs8(private_key_raw.private_key)
                .map_err(ParseError::KeyRejected)?,
        ),
        _ => return Err(Report::new(ParseError::UnsupportedKeyType)),
    };
    Ok(signing_key)
}
