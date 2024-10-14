use std::{env, io};

use color_eyre::eyre::Result;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

pub fn init_tracing() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::new(
            env::var(tracing_subscriber::EnvFilter::DEFAULT_ENV).unwrap_or_else(|_| "info".into()),
        ))
        .with_writer(io::stderr)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
