use eyre::Result;
use regex::Regex;
use url::Url;

#[derive(Debug, PartialEq, Eq)]
pub struct RepoIssue {
    pub owner: String,
    pub repo: String,
    pub issue_number: u64,
}

impl RepoIssue {
    fn try_parse_github_url(input: &str) -> Option<Result<Self>> {
        if !input.starts_with("github.com/") {
            return None;
        }
        let url_str = format!("https://{input}");
        Some(
            Url::parse(&url_str)
                .map_err(Into::into)
                .and_then(|url| Self::from_url(&url)),
        )
    }

    #[must_use]
    pub fn full_repo_name(&self) -> String {
        let Self { owner, repo, .. } = self;
        format!("{owner}/{repo}")
    }

    /// Parse a repository issue reference from various formats:
    /// - Full URL: <https://github.com/owner/repo/issues/123>
    /// - Domain URL: github.com/owner/repo/issues/123
    /// - Path only: owner/repo/issues/123
    /// - Short form: owner/repo/123
    /// - Issue reference: owner/repo#123
    pub fn parse(input: &str) -> Result<Self> {
        // Try parsing as full URL first
        if let Ok(url) = Url::parse(input) {
            return Self::from_url(&url);
        }

        // Try parsing as domain URL (add https:// prefix)
        if let Some(result) = Self::try_parse_github_url(input) {
            return result;
        }

        // Try parsing as path or issue reference
        Self::from_path(input)
    }

    fn from_url(url: &Url) -> Result<Self> {
        let path = url.path().trim_start_matches('/');
        Self::from_path(path)
    }

    fn from_path(path: &str) -> Result<Self> {
        // Match patterns like:
        // - owner/repo/issues/123
        // - owner/repo/123
        // - owner/repo#123
        let re = Regex::new(r"^([^/]+)/([^/#]+)(?:/(?:issues/)?|#)(\d+)$")?;

        if let Some(caps) = re.captures(path) {
            let owner = caps[1].to_string();
            let repo = caps[2].to_string();
            let issue_number = caps[3].parse()?;

            Ok(Self {
                owner,
                repo,
                issue_number,
            })
        } else {
            Err(eyre::eyre!("Invalid repository issue reference format"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_url() {
        let input = "https://github.com/ghbountybot/cli/issues/2";
        let result = RepoIssue::parse(input).unwrap();
        assert_eq!(
            result,
            RepoIssue {
                owner: "ghbountybot".to_string(),
                repo: "cli".to_string(),
                issue_number: 2
            }
        );
    }

    #[test]
    fn test_parse_domain_url() {
        let input = "github.com/ghbountybot/cli/issues/2";
        let result = RepoIssue::parse(input).unwrap();
        assert_eq!(
            result,
            RepoIssue {
                owner: "ghbountybot".to_string(),
                repo: "cli".to_string(),
                issue_number: 2
            }
        );
    }

    #[test]
    fn test_parse_path_only() {
        let input = "ghbountybot/cli/issues/2";
        let result = RepoIssue::parse(input).unwrap();
        assert_eq!(
            result,
            RepoIssue {
                owner: "ghbountybot".to_string(),
                repo: "cli".to_string(),
                issue_number: 2
            }
        );
    }

    #[test]
    fn test_parse_short_form() {
        let input = "ghbountybot/cli/2";
        let result = RepoIssue::parse(input).unwrap();
        assert_eq!(
            result,
            RepoIssue {
                owner: "ghbountybot".to_string(),
                repo: "cli".to_string(),
                issue_number: 2
            }
        );
    }

    #[test]
    fn test_parse_issue_reference() {
        let input = "ghbountybot/cli#2";
        let result = RepoIssue::parse(input).unwrap();
        assert_eq!(
            result,
            RepoIssue {
                owner: "ghbountybot".to_string(),
                repo: "cli".to_string(),
                issue_number: 2
            }
        );
    }

    #[test]
    fn test_parse_invalid_input() {
        let inputs = [
            "not-a-url",
            "ghbountybot",
            "ghbountybot/",
            "ghbountybot/cli",
            "ghbountybot/cli/",
            "ghbountybot/cli/issues",
            "ghbountybot/cli/issues/",
            "ghbountybot/cli/issues/abc",
        ];

        for input in inputs {
            assert!(
                RepoIssue::parse(input).is_err(),
                "Expected error for input: {input}"
            );
        }
    }
}
