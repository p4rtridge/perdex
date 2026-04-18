use std::fmt;

use tower::BoxError;

/// HttpClient error type
pub struct Error {
    pub inner: BoxError,
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T> From<T> for Error
where
    T: Into<BoxError>,
{
    #[inline]
    fn from(inner: T) -> Self {
        Self {
            inner: inner.into(),
        }
    }
}
