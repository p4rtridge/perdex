use std::time::Duration;

use error_stack::ResultExt;
use reqwest::Request;

pub use http;
pub use reqwest::Body;

use crate::{
    error::{Error, Result},
    resolver::Resolver,
    response::Response,
};

mod error;
pub mod resolver;
pub mod response;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_BODY_LIMIT: usize = 1024 * 1024; // 1 MiB
const DEFAULT_USER_AGENT: &str = "pd-http/0.1.0"; // TODO: Use actual version

/// A wrapper around `reqwest::Client`
///
/// This is thread-safe and can be cloned because `reqwest::Client` is designed to be shared across threads.
#[derive(Clone)]
pub struct Client {
    client: reqwest::Client,
    body_limit: Option<usize>,
}

impl Client {
    /// Creates a new `HttpClientBuilder`
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
            .user_agent(DEFAULT_USER_AGENT)
            .expect("Failed to init HTTP client builder")
    }

    /// Executes the given HTTP request and returns a `Response``
    pub async fn execute(&self, request: http::Request<Body>) -> Result<Response> {
        let request = Request::try_from(request).change_context(Error::RequestExecution)?;
        let response = self
            .client
            .execute(request)
            .await
            .change_context(Error::RequestExecution)?;
        Ok(Response::new(response, self.body_limit))
    }

    /// Executes the given HTTP request and signs it using HTTP Signatures
    ///
    /// The headers need to include a `Digest` header if it's a POST request.
    ///
    /// # Errors
    ///
    /// - Signing the request failed
    /// - Executing the request failed
    pub async fn execute_signed(
        &self,
        request: http::Request<Body>,
        key_id: &str,
        private_key_der: &[u8],
    ) -> Result<Response> {
        let request = pd_signature::cavage::sig::sign(request, key_id, private_key_der)
            .await
            .change_context(Error::RequestBuild)?;

        self.execute(request).await
    }
}

/// A builder for `HttpClient`
#[derive(Debug)]
pub struct ClientBuilder {
    dns_resolver: Option<Resolver>,
    body_limit: Option<usize>,
    default_headers: http::HeaderMap,
    timeout: Option<Duration>,

    /// This opt should only be used for testing purposes
    accept_invalid_certs: bool,
}

impl ClientBuilder {
    /// Builds the `HttpClient`
    pub fn build(self) -> Result<Client> {
        let mut client_builder = reqwest::Client::builder()
            .tls_danger_accept_invalid_certs(self.accept_invalid_certs)
            .default_headers(self.default_headers);

        if let Some(timeout) = self.timeout {
            client_builder = client_builder.timeout(timeout);
        }

        let resolver = self
            .dns_resolver
            .unwrap_or_else(|| Resolver::builder().build());
        client_builder = client_builder.dns_resolver(resolver);

        let client = client_builder.build().change_context(Error::RequestBuild)?;
        Ok(Client {
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
        K: TryInto<http::header::HeaderName>,
        K::Error: std::error::Error + Send + Sync + 'static,
        V: TryInto<http::header::HeaderValue>,
        V::Error: std::error::Error + Send + Sync + 'static,
    {
        self.default_headers.insert(
            name.try_into()
                .change_context_lazy(|| Error::Other("Failed to convert header name".into()))?,
            value
                .try_into()
                .change_context_lazy(|| Error::Other("Failed to convert header value".into()))?,
        );
        Ok(self)
    }

    /// Sets the `User-Agent` header for all requests made by the client.
    pub fn user_agent<V>(self, value: V) -> Result<Self>
    where
        V: TryInto<http::header::HeaderValue>,
        V::Error: std::error::Error + Send + Sync + 'static,
    {
        self.default_header(http::header::USER_AGENT, value)
    }

    /// Sets a timeout for all requests made by the client
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Configures the client to accept invalid TLS certificates. This should only be used for testing purposes.
    #[must_use]
    pub fn accept_invalid_certs(mut self, accept: bool) -> Self {
        self.accept_invalid_certs = accept;
        self
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            accept_invalid_certs: false,
            dns_resolver: None,
            body_limit: Some(DEFAULT_BODY_LIMIT),
            default_headers: http::HeaderMap::new(),
            timeout: Some(DEFAULT_TIMEOUT),
        }
    }
}
