pub mod command;

use clap::Parser;
pub use command::Command;

/// CLI tool for managing GitHub bounty workflows
///
/// This tool helps automate the process of working on GitHub issues,
/// including forking repositories and setting up development branches.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The bounty command to execute
    #[command(subcommand)]
    pub command: Command,

    /// GitHub personal access token for authentication
    ///
    /// Can be set via `GITHUB_TOKEN` environment variable or passed directly
    /// via `--github-token` flag. The token needs repo permissions.
    #[clap(long, env = "GITHUB_TOKEN")]
    pub github_token: String,
}
