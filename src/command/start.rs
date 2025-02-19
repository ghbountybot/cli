use crate::GitHub;
use eyre::WrapErr;
use git2::{Cred, PushOptions, RemoteCallbacks, Repository};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;
use tracing::{debug, instrument};

#[instrument(skip(github), fields(owner, repo))]
pub async fn start_bounty(
    repo_full_name: &str,
    issue_number: u64,
    github: GitHub,
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

    status_pb.set_message("Creating fork of repository...");
    let fork_owner = github.fork_repo(owner, repo).await?;

    debug!(?fork_owner, "fork created/exists");
    status_pb.set_message("✓ Fork created successfully");

    // Clone the repository
    let repo_url = format!("https://github.com/{fork_owner}/{repo}.git");
    let temp_dir = tempfile::tempdir()?;
    status_pb.set_message("Cloning repository...");

    let mut callbacks = RemoteCallbacks::new();
    let token = github.token();
    callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
        Cred::userpass_plaintext("git", token)
    });

    let mut fetch_options = git2::FetchOptions::new();
    let mut fetch_callbacks = RemoteCallbacks::new();
    let token = github.token();
    fetch_callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
        Cred::userpass_plaintext("git", token)
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
        "Start work on bounty",
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
            "+refs/heads/{branch_name}:refs/heads/{branch_name}"
        )],
        Some(&mut push_options),
    )?;

    // Get repository info to find default branch
    status_pb.set_message("Getting repository info...");
    let repo_info = github.get_repo_info(owner, repo).await?;

    let default_branch = repo_info
        .default_branch
        .unwrap_or_else(|| "main".to_string());

    // Check if PR already exists
    status_pb.set_message("Checking for existing pull requests...");
    let head = format!("{fork_owner}:{branch_name}");
    let pr = if let Some(existing_pr) = github.find_existing_pr(owner, repo, &head).await? {
        status_pb.set_message("Found existing pull request");
        existing_pr
    } else {
        // Create draft PR
        status_pb.set_message("Creating draft pull request...");
        github
            .create_draft_pr(
                owner,
                repo,
                format!("Resolves: #{issue_number}"),
                head,
                default_branch,
                format!(
                    "This pull request resolves issue #{issue_number}.\n\n\
                    Work in progress 🚧\n\n\
                    ## Bounty Guidelines:\n\
                    - Please post your progress at least once per day\n\
                    - This PR will remain in draft status until work is complete\n\
                    - Only mark as ready for review when you have finished your work\n\
                    - You may be unassigned from this issue if there are no pushes for several days\n\
                    \n\
                    ## Community\n\
                    [Join Our Discord](https://discord.gg/jcQpX6jW8u) to connect with other bounty hunters and give feedback.\n\
                    \n\
                    Happy coding! 🎉"
                ),
            )
            .await?
    };

    status_pb.finish_with_message(format!("✨ Ready to work on issue #{issue_number}"));

    // Print final status in a clean way
    println!("\n🔗 Issue: https://github.com/{owner}/{repo}/issues/{issue_number}");
    println!("🌿 Branch: https://github.com/{fork_owner}/{repo}/tree/{branch_name}");
    println!("📝 Pull Request: {}", pr.html_url.unwrap());

    Ok(())
}
