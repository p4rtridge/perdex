use error_stack::Report;

pub mod parse;
mod serialize;
pub mod sig;

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
    pub(crate) fn builder() -> SignatureHeaderBuilder<'a, H> {
        SignatureHeaderBuilder::default()
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
