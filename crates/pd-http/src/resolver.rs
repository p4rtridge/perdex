/// A DNS resolver that uses `hickory_resolver` under the hood.
/// Defaults to using Quad9's public DNS servers over both UDP and TCP.
use std::{net::SocketAddr, sync::Arc, vec::IntoIter};

use futures_util::{FutureExt, future::BoxFuture};
use hickory_resolver::{TokioResolver, config::QUAD9, net::runtime::TokioRuntimeProvider};

pub use hickory_resolver::config::ResolverConfig;
use hyper_util::client::legacy::connect::dns::Name;
use tower::{BoxError, Service};

/// A wrapper around `hickory_resolver::Resolver` that implements `tower::Service<Name>` for DNS resolution.
#[derive(Debug, Clone)]
pub struct Resolver {
    inner: Arc<TokioResolver>,
}

impl Resolver {
    /// Creates a new `ResolverBuilder`
    pub fn builder() -> ResolverBuilder {
        ResolverBuilder::default()
    }
}

impl Service<Name> for Resolver {
    type Error = BoxError;
    type Response = ResolveIter;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, req: Name) -> Self::Future {
        let dns_client = self.inner.clone();

        async move {
            let lookup_ips = dns_client.lookup_ip(req.as_str()).await?;

            let addrs = lookup_ips
                .iter()
                .map(|ip| SocketAddr::new(ip, 0))
                .collect::<Vec<SocketAddr>>()
                .into_iter();
            Ok(ResolveIter { inner: addrs })
        }
        .boxed()
    }
}

pub struct ResolveIter {
    inner: IntoIter<SocketAddr>,
}

impl Iterator for ResolveIter {
    type Item = SocketAddr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// A builder for `Resolver`
#[derive(Debug, Default)]
pub struct ResolverBuilder {
    config: Option<ResolverConfig>,
}

impl ResolverBuilder {
    /// Sets the resolver configuration to use for the resolver.
    ///
    /// If not set, defaults to using Quad9's public DNS servers over both UDP and TCP.
    #[must_use]
    #[inline]
    pub fn config(self, config: ResolverConfig) -> Self {
        Self {
            config: Some(config),
        }
    }

    /// Builds the `Resolver`
    #[must_use]
    pub fn build(self) -> Resolver {
        let config = self
            .config
            .unwrap_or_else(|| ResolverConfig::udp_and_tcp(&QUAD9));
        let tokio_runtime = TokioRuntimeProvider::default();

        let resolver = hickory_resolver::Resolver::builder_with_config(config, tokio_runtime)
            .build()
            .expect("Failed to build resolver");

        Resolver {
            inner: Arc::new(resolver),
        }
    }
}
