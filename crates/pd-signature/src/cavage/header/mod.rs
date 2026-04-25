use std::time::{Duration, SystemTime};

use error_stack::Report;

mod lexer;
mod serde;

use http::{Method, Request, header::DATE};
pub use serde::{derserialize, serialize};
use thiserror::Error;

use crate::cavage::{get_current_time, is_subset};

const CLOCK_SKEW_ADJUSTMENT: Duration = Duration::from_secs(60); // 1 minute
const MAX_ACCEPTED_SIGNATURE_AGE: Duration = Duration::from_secs(15 * 60); // 15 minutes

const REQUIRED_GET_HEADERS: &[&str] = &["host"];
const REQUIRED_POST_HEADERS: &[&str] = &["host", "content-type", "digest"];

#[derive(Debug, Error)]
pub enum HeaderError {
    #[error("Clock skewed")]
    ClockSkewed,

    #[error("Invalid signature header")]
    InvalidSignatureHeader,

    #[error("Missing required headers in signature")]
    MissingRequiredHeaders,

    #[error("Signature expired")]
    SignatureExpired,

    #[error("Signature too old")]
    SignatureTooOld,

    #[error("Unsupported HTTP method")]
    UnsupportedHttpMethod,
}

/// Struct representation of the `Signature` HTTP header
#[derive(Debug)]
pub struct SignatureHeader<'a, H> {
    /// Unique identifier of the key this request was signed with
    pub key_id: &'a str,

    /// The headers that are part of the signature
    pub headers: H,

    /// The Base64 encoded signature
    pub signature: &'a str,

    /// (Optional) Unix timestamp in seconds when the signature was created
    pub created: Option<u64>,

    /// (Optional) Unix timestamp in seconds when the signature should be considered invalid
    pub expires: Option<u64>,
}

impl<'a, H> SignatureHeader<'a, H> {
    /// Create a new `SignatureHeaderBuilder` to construct a `SignatureHeader`
    pub(crate) fn builder() -> SignatureHeaderBuilder<'a, H> {
        SignatureHeaderBuilder::default()
    }
}

impl<'a, H> SignatureHeader<'a, H>
where
    H: Iterator<Item = &'a str> + Clone,
{
    /// Perform a basic safety check on the signature header and the request
    pub fn validate<B>(&self, req: &Request<B>) -> Result<(), HeaderError> {
        let collected_headers = self.headers.clone().collect::<Vec<_>>();

        let is_subset = match *req.method() {
            Method::GET => is_subset(REQUIRED_GET_HEADERS, &collected_headers),
            Method::POST => is_subset(REQUIRED_POST_HEADERS, &collected_headers),
            _ => return Err(HeaderError::UnsupportedHttpMethod),
        };
        if !is_subset {
            return Err(HeaderError::MissingRequiredHeaders);
        }

        if !collected_headers.contains(&"date") && !collected_headers.contains(&"(created)") {
            return Err(HeaderError::MissingRequiredHeaders);
        }

        // Add a small adjustment to the current time to account for clock skew between the signer and the verifier
        let now = get_current_time();

        if let Some(expires) = self.expires {
            let expiration_time = SystemTime::UNIX_EPOCH
                .checked_add(Duration::from_secs(expires))
                .and_then(|exp| exp.checked_add(CLOCK_SKEW_ADJUSTMENT))
                .ok_or(HeaderError::InvalidSignatureHeader)?;

            let time_until_expiration = expiration_time
                .duration_since(now)
                .map_err(|_| HeaderError::SignatureExpired)?;
            if time_until_expiration > MAX_ACCEPTED_SIGNATURE_AGE {
                return Err(HeaderError::SignatureExpired);
            }
        }

        if let Some(created) = self.created {
            let created_time = SystemTime::UNIX_EPOCH
                .checked_add(Duration::from_secs(created))
                .ok_or(HeaderError::InvalidSignatureHeader)?;

            if created_time > now + CLOCK_SKEW_ADJUSTMENT {
                return Err(HeaderError::ClockSkewed);
            }
            if now.duration_since(created_time).unwrap_or_default() > MAX_ACCEPTED_SIGNATURE_AGE {
                return Err(HeaderError::SignatureTooOld);
            }
        }

        if let Some(date_header) = req.headers().get(DATE) {
            let date_header_time = httpdate::parse_http_date(
                date_header
                    .to_str()
                    .map_err(|_| HeaderError::InvalidSignatureHeader)?,
            )
            .map_err(|_| HeaderError::InvalidSignatureHeader)?;

            if date_header_time > now + CLOCK_SKEW_ADJUSTMENT {
                return Err(HeaderError::ClockSkewed);
            }
            if now.duration_since(date_header_time).unwrap_or_default() > MAX_ACCEPTED_SIGNATURE_AGE
            {
                return Err(HeaderError::SignatureTooOld);
            }
        }

        Ok(())
    }
}

/// Signature header builder error
#[derive(Debug, thiserror::Error)]
pub(crate) enum SignatureHeaderBuilderError {
    /// Missing field
    #[error("Missing field: {0}")]
    MissingField(&'static str),
}

#[derive(Debug)]
pub(crate) struct SignatureHeaderBuilder<'a, H> {
    key_id: Option<&'a str>,
    headers: Option<H>,
    signature: Option<&'a str>,
    created: Option<u64>,
    expires: Option<u64>,
}

impl<'a, H> SignatureHeaderBuilder<'a, H> {
    /// Build the `SignatureHeader` from the provided values
    pub fn build(self) -> Result<SignatureHeader<'a, H>, Report<SignatureHeaderBuilderError>> {
        let key = self
            .key_id
            .ok_or_else(|| Report::new(SignatureHeaderBuilderError::MissingField("keyId")))?;
        let headers = self
            .headers
            .ok_or_else(|| Report::new(SignatureHeaderBuilderError::MissingField("headers")))?;
        let signature = self
            .signature
            .ok_or_else(|| Report::new(SignatureHeaderBuilderError::MissingField("signature")))?;

        Ok(SignatureHeader {
            key_id: key,
            headers,
            signature,
            created: self.created,
            expires: self.expires,
        })
    }

    pub fn key_id(&mut self, key_id: &'a str) {
        self.key_id = Some(key_id);
    }

    pub fn headers(&mut self, headers: H) {
        self.headers = Some(headers);
    }

    pub fn signature(&mut self, signature: &'a str) {
        self.signature = Some(signature);
    }

    pub fn created(&mut self, created: u64) {
        self.created = Some(created);
    }

    pub fn expires(&mut self, expires: u64) {
        self.expires = Some(expires);
    }
}

impl<'a, H> Default for SignatureHeaderBuilder<'a, H> {
    fn default() -> Self {
        Self {
            key_id: None,
            headers: None,
            signature: None,
            created: None,
            expires: None,
        }
    }
}
