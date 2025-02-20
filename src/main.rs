#![allow(
    clippy::doc_markdown,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::future_not_send,
    clippy::missing_const_for_fn
)]

use bounty::{command, config, Cli};
use clap::Parser;

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

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Set up tracing subscriber with error layer
    install_tracing();

    // Install color-eyre with spantrace support
    color_eyre::install()?;

    run().await
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
