use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use std::io;

mod commands;

use commands::bounty::handle_bounty;

/// CLI tool for managing GitHub bounty workflows
///
/// This tool helps automate the process of working on GitHub issues,
/// including forking repositories and setting up development branches.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The bounty command to execute
    #[command(subcommand)]
    command: Commands,

    /// GitHub personal access token for authentication
    ///
    /// Can be set via `GITHUB_TOKEN` environment variable or passed directly
    /// via `--github-token` flag. The token needs repo permissions.
    #[clap(long, env = "GITHUB_TOKEN")]
    github_token: String,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Bounty-related commands
    #[command(subcommand)]
    Bounty(commands::bounty::Command),

    /// Generate shell completion scripts
    Completion {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

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

    match cli.command {
        Commands::Bounty(cmd) => {
            handle_bounty(cmd, &cli.github_token).await?;
        }
        Commands::Completion { shell } => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            generate(shell, &mut cmd, name, &mut io::stdout());
        }
    }

    Ok(())
}
