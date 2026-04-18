use std::time::Duration;

use http::{HeaderMap, HeaderName, HeaderValue, header::USER_AGENT};
use reqwest::Client;
use tower::BoxError;

pub use reqwest::Request;

use crate::{error::Error, resolver::Resolver, response::Response};

pub mod error;
pub mod resolver;
pub mod response;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_BODY_LIMIT: usize = 1024 * 1024; // 1 MiB
const DEFAULT_USER_AGENT: &str = "pd-http/0.1.0"; // TODO: Use actual version

pub(crate) type Result<T, E = error::Error> = std::result::Result<T, E>;

/// A wrapper around `reqwest::Client` that provides additional features like body size limits and custom DNS resolvers.
pub struct HttpClient {
    client: Client,
    body_limit: Option<usize>,
}

impl HttpClient {
    /// Creates a new `HttpClientBuilder`
    pub fn builder() -> HttpClientBuilder {
        HttpClientBuilder::default()
            .user_agent(DEFAULT_USER_AGENT)
            .expect("Failed to build HTTP client builder")
    }

    /// Executes the given HTTP request and returns a `Response``
    #[inline]
    pub async fn execute(&self, request: Request) -> Result<Response> {
        let response = self.client.execute(request).await?;
        Ok(Response::new(response, self.body_limit))
    }
}

/// A builder for `HttpClient`
#[derive(Debug)]
pub struct HttpClientBuilder {
    dns_resolver: Option<Resolver>,
    body_limit: Option<usize>,
    default_headers: HeaderMap,
    timeout: Option<Duration>,
}

impl HttpClientBuilder {
    /// Builds the `HttpClient`
    pub fn build(self) -> Result<HttpClient> {
        let mut client_builder = Client::builder().default_headers(self.default_headers);

        if let Some(timeout) = self.timeout {
            client_builder = client_builder.timeout(timeout);
        }

        let resolver = self
            .dns_resolver
            .unwrap_or_else(|| Resolver::builder().build());
        client_builder = client_builder.dns_resolver(resolver);

        let client = client_builder.build().map_err(Error::from)?;
        Ok(HttpClient {
            client,
            body_limit: self.body_limit,
        })
    }

    /// Sets a custom DNS resolver for the HTTP client.
    #[must_use]
    pub fn dns_resolver(mut self, resolver: Resolver) -> Self {
        self.dns_resolver = Some(resolver);
        self
    }

    /// Sets a maximum body size limit for HTTP responses. If the response body exceeds this limit, an error will be returned.
    ///
    /// The default body limit is 1 MiB.
    #[must_use]
    pub fn body_limit(mut self, limit: Option<usize>) -> Self {
        self.body_limit = limit;
        self
    }

    /// Adds a default header to be included in all requests made by the client.
    pub fn default_header<K, V>(mut self, name: K, value: V) -> Result<Self>
    where
        K: TryInto<HeaderName>,
        K::Error: Into<BoxError>,
        V: TryInto<HeaderValue>,
        V::Error: Into<BoxError>,
    {
        self.default_headers.insert(
            name.try_into().map_err(Error::from)?,
            value.try_into().map_err(Error::from)?,
        );
        Ok(self)
    }

    /// Sets the `User-Agent` header for all requests made by the client.
    pub fn user_agent<V>(self, value: V) -> Result<Self>
    where
        V: TryInto<HeaderValue>,
        V::Error: Into<BoxError>,
    {
        self.default_header(USER_AGENT, value)
    }

    /// Sets a timeout for all requests made by the client
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

impl Default for HttpClientBuilder {
    fn default() -> Self {
        Self {
            dns_resolver: None,
            body_limit: Some(DEFAULT_BODY_LIMIT),
            default_headers: HeaderMap::new(),
            timeout: Some(DEFAULT_TIMEOUT),
        }
    }
}
