pub mod command;
pub mod config;
pub mod parse;

use clap::Parser;
pub use command::Command;
pub use parse::RepoIssue;

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
}
