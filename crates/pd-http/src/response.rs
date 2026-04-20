use async_stream::try_stream;
use bytes::{Bytes, BytesMut};
use error_stack::{ResultExt, bail};
use futures_util::{StreamExt, stream::BoxStream};
use http::{HeaderMap, StatusCode, Version};
use serde::de::DeserializeOwned;

use crate::{Error, Result};

const INITIAL_BUFFER_SIZE: usize = 8 * 1024; // 8 KiB

/// A Response to a submitted `Request`.
pub struct Response {
    inner: reqwest::Response,
    body_limit: Option<usize>,
}

impl Response {
    #[must_use]
    pub fn new(inner: reqwest::Response, body_limit: Option<usize>) -> Self {
        Self { inner, body_limit }
    }

    /// Get the `StatusCode` of this `Response`.
    #[inline]
    pub fn status(&self) -> StatusCode {
        self.inner.status()
    }

    /// Get the HTTP `Version` of this `Response`.
    #[inline]
    pub fn version(&self) -> Version {
        self.inner.version()
    }

    /// Get the `Headers` of this `Response`.
    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }

    /// Get a mutable reference to the `Headers` of this `Response`.
    #[inline]
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        self.inner.headers_mut()
    }

    /// Get the content length of the response, if it is known.
    ///
    /// This value does not directly represents the value of the `Content-Length`
    /// header, but rather the size of the response's body. To read the header's
    /// value, please use the [`Response::headers`] method instead.
    ///
    /// Reasons it may not be known:
    ///
    /// - The response does not include a body (e.g. it responds to a `HEAD`
    ///   request).
    /// - The response is gzipped and automatically decoded (thus changing the
    ///   actual decoded length).
    #[inline]
    pub fn content_length(&self) -> Option<u64> {
        self.inner.content_length()
    }

    /// Get the full response body as `Bytes`.
    pub async fn bytes(mut self) -> Result<Bytes> {
        let mut validator = BodyLimit::new(self.content_length(), self.body_limit)?;

        let mut bytes = BytesMut::with_capacity(INITIAL_BUFFER_SIZE);
        while let Some(chunk) = self
            .inner
            .chunk()
            .await
            .change_context(Error::ResponseReadError)?
        {
            validator.validate(chunk.len())?;
            bytes.extend_from_slice(&chunk);
        }
        Ok(bytes.freeze())
    }

    /// Stream a chunk of the response body.
    pub async fn stream(self) -> BoxStream<'static, Result<Bytes>> {
        let inner = self.inner;
        let limit = self.body_limit;

        try_stream! {
            let mut validator = BodyLimit::new(inner.content_length(), limit)?;
            let mut stream = inner.bytes_stream();

            while let Some(chunk) = stream.next().await {
                let chunk = chunk.change_context(Error::ResponseReadError)?;
                validator.validate(chunk.len())?;
                yield chunk;
            }
        }
        .boxed()
    }

    /// Read the body and attempt to interpret it as a UTF-8 encoded string
    pub async fn text(self) -> Result<String> {
        let bytes = self.bytes().await?;
        let text = simdutf8::basic::from_utf8(&bytes)
            .change_context(Error::ResponseReadError)?
            .to_string();
        Ok(text)
    }

    /// Try to deserialize the response body as JSON.
    pub async fn json<T>(self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let bytes = self.bytes().await?;
        sonic_rs::from_slice(&bytes).change_context(Error::ResponseReadError)
    }
}

/// A helper struct to enforce body size limits on responses.
#[derive(Debug)]
struct BodyLimit {
    limit: Option<usize>,
    bytes_read: usize,
}

impl BodyLimit {
    fn new(content_length: Option<u64>, limit: Option<usize>) -> Result<Self> {
        if let Some(limit) = limit
            && let Some(content_length) = content_length
            && content_length > limit as u64
        {
            bail!(Error::BodyLimitExceeded(limit));
        }

        Ok(Self {
            limit,
            bytes_read: 0,
        })
    }

    fn validate(&mut self, chunk_size: usize) -> Result<()> {
        if let Some(limit) = self.limit {
            self.bytes_read += chunk_size;
            if self.bytes_read > limit {
                bail!(Error::BodyLimitExceeded(limit));
            }
        }
        Ok(())
    }
}
