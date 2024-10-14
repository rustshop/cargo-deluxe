use std::env;

use color_eyre::eyre::Result;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

pub fn init_tracing() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        // Use the environment variable, if set, falling back to the specified level if not
        .with_env_filter(EnvFilter::new(
            env::var(tracing_subscriber::EnvFilter::DEFAULT_ENV).unwrap_or_else(|_| "info".into()),
        ))
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
