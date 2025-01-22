use crate::Cli;
use clap::CommandFactory;
use clap_complete::Shell;
use eyre::WrapErr;
use git2::{build::RepoBuilder, BranchType, Cred, RemoteCallbacks, Repository};
use octocrab::Octocrab;
use tracing::{debug, instrument, warn};

#[derive(clap::Subcommand, Debug)]
/// Start of Selection
pub enum Command {
    /// Start working on a bounty by forking repo and creating a branch
    #[command(name = "start", aliases = ["s"])]
    Start {
        /// Repository in format "owner/repo"
        repo: String,
        /// Issue number to work on
        issue_number: u64,
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
        Command::Start { repo, issue_number } => {
            start_bounty(&repo, issue_number, github_token).await?;
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

    println!("🚀 Starting work on bounty for {owner}/{repo}#{issue_number}");
    debug!("initializing GitHub client");

    // Initialize GitHub client
    let octocrab = Octocrab::builder()
        .personal_token(github_token.to_string())
        .build()
        .wrap_err("failed to initialize GitHub client")?;

    println!("🔄 Creating fork of repository...");
    // Create a fork if it doesn't exist
    let fork = octocrab
        .repos(owner, repo)
        .create_fork()
        .send()
        .await
        .wrap_err("failed to create fork")?;

    debug!(?fork.owner, "fork created/exists");
    let fork_owner = fork.owner.unwrap().login.clone();

    // Handle all git operations in a separate sync function
    let repo_path = setup_git_repository(owner, repo, &fork_owner, issue_number, github_token)?;

    // Create draft pull request
    let pr = octocrab
        .pulls(owner, repo)
        .create(
            format!("WIP: Fix #{issue_number}"),
            format!("{fork_owner}:issue-{issue_number}"),
            "main",
        )
        .body(format!(
            "Working on issue #{issue_number}\n\nThis PR is a work in progress."
        ))
        .draft(true)
        .send()
        .await
        .wrap_err("failed to create pull request")?;

    println!("✨ Created branch 'issue-{issue_number}' for issue #{issue_number}");
    println!("📂 Repository cloned to: {}", repo_path.display());
    println!("🔨 Ready to work on https://github.com/{owner}/{repo}/issues/{issue_number}");
    println!("🚀 Draft PR created: {}", pr.html_url.unwrap());

    Ok(())
}

#[instrument(skip(github_token))]
fn setup_git_repository(
    owner: &str,
    repo: &str,
    fork_owner: &str,
    issue_number: u64,
    github_token: &str,
) -> eyre::Result<std::path::PathBuf> {
    // Clone or open local repo
    let current_dir = std::env::current_dir().wrap_err("failed to get current directory")?;
    let repo_path = current_dir.join(repo);

    let repository = if repo_path.exists() {
        debug!(path = ?repo_path, "opening existing repository");
        Repository::open(&repo_path).wrap_err("failed to open existing repository")?
    } else {
        println!("📦 Cloning repository...");
        let fork_url = format!("https://github.com/{fork_owner}/{repo}.git");

        debug!(%fork_url, "using fork URL");

        // Set up authentication callbacks
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
            debug!("authenticating git operation");
            Cred::userpass_plaintext("git", github_token)
        });

        let mut builder = RepoBuilder::new();
        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(callbacks);
        builder.fetch_options(fo);
        builder
            .clone(&fork_url, &repo_path)
            .wrap_err("failed to clone repository")?
    };

    let branch_name = format!("issue-{issue_number}");
    debug!(%branch_name, "setting up branch");

    let head = repository
        .head()
        .wrap_err("failed to get repository HEAD")?;
    let commit = head
        .peel_to_commit()
        .wrap_err("failed to get HEAD commit")?;

    // Create a branch if it doesn't exist
    if repository
        .find_branch(&branch_name, BranchType::Local)
        .is_ok()
    {
        warn!(%branch_name, "branch already exists");
    } else {
        debug!(%branch_name, "creating new branch");
        repository
            .branch(&branch_name, &commit, false)
            .wrap_err("failed to create branch")?;
    }

    // Checkout branch
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

    // Create a simple file to initialize the PR
    let readme_path = repo_path.join("BOUNTY.md");
    std::fs::write(
        &readme_path,
        format!("# Working on issue #{issue_number}\n\nThis PR is a work in progress.\n"),
    )
    .wrap_err("failed to create BOUNTY.md")?;

    // Stage and commit the file
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

    // Push the branch to the fork
    let mut remote = repository
        .find_remote("origin")
        .wrap_err("failed to find remote")?;

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, _username_from_url, _allowed_types| {
        Cred::userpass_plaintext("git", github_token)
    });

    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(callbacks);

    remote
        .push(
            &[&format!(
                "refs/heads/{branch_name}:refs/heads/{branch_name}"
            )],
            Some(&mut push_options),
        )
        .wrap_err("failed to push branch")?;

    Ok(repo_path)
}
