use async_stream::try_stream;
use bytes::{Buf, Bytes};
use futures_util::{Stream, StreamExt};
use http::{Extensions, HeaderMap, StatusCode, Uri, Version};
use http_body_util::{BodyExt, BodyStream};
use hyper::Response as HyperResponse;
use pd_ap_type::jsonld::RdfNode;
use serde::de::DeserializeOwned;
use tower::BoxError;
use tower_http::follow_redirect::RequestUri;

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

    /// Read the body and deserialise it as JSON-LD node and verify the returned node's `@id`
    pub async fn jsonld<T>(mut self) -> Result<T>
    where
        T: DeserializeOwned + RdfNode,
    {
        let Some(server_authority) = self
            .extensions_mut()
            .remove()
            .and_then(|RequestUri(uri)| uri.authority().cloned())
        else {
            return Err(HttpError::HeadersRead(BoxError::from(
                "Failed to get server authority",
            )));
        };

        let node = self.json::<T>().await?;
        if let Some(id) = node.id()
            && Uri::try_from(id)
                .map_err(|err| HttpError::JsonldValidation(BoxError::from(err)))?
                .authority()
                .is_none_or(|node_authority| *node_authority == server_authority)
        {
            return Err(HttpError::JsonldValidation(BoxError::from(
                "Authority of `@id` doesn't belong to the originating server",
            )));
        }

        Ok(node)
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

    /// Returns a mutable reference to the associated extensions.
    #[inline]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        self.inner.extensions_mut()
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
}
