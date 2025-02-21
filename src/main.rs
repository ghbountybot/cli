#![allow(
    clippy::doc_markdown,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::future_not_send,
    clippy::missing_const_for_fn
)]

use bounty::{command, config, Cli};
use clap::Parser;

// Build-time constant for Sentry DSN
const SENTRY_DSN: &str = match option_env!("SENTRY_DSN") {
    Some(dsn) => dsn,
    None => "",
};

fn install_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer().with_target(false).pretty().with_ansi(true);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}

fn main() -> eyre::Result<()> {
    // Set up tracing subscriber with error layer
    install_tracing();

    // Install color-eyre with spantrace support
    color_eyre::install()?;

    // Check if Sentry is disabled at runtime
    let sentry_disabled = std::env::var("DISABLE_SENTRY").is_ok();

    // Initialize Sentry with build-time DSN only if not disabled
    let _guard = if sentry_disabled {
        sentry::init(sentry::ClientOptions::default())
    } else {
        sentry::init((
            SENTRY_DSN,
            sentry::ClientOptions {
                release: sentry::release_name!(),
                default_integrations: true,
                auto_session_tracking: true,
                attach_stacktrace: true,
                ..Default::default()
            },
        ))
    };

    // Create and run the async runtime
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(run())
}

async fn run() -> eyre::Result<()> {
    let cli = Cli::parse();
    let config = config::Config::load()?;

    match cli.command {
        Some(cmd) => command::handle(cmd, config.try_get_github_token().as_deref()).await?,
        None => command::handle_default_command().await?,
    }

    Ok(())
}
