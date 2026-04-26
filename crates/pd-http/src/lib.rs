use std::time::Duration;
use std::{error::Error as StdError, sync::Arc};

use bytes::Bytes;
use http::{HeaderMap, Request, StatusCode};
use http_body_util::Limited;
use hyper::{Request as HyperRequest, Response as HyperResponse};
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::{
    client::legacy::{Client as HyperClient, connect::HttpConnector},
    rt::{TokioExecutor, TokioTimer},
};
use pd_signature::cavage::sig::SigExt;
use tower::{
    BoxError, Service, ServiceBuilder, ServiceExt,
    layer::util::Identity,
    util::{BoxCloneService, Either},
};
use tower_http::{
    decompression::DecompressionLayer, follow_redirect::FollowRedirectLayer,
    map_response_body::MapResponseBodyLayer, timeout::TimeoutLayer,
};

use crate::{
    error::{HttpError, Result},
    resolver::Resolver,
    response::Response,
};

pub use self::body::Body;

mod body;
mod error;
pub mod resolver;
pub mod response;

pub(crate) type BoxBody<E = BoxError> = http_body_util::combinators::BoxBody<Bytes, E>;

// Keep idle connections low to avoid exhausting file descriptors (ulimit) while still allowing some reuse for performance
const DEFAULT_POOL_IDLE_TIMEOUT: Duration = Duration::from_secs(90);
const DEFAULT_MAX_IDLE_PER_HOST: usize = 32;
const DEFAULT_BODY_LIMIT: usize = 1024 * 1024; // 1 MiB
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30); // Same as firefox
const DEFAULT_USER_AGENT: &str = "pd-http/0.1.0"; // TODO: Use actual version

/// An HTTP client for making requests to other servers, with support for features like:
/// - Automatic decompression of response bodies
/// - Following redirects
/// - Configurable timeouts
/// - Configurable default headers
/// - Configurable maximum response body size
/// - HTTP Signatures for request signing
///
/// [`Client`] is cheap to clone and is designed to be shared across the application.
#[derive(Clone)]
pub struct Client {
    default_headers: Arc<HeaderMap>,
    svc: BoxCloneService<HyperRequest<Body>, HyperResponse<BoxBody>, BoxError>,
}

impl Client {
    /// Creates a new `HttpClientBuilder`
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
            .user_agent(DEFAULT_USER_AGENT)
            .expect("Failed to init HTTP client builder")
    }

    /// Executes the given HTTP request and returns a [`Response`].
    pub async fn execute(&self, request: Request<Body>) -> Result<Response> {
        let request = self.prepare_request(request);

        let ready_svc = self.svc.clone();
        let response = ready_svc
            .oneshot(request)
            .await
            .map_err(HttpError::RequestExecution)?;

        Ok(Response::new(response))
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
        request: Request<Body>,
        key_id: &str,
        private_key_der: &[u8],
    ) -> Result<Response> {
        let request = request
            .sign(key_id, private_key_der)
            .await
            .map_err(|err| HttpError::Signature(err.into()))?;

        self.execute(request).await
    }

    #[inline]
    fn prepare_request(&self, mut request: Request<Body>) -> Request<Body> {
        request.headers_mut().extend(
            self.default_headers
                .iter()
                .map(|(k, v)| (k.clone(), v.clone())),
        );
        request
    }
}

/// A builder for `HttpClient`
#[derive(Debug)]
pub struct ClientBuilder {
    body_limit: Option<usize>,
    default_headers: http::HeaderMap,
    dns_resolver: Option<Resolver>,
    max_idle_per_host: Option<usize>,
    pool_idle_timeout: Option<Duration>,
    timeout: Option<Duration>,
}

impl ClientBuilder {
    /// Build the [`Client`]
    pub fn build(mut self) -> Client {
        let max_idle_per_host = self.max_idle_per_host.unwrap_or(DEFAULT_MAX_IDLE_PER_HOST);

        let dns_resolver = self
            .dns_resolver
            .take()
            .unwrap_or_else(|| Resolver::builder().build());

        let connector = HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_only()
            .enable_all_versions()
            .wrap_connector(HttpConnector::new_with_resolver(dns_resolver));

        let client = HyperClient::builder(TokioExecutor::new())
            .pool_idle_timeout(self.pool_idle_timeout)
            .pool_max_idle_per_host(max_idle_per_host)
            .pool_timer(TokioTimer::new())
            .build(connector)
            .map_response(|res| {
                let (parts, body) = res.into_parts();
                let body = BoxBody::new(body);
                HyperResponse::from_parts(parts, body)
            });

        // Seperate the construction so we can make it mockable in tests
        self.service(client)
    }

