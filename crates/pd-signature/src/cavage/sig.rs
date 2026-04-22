use std::fmt::Write;
use std::time::SystemTime;

use error_stack::{Report, ResultExt};
use http::{HeaderValue, Method, Request, header::DATE};
use thiserror::Error;

use crate::{
    cavage::{BoxError, SignatureHeader, header},
    crypto,
};

const SIGNATURE_HEADER: &str = "Signature";
const GET_HEADERS: &[&str] = &["host", "date"];
const POST_HEADERS: &[&str] = &["host", "date", "content-type", "digest"];

#[derive(Debug, Error)]
pub enum SigError {
    #[error("Blocking error")]
    Blocking,

    #[error("Failed to build signature string")]
    BuildSigString,

    #[error("Missing signature header")]
    MissingSignatureHeader,

    #[error("Failed to get key: {0}")]
    GetKey(#[source] BoxError),

    #[error("Invalid key")]
    InvalidKey,

    #[error("Failed to parse signature header")]
    InvalidSignatureHeader,

    #[error("Unsupported HTTP method")]
    UnsupportedHttpMethod,

    #[error("Verification failed")]
    Verify,
}

/// Sign an HTTP request using the provided signing key
///
/// The key parameter has to be an PEM-encoded private key in the PKCS#8 format
///
/// This will fail if the key algorithm is unsupported
///
/// Currently supported algorithms:
/// - RSA
/// - Ed25519 (PKCS#8 v2 only)
pub async fn sign<B>(
    mut req: Request<B>,
    key_id: &str,
    key: &[u8],
) -> Result<Request<B>, Report<SigError>> {
    // Overwrite the date header with the current time
    let date_header_value = HeaderValue::from_str(&httpdate::fmt_http_date(SystemTime::now()))
        .expect("Failed to format date header value");
    req.headers_mut().insert(DATE, date_header_value);

    let headers = match *req.method() {
        Method::GET => GET_HEADERS.iter().copied(),
        Method::POST => POST_HEADERS.iter().copied(),
        _ => return Err(Report::new(SigError::UnsupportedHttpMethod)),
    };

    let signature_header = SignatureHeader {
        key_id,
        headers,
        signature: "",
        created: None,
        expires: None,
    };
    let signature_string =
        build_signature_string(&req, &signature_header).change_context(SigError::BuildSigString)?;

    let key = crypto::parse::private_key(key).change_context(SigError::InvalidKey)?;
    let signature = pd_blocking::crypto(move || crypto::sign(signature_string.as_bytes(), &key))
        .await
        .change_context(SigError::Blocking)?;

    let signature_header = SignatureHeader {
        key_id: signature_header.key_id,
        headers: signature_header.headers,
        signature: &signature,
        created: signature_header.created,
        expires: signature_header.expires,
    };
    let signature_header_value = HeaderValue::from_str(&header::serialize(signature_header))
        .expect("Failed to serialize signature header");
    req.headers_mut()
        .insert(SIGNATURE_HEADER, signature_header_value);

    Ok(req)
}

/// Verify an HTTP requests signature using the provided key closure
///
/// The closure is expected to return a future which resolves into a result which contains a PEM-encoded PKCS#8 verifying key.
///
/// This will fail if the key algorithm is unsupported
///
/// Currently supported algorithms:
/// - RSA
/// - Ed25519 (PKCS#8 v2 only)
pub async fn verify<B, F, Fut, E>(req: &Request<B>, get_key: F) -> Result<(), Report<SigError>>
where
    F: Fn(&str) -> Fut,
    Fut: Future<Output = Result<Vec<u8>, E>>,
    E: Into<BoxError>,
{
    let Some(sig_header_value) = req.headers().get(SIGNATURE_HEADER) else {
        return Err(Report::new(SigError::MissingSignatureHeader));
    };

    let signature_header = header::parse(
        sig_header_value
            .to_str()
            .change_context(SigError::InvalidSignatureHeader)?,
    )
    .change_context(SigError::InvalidSignatureHeader)?;
    let encoded_signature = signature_header.signature.to_string();

    let signature_string =
        build_signature_string(req, &signature_header).change_context(SigError::BuildSigString)?;

    let pem_key = get_key(signature_header.key_id)
        .await
        .map_err(|err| Report::new(SigError::GetKey(err.into())))?;
    let public_key = crypto::parse::public_key(&pem_key).change_context(SigError::InvalidKey)?;

    pd_blocking::crypto(move || {
        crypto::verify(signature_string.as_bytes(), &encoded_signature, &public_key)
    })
    .await
    .change_context(SigError::Blocking)?
    .change_context(SigError::Verify)?;
    Ok(())
}

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Missing header value")]
    MissingHeaderValue,

    #[error("Failed to convert header value to string")]
    StrConversionFailed,
}

/// Build a signature string from a parsed signature header and an HTTP request
pub fn build_signature_string<'a, B, H>(
    request: &Request<B>,
    signature_header: &SignatureHeader<'_, H>,
) -> Result<String, Report<BuildError>>
where
    H: Iterator<Item = &'a str> + Clone,
{
    let mut signature_string = String::new();

    for header in signature_header.headers.clone() {
        match header {
            header @ "(request-target)" => {
                let method = request.method().as_str().to_lowercase();
                let path_and_query = request.uri().path_and_query().map_or_else(
                    || request.uri().path(),
                    |path_and_query| path_and_query.as_str(),
                );

                let _ = writeln!(signature_string, "{header}: {method} {path_and_query}");
            }
            header @ "(created)" => {
                let created = signature_header
                    .created
                    .ok_or(BuildError::MissingHeaderValue)?;

                let _ = writeln!(signature_string, "{header}: {created}");
            }
            header @ "(expires)" => {
                let expires = signature_header
                    .expires
                    .ok_or(BuildError::MissingHeaderValue)?;

                let _ = writeln!(signature_string, "{header}: {expires}");
            }
            header => {
                let header_value = request
                    .headers()
                    .get(header)
                    .ok_or(BuildError::MissingHeaderValue)?
                    .to_str()
                    .change_context(BuildError::StrConversionFailed)?;

                let _ = writeln!(
                    signature_string,
                    "{}: {}",
                    header.to_lowercase(),
                    header_value
                );
            }
        }
    }

    // Remove the last new-line
    signature_string.pop();

    Ok(signature_string)
}
