use anyhow::Result;
use std::path::PathBuf;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

pub fn init_tracing(
    level: &str,
    log_dir: Option<PathBuf>,
) -> Result<Option<tracing_appender::non_blocking::WorkerGuard>> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    let registry = tracing_subscriber::registry().with(filter);

    if let Some(dir) = log_dir {
        std::fs::create_dir_all(&dir)?;
        let file_appender = tracing_appender::rolling::daily(dir, "fetcher.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        let file_layer = fmt::layer().with_ansi(false).with_writer(non_blocking);

        let stdout_layer = fmt::layer().with_ansi(true).with_target(false);

        registry.with(file_layer).with(stdout_layer).init();

        Ok(Some(guard))
    } else {
        registry
            .with(fmt::layer().with_ansi(true).with_target(false))
            .init();
        Ok(None)
    }
}
