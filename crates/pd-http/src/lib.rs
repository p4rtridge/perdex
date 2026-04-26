use std::error::Error as StdError;
use std::time::Duration;

use bytes::Bytes;
use http::{HeaderMap, Request, StatusCode};
use http_body_util::Limited;
use hyper::{Request as HyperRequest, Response as HyperResponse};
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::{
    client::legacy::connect::HttpConnector,
    {client::legacy::Client as HyperClient, rt::TokioExecutor},
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

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_BODY_LIMIT: usize = 1024 * 1024; // 1 MiB
const DEFAULT_USER_AGENT: &str = "pd-http/0.1.0"; // TODO: Use actual version

/// An HTTP client for making requests to other servers, with support for features like:
/// - Automatic decompression of response bodies
/// - Following redirects
/// - Configurable timeouts
/// - Configurable default headers
/// - Configurable maximum response body size
/// - HTTP Signatures for request signing
#[derive(Clone)]
pub struct Client {
    default_headers: HeaderMap,
    inner: BoxCloneService<HyperRequest<Body>, HyperResponse<BoxBody>, BoxError>,
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

        let ready_svc = self.inner.clone();
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
        request.headers_mut().extend(self.default_headers.clone());
        request
    }
}

/// A builder for `HttpClient`
#[derive(Debug)]
pub struct ClientBuilder {
    dns_resolver: Option<Resolver>,
    body_limit: Option<usize>,
    default_headers: http::HeaderMap,
    timeout: Option<Duration>,
}

impl ClientBuilder {
    /// Build the [`Client`]
    pub fn build(mut self) -> Client {
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
            default_headers: self.default_headers,
            inner: service,
        }
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

    /// Sets a timeout for all requests made by the client
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            dns_resolver: None,
            body_limit: Some(DEFAULT_BODY_LIMIT),
            default_headers: http::HeaderMap::new(),
            timeout: Some(DEFAULT_TIMEOUT),
        }
    }
}
