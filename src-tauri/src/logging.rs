use std::path::{Path, PathBuf};

use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tracing::instrument(fields(level = level, file_logging = log_file.is_some()))]
pub fn init_logging(level: &str, log_file: Option<PathBuf>) -> anyhow::Result<()> {
    let filter = EnvFilter::try_new(level).unwrap_or_else(|_| EnvFilter::new("info"));

    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_ansi(true)
        .compact();

    if let Some(path) = log_file {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let dir = path.parent().unwrap_or(Path::new("."));
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("oxban.log");

        let appender = tracing_appender::rolling::daily(dir, filename);

        let file_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_file(true)
            .with_line_number(true)
            .with_thread_ids(true)
            .with_ansi(false)
            .compact()
            .with_writer(appender);

        tracing_subscriber::registry()
            .with(filter)
            .with(stdout_layer)
            .with(file_layer)
            .try_init()?;
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(stdout_layer)
            .try_init()?;
    }

    Ok(())
}
