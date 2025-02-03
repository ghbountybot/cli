use crate::{Cli, RepoIssue};
use clap::CommandFactory;
use clap_complete::Shell;
use tracing::debug;

mod login;
mod start;

/// Default GitHub App client ID for bountybot
static CLIENT_ID: &str = "Ov23liQIMCvcASsBifc1";

#[derive(clap::Subcommand, Debug)]
/// Start of Selection
pub enum Command {
    /// Start working on a bounty by forking repo and creating a branch
    #[command(name = "start", aliases = ["s"])]
    Start {
        /// Issue reference in any of these formats:
        /// - <https://github.com/owner/repo/issues/123>
        /// - github.com/owner/repo/issues/123
        /// - owner/repo/issues/123
        /// - owner/repo/123
        /// - owner/repo#123
        issue_ref: String,
    },

    /// Generate shell completion scripts
    #[command(name = "completion", aliases = ["c"])]
    Completion {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Login to GitHub using device flow
    #[command(name = "login", aliases = ["l"])]
    Login {
        /// The GitHub App's client ID
        #[arg(long, env = "GITHUB_CLIENT_ID", default_value = CLIENT_ID)]
        client_id: String,
    },
}

impl Command {
    /// Returns true if this command requires authentication
    const fn requires_auth(&self) -> bool {
        matches!(self, Self::Start { .. })
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
        token = Some(login::login(CLIENT_ID).await?);
    }

    match command {
        Command::Start { issue_ref } => {
            let repo_issue = RepoIssue::parse(&issue_ref)?;
            // We can safely unwrap here because we either have a token or would have returned above
            start::start_bounty(
                &format!("{}/{}", repo_issue.owner, repo_issue.repo),
                repo_issue.issue_number,
                &token.unwrap(),
            )
            .await?;
        }
        Command::Completion { shell } => completion(shell),
        Command::Login { client_id } => {
            let _token = login::login(&client_id).await?;
        }
    }
    Ok(())
}

fn completion(shell: Shell) {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
}
