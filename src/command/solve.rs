use crate::{GitHub, RepoIssue};
use eyre::Result;

pub async fn handle(issue_ref: Option<String>, github: GitHub) -> Result<()> {
    let issue_ref = if let Some(issue_ref) = issue_ref {
        issue_ref
    } else {
        crate::issue::prompt_issue_reference().await?
    };

    let repo_issue = RepoIssue::parse(&issue_ref)?;
    super::start::start_bounty(
        &repo_issue.full_repo_name(),
        repo_issue.issue_number,
        github,
    )
    .await
}
