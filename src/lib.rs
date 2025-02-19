#![allow(
    clippy::doc_markdown,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::future_not_send,
    clippy::missing_const_for_fn
)]

use clap::Parser;

mod animation;
mod bountybot;
pub mod command;
pub mod config;
mod github;
mod issue;
mod parse;

pub use animation::show_welcome_animation;
pub use bountybot::{BountyBotClient, QuestIssue};
pub use command::{handle, handle_default_command, Command};
pub use config::Config;
pub use github::{login, GitHub, CLIENT_ID};
pub use issue::prompt_issue_reference;
pub use parse::RepoIssue;

/// 🎯 BountyBot `CLI` - Streamline your GitHub bounty workflow
///
/// A powerful tool that helps automate the process of working on GitHub issues,
/// including forking repositories and setting up development branches.
///
/// Start by running: `bounty start <issue-url>`
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(styles = get_styles())]
pub struct Cli {
    /// The bounty command to execute
    #[command(subcommand)]
    pub command: Option<Command>,
}

fn get_styles() -> clap::builder::Styles {
    use clap::builder::Styles;
    Styles::styled()
        .usage(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::AnsiColor::BrightBlue.into())),
        )
        .header(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::AnsiColor::BrightYellow.into())),
        )
        .literal(anstyle::Style::new().fg_color(Some(anstyle::AnsiColor::BrightGreen.into())))
        .placeholder(anstyle::Style::new().fg_color(Some(anstyle::AnsiColor::BrightBlue.into())))
}
