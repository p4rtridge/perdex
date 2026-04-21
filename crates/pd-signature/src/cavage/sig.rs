use std::fmt::Write;
use std::time::SystemTime;

use error_stack::{Report, ResultExt};
use http::{HeaderValue, Method, Request, header::DATE};

use crate::{
    cavage::{SignatureHeader, serialize},
    crypto,
};

const GET_HEADERS: &[&str] = &["host", "date"];
const POST_HEADERS: &[&str] = &["host", "date", "content-type", "digest"];

#[derive(Debug, thiserror::Error)]
pub enum SigError {
    #[error("Invalid key")]
    InvalidKey,

    #[error("Failed to create signature string")]
    CreateSigString,

    #[error("Unsupported HTTP method")]
    UnsupportedHttpMethod,
}

pub fn sign<B>(
    mut req: Request<B>,
    key_id: &str,
    key: &[u8],
) -> Result<Request<B>, Report<SigError>> {
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

    let key = crypto::parse::private_key(key).change_context(SigError::InvalidKey)?;
    let signature_string = create_sig_string(&req, &signature_header)?;
    // TODO: Send to seperated thread to avoid blocking the main thread
    let signature = crypto::sign(signature_string.as_bytes(), &key);

    let signature_header = SignatureHeader {
        key_id: signature_header.key_id,
        headers: signature_header.headers,
        signature: &signature,
        created: signature_header.created,
        expires: signature_header.expires,
    };
    let signature_header_value = HeaderValue::from_str(&serialize::serialize(signature_header))
        .expect("Failed to serialize signature header");

    req.headers_mut()
        .insert("Signature", signature_header_value);
    Ok(req)
}

/// Create a new signature string from a parsed signature header and an HTTP request
pub fn create_sig_string<'a, B, I>(
    request: &http::Request<B>,
    signature_header: &SignatureHeader<'_, I>,
) -> Result<String, Report<SigError>>
where
    I: Iterator<Item = &'a str> + Clone,
{
    let mut signature_string = String::new();

    for name in signature_header.headers.clone() {
        match name {
            name @ "(request-target)" => {
                let method = request.method().as_str().to_lowercase();
                let path_and_query = request.uri().path_and_query().map_or_else(
                    || request.uri().path(),
                    |path_and_query| path_and_query.as_str(),
                );

                let _ = writeln!(signature_string, "{name}: {method} {path_and_query}");
            }
            name @ "(created)" => {
                let created = signature_header.created.ok_or(SigError::CreateSigString)?;
                let _ = writeln!(signature_string, "{name}: {created}");
            }
            name @ "(expires)" => {
                let expires = signature_header.expires.ok_or(SigError::CreateSigString)?;
                let _ = writeln!(signature_string, "{name}: {expires}");
            }
            header => {
                let value = request
                    .headers()
                    .get(header)
                    .ok_or(SigError::CreateSigString)?
                    .to_str()
                    .change_context(SigError::CreateSigString)?;

                let _ = writeln!(signature_string, "{}: {}", header.to_lowercase(), value);
            }
        }
    }

    // Remove the last new-line
    signature_string.pop();

    Ok(signature_string)
}
