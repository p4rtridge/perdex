/// A DNS resolver for `reqwest` that uses `hickory_resolver` under the hood.
/// Defaults to using Quad9's public DNS servers over both UDP and TCP.
use std::sync::Arc;

use hickory_resolver::{TokioResolver, config::QUAD9, net::runtime::TokioRuntimeProvider};

pub use hickory_resolver::config::ResolverConfig;
use reqwest::dns::{self, Resolve};

/// A wrapper around `hickory_resolver::Resolver` that implements `reqwest::dns::Resolve`
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

impl Resolve for Resolver {
    fn resolve(&self, name: dns::Name) -> dns::Resolving {
        let resolver = self.inner.clone();
        Box::pin(async move {
            let lookup = resolver.lookup_ip(name.as_str()).await?;

            let addrs = lookup
                .iter()
                .map(|ip| std::net::SocketAddr::new(ip, 0))
                .collect::<Vec<_>>()
                .into_iter();
            let addrs: dns::Addrs = Box::new(addrs);
            Ok(addrs)
        })
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
