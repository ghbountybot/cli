use crate::{Cli, RepoIssue};
use clap::CommandFactory;
use clap_complete::Shell;
use eyre::WrapErr;
use git2::{Cred, PushOptions, RemoteCallbacks, Repository};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use octocrab::Octocrab;
use std::time::Duration;
use tracing::{debug, instrument};

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
}

#[instrument(skip(github_token))]
pub async fn handle(command: Command, github_token: &str) -> eyre::Result<()> {
    debug!(?command, "handling bounty command");
    match command {
        Command::Start { issue_ref } => {
            let repo_issue = RepoIssue::parse(&issue_ref)?;
            start_bounty(
                &format!("{}/{}", repo_issue.owner, repo_issue.repo),
                repo_issue.issue_number,
                github_token,
            )
            .await?;
        }
        Command::Completion { shell } => completion(shell),
    }
    Ok(())
}

fn completion(shell: Shell) {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
}

#[instrument(skip(github_token), fields(owner, repo))]
async fn start_bounty(
    repo_full_name: &str,
    issue_number: u64,
    github_token: &str,
) -> eyre::Result<()> {
    let (owner, repo) = repo_full_name
        .split_once('/')
        .ok_or_else(|| eyre::eyre!("Invalid repo format. Expected 'owner/repo'"))?;

    // Set span fields after we have the values
    tracing::Span::current().record("owner", owner);
    tracing::Span::current().record("repo", repo);

    let multi = MultiProgress::new();
    let spinner_style = ProgressStyle::with_template("{spinner:.green} {msg:.bold.dim}")
        .unwrap()
        .tick_chars("⣾⣽⣻⢿⡿⣟⣯⣷");

    let status_pb = multi.add(ProgressBar::new_spinner());
    status_pb.set_style(spinner_style);
    status_pb.enable_steady_tick(Duration::from_millis(80));

    status_pb.set_message(format!(
        "Starting work on bounty for {owner}/{repo}#{issue_number}"
    ));

    debug!("initializing GitHub client");
    let octocrab = Octocrab::builder()
        .personal_token(github_token.to_string())
        .build()
        .wrap_err("failed to initialize GitHub client")?;

    status_pb.set_message("Creating fork of repository...");
    let fork = octocrab
        .repos(owner, repo)
        .create_fork()
        .send()
        .await
        .wrap_err("failed to create fork")?;

    debug!(?fork.owner, "fork created/exists");
    let fork_owner = fork.owner.unwrap().login;
    status_pb.set_message("✓ Fork created successfully");

    // Clone the repository
    let repo_url = format!("https://github.com/{fork_owner}/{repo}.git");
    let temp_dir = tempfile::tempdir()?;
    status_pb.set_message("Cloning repository...");

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, _username_from_url, _allowed_types| {
        Cred::userpass_plaintext("git", github_token)
    });

    let mut fetch_options = git2::FetchOptions::new();
    let mut fetch_callbacks = RemoteCallbacks::new();
    fetch_callbacks.credentials(|_url, _username_from_url, _allowed_types| {
        Cred::userpass_plaintext("git", github_token)
    });
    fetch_options.remote_callbacks(fetch_callbacks);

    let git_repo =
        Repository::clone(&repo_url, temp_dir.path()).wrap_err("failed to clone repository")?;

    // Create and checkout new branch
    let branch_name = format!("issue-{issue_number}");
    status_pb.set_message(format!("Creating branch {branch_name}..."));

    let head = git_repo.head()?.peel_to_commit()?;
    git_repo.branch(&branch_name, &head, false)?;

    // Create empty commit
    let sig = git_repo.signature()?;
    let tree_id = head.tree_id();
    let tree = git_repo.find_tree(tree_id)?;

    git_repo.commit(
        Some(&format!("refs/heads/{branch_name}")),
        &sig,
        &sig,
        &format!("Start work on #{issue_number}"),
        &tree,
        &[&head],
    )?;

    // Push the branch
    status_pb.set_message("Pushing branch...");
    let mut remote = git_repo.find_remote("origin")?;
    let mut push_options = PushOptions::new();
    push_options.remote_callbacks(callbacks);

    remote.push(
        &[&format!(
            "refs/heads/{branch_name}:refs/heads/{branch_name}"
        )],
        Some(&mut push_options),
    )?;

    // Get repository info to find default branch
    status_pb.set_message("Getting repository info...");
    let repo_info = octocrab
        .repos(owner, repo)
        .get()
        .await
        .wrap_err("failed to get repository info")?;

    let default_branch = repo_info
        .default_branch
        .unwrap_or_else(|| "main".to_string());

    // Create draft PR
    status_pb.set_message("Creating draft pull request...");
    let pr = octocrab
        .pulls(owner, repo)
        .create(
            format!("WIP: Fix #{issue_number}"),
            format!("{fork_owner}:{branch_name}"),
            default_branch,
        )
        .draft(true)
        .send()
        .await
        .wrap_err("failed to create pull request")?;

    status_pb.finish_with_message(format!("✨ Ready to work on issue #{issue_number}"));

    // Print final status in a clean way
    println!("\n🔗 Issue: https://github.com/{owner}/{repo}/issues/{issue_number}");
    println!("🌿 Branch: https://github.com/{fork_owner}/{repo}/tree/{branch_name}");
    println!("📝 Pull Request: {}", pr.html_url.unwrap());

    Ok(())
}