    /// Build the HTTP client by wrapping another HTTP client service
    #[must_use]
    pub fn service<S, B>(self, client: S) -> Client
    where
        S: Service<Request<Body>, Response = HyperResponse<B>> + Clone + Send + Sync + 'static,
        S::Error: StdError + Send + Sync + 'static,
        S::Future: Send,
        B: http_body::Body + Default + Send + Sync + 'static,
        B::Data: Send + Sync,
        B::Error: StdError + Send + Sync + 'static,
    {
        let body_limit = self.body_limit.map_or_else(
            || Either::Left(MapResponseBodyLayer::new(BoxBody::new)),
            |limit| {
                Either::Right(MapResponseBodyLayer::new(move |body| {
                    BoxBody::new(Limited::new(body, limit))
                }))
            },
        );

        let timeout = self.timeout.map_or_else(
            || Either::Left(Identity::new()),
            |duration| {
                Either::Right(TimeoutLayer::with_status_code(
                    StatusCode::REQUEST_TIMEOUT,
                    duration,
                ))
            },
        );

        let service = ServiceBuilder::new()
            .layer(body_limit)
            .layer(DecompressionLayer::new())
            .layer(timeout)
            .layer(FollowRedirectLayer::new())
            .service(client)
            .map_err(BoxError::from);
        let service = BoxCloneService::new(service);

        Client {
            default_headers: self.default_headers.into(),
            svc: service,
        }
    }

    /// Sets a maximum body size limit for HTTP responses. If the response body exceeds this limit, an error will be returned.
    ///
    /// The default body limit is 1 MiB.
    #[must_use]
    pub fn body_limit(mut self, limit: Option<usize>) -> Self {
        self.body_limit = limit;
        self
    }

    /// Sets a custom DNS resolver for the HTTP client.
    ///
    /// The default DNS resolver is QUAD9
    #[must_use]
    pub fn dns_resolver(mut self, resolver: Resolver) -> Self {
        self.dns_resolver = Some(resolver);
        self
    }

    /// Adds a default header to be included in all requests made by the client.
    pub fn default_header<K, V>(mut self, name: K, value: V) -> Result<Self>
    where
        K: TryInto<http::header::HeaderName>,
        K::Error: Into<BoxError>,
        V: TryInto<http::header::HeaderValue>,
        V::Error: Into<BoxError>,
    {
        self.default_headers.insert(
            name.try_into()
                .map_err(|err| HttpError::HeaderConversion(err.into()))?,
            value
                .try_into()
                .map_err(|err| HttpError::HeaderConversion(err.into()))?,
        );
        Ok(self)
    }

    /// Sets the `User-Agent` header for all requests made by the client.
    pub fn user_agent<V>(self, value: V) -> Result<Self>
    where
        V: TryInto<http::header::HeaderValue>,
        V::Error: Into<BoxError>,
    {
        self.default_header(http::header::USER_AGENT, value)
    }

    /// Sets the maximum number of idle connections to keep per host in the connection pool.
    ///
    /// The default maximum idle connections per host is 32.
    #[must_use]
    pub fn max_idle_per_host(mut self, max: Option<usize>) -> Self {
        self.max_idle_per_host = max;
        self
    }

    /// Sets a timeout for idle connections in the connection pool. Idle connections will be closed after this duration.
    ///
    /// The default pool idle timeout is 30 seconds.
    #[must_use]
    pub fn pool_idle_timeout(mut self, duration: Option<Duration>) -> Self {
        self.pool_idle_timeout = duration;
        self
    }

    /// Sets a timeout for all requests made by the client
    ///
    /// The default timeout is 30 seconds.
    #[must_use]
    pub fn timeout(mut self, duration: Option<Duration>) -> Self {
        self.timeout = duration;
        self
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            body_limit: Some(DEFAULT_BODY_LIMIT),
            default_headers: http::HeaderMap::new(),
            dns_resolver: None,
            max_idle_per_host: Some(DEFAULT_MAX_IDLE_PER_HOST),
            pool_idle_timeout: Some(DEFAULT_POOL_IDLE_TIMEOUT),
            timeout: Some(DEFAULT_TIMEOUT),
        }
    }
}
