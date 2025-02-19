use crate::{GitHub, CLIENT_ID};
use inquire::Confirm;
use owo_colors::OwoColorize;
use tracing::debug;

mod completion;
mod login;
mod solve;
mod start;

fn print_step(text: &str) {
    println!("\n{}", text.bold().bright_blue());
}

fn print_success(text: &str) {
    println!("\n{}", text.bright_green());
}

#[derive(clap::Subcommand, Debug)]
#[command(about = "A CLI tool for managing GitHub bounties")]
pub enum Command {
    /// 🚀 Start working on a bounty by forking the repository and creating a branch
    ///
    /// This is the main command you'll use to begin working on a bounty.
    /// It will:
    /// 1. Fork the repository
    /// 2. Create a new branch
    /// 3. Set up a draft PR
    #[command(name = "solve", aliases = ["s"], display_order = 1)]
    #[allow(clippy::doc_markdown)]
    Solve {
        /// Issue reference in any of these formats:
        ///
        /// - https://github.com/owner/repo/issues/123
        ///
        /// - github.com/owner/repo/issues/123
        ///
        /// - owner/repo/issues/123
        ///
        /// - owner/repo/123
        ///
        /// - owner/repo#123
        #[arg(required = false)]
        issue_ref: Option<String>,
    },

    /// 🔧 Generate shell completion scripts
    #[command(name = "completion", aliases = ["c"], display_order = 3)]
    Completion {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },

    /// 🔑 Login to GitHub using device flow
    #[command(name = "login", aliases = ["l"], display_order = 2)]
    Login {
        /// The GitHub App's client ID
        #[arg(long, env = "GITHUB_CLIENT_ID", default_value = CLIENT_ID)]
        client_id: String,
    },
}

impl Command {
    /// Returns true if this command requires authentication
    const fn requires_auth(&self) -> bool {
        matches!(self, Self::Solve { .. })
    }
}

/// Handle the command execution
///
/// # Arguments
/// * `command` - The command to execute
/// * `github_token` - Optional GitHub token for authentication
///
/// # Returns
/// * `eyre::Result<()>` - Result of the command execution
///
/// # Panics
/// * When unwrapping the GitHub token in the Start command, which is safe because we check for token presence before execution
pub async fn handle(command: Command, github_token: Option<&str>) -> eyre::Result<()> {
    debug!(?command, "handling bounty command");

    // Convert input token to owned string if present
    let mut token = github_token.map(String::from);

    // If command requires auth and we don't have a token, trigger login flow
    if command.requires_auth() && token.is_none() {
        println!("This command requires authentication.\n");
        token = Some(crate::github::login(CLIENT_ID).await?);
    }

    match command {
        Command::Solve { issue_ref } => {
            // We can safely unwrap here because we either have a token or would have returned above
            let github_token = token.unwrap();
            let github = GitHub::new(&github_token)?;
            solve::handle(issue_ref, github).await?;
        }
        Command::Completion { shell } => completion::handle(shell)?,
        Command::Login { client_id } => login::handle(&client_id).await?,
    }
    Ok(())
}

/// Handle first-time setup and default command selection
pub async fn handle_default_command() -> eyre::Result<()> {
    let mut config = crate::config::Config::load()?;

    if config.is_first_time() {
        crate::animation::show_welcome_animation();
        print_step("Welcome to BountyBot CLI");

        // Ask about shell completion
        if Confirm::new("Set up fish shell completions?")
            .with_default(true)
            .prompt()?
        {
            completion::handle(clap_complete::Shell::Fish)?;
            print_success("Fish completions installed");
        }

        // Always trigger login on first run
        print_step("Setting up GitHub access");
        let token = crate::github::login(CLIENT_ID).await?;
        config.set_github_token(token)?;

        config.complete_first_time_setup()?;
        print_success("Setup complete - ready to work on bounties!");
    }

    // If no command was provided, try to determine the best action
    if config.try_get_github_token().is_none() {
        // No token, so we should login
        Command::Login {
            client_id: CLIENT_ID.to_string(),
        }
        .handle_command(None)
        .await?;
    } else {
        // We have a token, so show the start command by default
        Command::Solve { issue_ref: None }
            .handle_command(config.try_get_github_token().as_deref())
            .await?;
    }

    Ok(())
}

impl Command {
    /// Handle a single command
    async fn handle_command(self, token: Option<&str>) -> eyre::Result<()> {
        match self {
            Self::Solve { issue_ref } => {
                // We can safely unwrap here because we either have a token or would have returned above
                let github = GitHub::new(token.unwrap())?;
                solve::handle(issue_ref, github).await?;
            }
            Self::Completion { shell } => completion::handle(shell)?,
            Self::Login { client_id } => login::handle(&client_id).await?,
        }
        Ok(())
    }
}
