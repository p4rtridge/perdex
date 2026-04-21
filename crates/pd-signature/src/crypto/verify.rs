use error_stack::{Report, ResultExt};
use ring::signature::UnparsedPublicKey;

/// Verification errors
#[derive(Debug, thiserror::Error)]
pub enum VerifyError {
    // Failed to decode the signature from base64
    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Verification failed")]
    Verification,
}

/// Verify the message with the given signature and public key
pub fn verify<B>(
    message: &[u8],
    encoded_signature: &str,
    key: &UnparsedPublicKey<B>,
) -> Result<(), Report<VerifyError>>
where
    B: AsRef<[u8]>,
{
    let signature = base64_simd::STANDARD
        .decode_to_vec(encoded_signature)
        .change_context(VerifyError::InvalidSignature)?;
    key.verify(message, &signature)
        .change_context(VerifyError::Verification)
}
