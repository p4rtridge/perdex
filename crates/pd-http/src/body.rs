use std::{borrow::Cow, fmt, pin::Pin, task::Poll};

use bytes::Bytes;
use futures_util::{StreamExt, TryStream, TryStreamExt, stream::BoxStream};
use http_body::Frame;
use http_body_util::StreamBody;
use pin_project_lite::pin_project;
use tower::BoxError;

pin_project! {
    #[project = BodyProj]
    pub enum Body {
        // Empty body
        Empty,

        // Full body
        Full {
            bytes: Option<Bytes>,
        },

        // Body backed by a `StreamBody`
        Stream {
            #[pin]
            stream: StreamBody<BoxStream<'static, Result<Frame<Bytes>, BoxError>>>,
        },
    }
}

impl Body {
    /// Creates an empty body
    #[inline]
    #[must_use]
    pub fn empty() -> Self {
        Self::Empty
    }

    /// Creates a body from a single chunk of bytes
    #[inline]
    #[must_use]
    pub fn full<B>(bytes: B) -> Self
    where
        B: Into<Bytes>,
    {
        Self::Full {
            bytes: Some(bytes.into()),
        }
    }

    /// Creates a body from a stream of byte frames
    #[inline]
    #[must_use]
    pub fn stream<S>(stream: S) -> Self
    where
        S: TryStream + Send + 'static,
        S::Ok: Into<Bytes>,
        S::Error: Into<BoxError>,
    {
        let stream = stream
            .map_ok(|chunk| Frame::data(chunk.into()))
            .map_err(Into::into)
            .boxed();
        Self::Stream {
            stream: StreamBody::new(stream),
        }
    }
}

impl http_body::Body for Body {
    type Data = Bytes;
    type Error = BoxError;

    #[inline]
    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match self.project() {
            BodyProj::Empty => Poll::Ready(None),
            BodyProj::Full { bytes } => {
                Poll::Ready(bytes.take().map(|bytes| Ok(Frame::data(bytes))))
            }
            BodyProj::Stream { stream } => stream.poll_frame(cx),
        }
    }

    fn is_end_stream(&self) -> bool {
        match self {
            Self::Empty => true,
            Self::Full { bytes } => bytes.is_none(),
            Self::Stream { stream } => stream.is_end_stream(),
        }
    }

    fn size_hint(&self) -> http_body::SizeHint {
        match self {
            Self::Empty => http_body::SizeHint::with_exact(0),
            Self::Full { bytes } => {
                if let Some(bytes) = bytes {
                    http_body::SizeHint::with_exact(bytes.len() as u64)
                } else {
                    http_body::SizeHint::with_exact(0)
                }
            }
            Self::Stream { stream } => stream.size_hint(),
        }
    }
}

impl From<Bytes> for Body {
    #[inline]
    fn from(value: Bytes) -> Self {
        Self::full(value)
    }
}

impl From<Cow<'_, str>> for Body {
    #[inline]
    fn from(value: Cow<'_, str>) -> Self {
        Self::full(value.into_owned())
    }
}

impl From<String> for Body {
    #[inline]
    fn from(value: String) -> Self {
        Self::full(value)
    }
}

impl From<Vec<u8>> for Body {
    #[inline]
    fn from(value: Vec<u8>) -> Self {
        Self::full(value)
    }
}

impl Default for Body {
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

impl fmt::Debug for Body {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(std::any::type_name::<Self>())
            .finish_non_exhaustive()
    }
}
