use crate::{Cli, RepoIssue};
use clap::CommandFactory;
use clap_complete::Shell;
use eyre::WrapErr;
use git2::{build::RepoBuilder, BranchType, Cred, RemoteCallbacks, Repository};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use octocrab::Octocrab;
use std::time::Duration;
use tracing::{debug, instrument, warn};

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

    // Single line that gets updated
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
    let fork_owner = fork.owner.unwrap().login.clone();
    status_pb.set_message("✓ Fork created successfully");

    let repo_path = setup_git_repository(
        owner,
        repo,
        &fork_owner,
        issue_number,
        github_token,
        &status_pb,
    )?;

    status_pb.set_message("Getting repository info...");
    let repo_info = octocrab
        .repos(owner, repo)
        .get()
        .await
        .wrap_err("failed to get repository info")?;

    let default_branch = repo_info
        .default_branch
        .unwrap_or_else(|| "master".to_string());
    debug!(%default_branch, "using repository default branch");

    status_pb.set_message("Checking for existing pull request...");
    // First check if PR already exists
    let existing_prs = octocrab
        .pulls(owner, repo)
        .list()
        .head(format!("{fork_owner}:issue-{issue_number}"))
        .send()
        .await
        .wrap_err("failed to list pull requests")?;

    let pr = if let Some(existing_pr) = existing_prs.items.into_iter().next() {
        debug!("Found existing PR");
        status_pb.set_message("✓ Found existing pull request");
        existing_pr
    } else {
        status_pb.set_message("Creating draft pull request...");
        octocrab
            .pulls(owner, repo)
            .create(
                format!("WIP: Fix #{issue_number}"),
                format!("{fork_owner}:issue-{issue_number}"),
                default_branch,
            )
            .body(format!(
                "Working on issue #{issue_number}\n\nThis PR is a work in progress."
            ))
            .draft(true)
            .send()
            .await
            .wrap_err("failed to create pull request")?
    };

    status_pb.finish_with_message(format!("✨ Ready to work on issue #{issue_number}"));

    let pr_url = pr.html_url.unwrap();

    // Print final status in a clean way
    println!("\n� Repository: {}", repo_path.display());
    println!("�🔗 Issue: https://github.com/{owner}/{repo}/issues/{issue_number}");
    println!("🚀 Pull Request: {pr_url}");

    // Open PR in browser
    if let Err(e) = open::that(pr_url.as_str()) {
        warn!("Failed to open PR URL in browser: {}", e);
    }

    Ok(())
}

