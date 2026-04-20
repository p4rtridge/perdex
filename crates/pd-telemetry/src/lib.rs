use error_stack::{Report, ResultExt};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, Layer, Registry, layer::SubscriberExt};

use crate::error::TelemetryError;

mod error;

pub fn initialize() -> Result<(), Report<TelemetryError>> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    let subscriber =
        Registry::default().with(tracing_subscriber::fmt::layer().with_filter(env_filter));

    tracing::subscriber::set_global_default(subscriber).change_context(TelemetryError)?;

    Ok(())
}
