use eyre::{Context, Result};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, Layer, Registry, layer::SubscriberExt};

pub fn init() -> Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    let subscriber = Registry::default()
        .with(tracing_subscriber::fmt::layer().with_filter(env_filter));
    
    tracing::subscriber::set_global_default(subscriber).wrap_err("Failed to set global default subscriber")?;
    
    Ok(())
}