#[instrument(skip(github_token))]
fn setup_git_repository(
    owner: &str,
    repo: &str,
    fork_owner: &str,
    issue_number: u64,
    github_token: &str,
    status_pb: &ProgressBar,
) -> eyre::Result<std::path::PathBuf> {
    let current_dir = std::env::current_dir().wrap_err("failed to get current directory")?;
    let repo_path = current_dir.join(repo);

    let repository = if repo_path.exists() {
        debug!(path = ?repo_path, "opening existing repository");
        status_pb.set_message("Opening existing repository...");
        let repo = Repository::open(&repo_path).wrap_err("failed to open existing repository")?;
        status_pb.set_message("✓ Opened existing repository");
        repo
    } else {
        status_pb.set_message("Cloning repository...");
        let fork_url = format!("https://github.com/{fork_owner}/{repo}.git");

        debug!(%fork_url, "using fork URL");

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
            debug!("authenticating git operation");
            Cred::userpass_plaintext("git", github_token)
        });

        let mut builder = RepoBuilder::new();
        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(callbacks);
        builder.fetch_options(fo);
        let repo = builder
            .clone(&fork_url, &repo_path)
            .wrap_err("failed to clone repository")?;
        status_pb.set_message("✓ Repository cloned successfully");
        repo
    };

    // Set the remote URL to the fork
    let mut remote = repository
        .find_remote("origin")
        .wrap_err("failed to find remote")?;
    let fork_url = format!("https://github.com/{fork_owner}/{repo}.git");
    remote
        .url()
        .map(|url| {
            if url != fork_url {
                debug!("updating remote URL to fork");
                repository
                    .remote_set_url("origin", &fork_url)
                    .wrap_err("failed to update remote URL")?;
            }
            Ok::<_, eyre::Report>(())
        })
        .transpose()?;

    status_pb.set_message("Setting up branch...");
    let branch_name = format!("issue-{issue_number}");
    debug!(%branch_name, "setting up branch");

    let head = repository
        .head()
        .wrap_err("failed to get repository HEAD")?;
    let commit = head
        .peel_to_commit()
        .wrap_err("failed to get HEAD commit")?;

    if repository
        .find_branch(&branch_name, BranchType::Local)
        .is_ok()
    {
        warn!(%branch_name, "branch already exists");
        status_pb.set_message("✓ Using existing branch");
    } else {
        debug!(%branch_name, "creating new branch");
        repository
            .branch(&branch_name, &commit, false)
            .wrap_err("failed to create branch")?;
        status_pb.set_message("✓ Created new branch");
    }

    debug!(%branch_name, "checking out branch");
    let obj = repository
        .revparse_single(&branch_name)
        .wrap_err("failed to find branch reference")?;
    repository
        .checkout_tree(&obj, None)
        .wrap_err("failed to checkout branch")?;
    repository
        .set_head(&format!("refs/heads/{branch_name}"))
        .wrap_err("failed to update HEAD")?;

    status_pb.set_message("Initializing PR...");

    let readme_path = repo_path.join("BOUNTY.md");
    std::fs::write(
        &readme_path,
        format!("# Working on issue #{issue_number}\n\nThis PR is a work in progress.\n"),
    )
    .wrap_err("failed to create BOUNTY.md")?;

    let mut index = repository
        .index()
        .wrap_err("failed to get repository index")?;
    index
        .add_path(std::path::Path::new("BOUNTY.md"))
        .wrap_err("failed to stage BOUNTY.md")?;
    index.write().wrap_err("failed to write index")?;

    let tree_id = index.write_tree().wrap_err("failed to write tree")?;
    let tree = repository
        .find_tree(tree_id)
        .wrap_err("failed to find tree")?;
    let signature = repository.signature().wrap_err("failed to get signature")?;
    let parent_commit = repository
        .head()
        .wrap_err("failed to get HEAD")?
        .peel_to_commit()
        .wrap_err("failed to get HEAD commit")?;

    repository
        .commit(
            Some("HEAD"),
            &signature,
            &signature,
            &format!("Initialize PR for issue #{issue_number}"),
            &tree,
            &[&parent_commit],
        )
        .wrap_err("failed to create commit")?;

    let mut fetch_callbacks = RemoteCallbacks::new();
    fetch_callbacks.credentials(|_url, _username_from_url, _allowed_types| {
        debug!("authenticating git fetch");
        Cred::userpass_plaintext("git", github_token)
    });

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(fetch_callbacks);

    debug!("fetching from remote");
    remote
        .fetch(&[] as &[&str], Some(&mut fetch_options), None)
        .wrap_err("failed to fetch from remote")?;

    let mut push_callbacks = RemoteCallbacks::new();
    push_callbacks.credentials(|_url, _username_from_url, _allowed_types| {
        debug!("authenticating git push");
        Cred::userpass_plaintext("git", github_token)
    });

    status_pb.set_message("Pushing changes...");
    let refspec = format!("+refs/heads/{branch_name}:refs/heads/{branch_name}");
    debug!(%refspec, "attempting to force push branch");

    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(push_callbacks);

    remote
        .push(&[&refspec], Some(&mut push_options))
        .wrap_err("failed to push branch")?;

    status_pb.set_message("✓ Changes pushed successfully");
    status_pb.finish_with_message("✓ PR initialized");

    Ok(repo_path)
}
