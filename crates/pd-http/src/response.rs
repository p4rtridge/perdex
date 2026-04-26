use async_stream::try_stream;
use bytes::{Buf, Bytes};
use futures_util::{Stream, StreamExt};
use http::{Extensions, HeaderMap, StatusCode, Version};
use http_body_util::{BodyExt, BodyStream};
use hyper::Response as HyperResponse;
use serde::de::DeserializeOwned;

use crate::{
    BoxBody,
    error::{HttpError, Result},
};

/// A Response to a submitted `Request`.
pub struct Response {
    inner: HyperResponse<BoxBody>,
}

impl Response {
    /// Creates a new [`Response`] from a [`HyperResponse<BoxBody>`].
    #[inline]
    #[must_use]
    pub fn new(inner: HyperResponse<BoxBody>) -> Self {
        Self { inner }
    }

    /// Consumes the [`Response`] and returns the response body as a [`Bytes`] buffer.
    #[inline]
    pub async fn bytes(self) -> Result<Bytes> {
        Ok(self
            .inner
            .collect()
            .await
            .map_err(HttpError::BodyRead)?
            .to_bytes())
    }

    /// Consumes the [`Response`] and returns the response body as a [`String`].
    #[inline]
    pub async fn text(self) -> Result<String> {
        let bytes = self.bytes().await?;
        simdutf8::basic::from_utf8(&bytes)
            .map(ToOwned::to_owned)
            .map_err(HttpError::TextDecoding)
    }

    /// Consumes the [`Response`] and deserializes the response body as JSON into the specified type `T`.
    #[inline]
    pub async fn json<T>(self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let bytes = self.bytes().await?;
        sonic_rs::from_slice(&bytes).map_err(HttpError::JsonDeserialization)
    }

    /// Consumes the [`Response`] and returns a stream of response body chunks as [`Bytes`].
    pub async fn stream(self) -> impl Stream<Item = Result<Bytes>> {
        let mut body_stream = BodyStream::new(self.inner.into_body());

        try_stream! {
            while let Some(frame) = body_stream.next().await {
                match frame.map_err(HttpError::StreamRead)?.into_data() {
                    Ok(chunk) if chunk.has_remaining() => yield chunk,
                    Ok(..) | Err(..) => continue, // Skip empty chunks and non-data frames
                }
            }
        }
        .boxed()
    }

    /// Get the [`StatusCode`] of this [`Response`].
    #[inline]
    pub fn status(&self) -> StatusCode {
        self.inner.status()
    }

    /// Get the HTTP [`Version`] of this [`Response`].
    #[inline]
    pub fn version(&self) -> Version {
        self.inner.version()
    }

    /// Get the [`HeaderMap`] of this [`Response`].
    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }

    /// Get a mutable reference to the [`HeaderMap`] of this [`Response`].
    #[inline]
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        self.inner.headers_mut()
    }

    /// Returns a mutable reference to the associated extensions.
    #[inline]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        self.inner.extensions_mut()
    }
}
