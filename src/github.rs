use eyre::{bail, Result, WrapErr};
use octocrab::Octocrab;
use std::time::Duration;
use tokio::time::sleep;
use tracing::instrument;

/// Default GitHub App client ID for bountybot
pub static CLIENT_ID: &str = "Ov23liQIMCvcASsBifc1";

pub struct GitHub {
    client: Octocrab,
    token: String,
}

impl GitHub {
    /// Create a new GitHub client with the given token
    pub fn new(token: &str) -> Result<Self> {
        let client = Octocrab::builder()
            .personal_token(token.to_string())
            .build()
            .wrap_err("failed to initialize GitHub client")?;

        Ok(Self {
            client,
            token: token.to_string(),
        })
    }

    /// Get the GitHub token
    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Fork a repository and return the fork owner's login
    pub async fn fork_repo(&self, owner: &str, repo: &str) -> Result<String> {
        let fork = self
            .client
            .repos(owner, repo)
            .create_fork()
            .send()
            .await
            .wrap_err("failed to create fork")?;

        let fork_owner = fork
            .owner
            .ok_or_else(|| eyre::eyre!("Fork owner not found"))?
            .login;

        Ok(fork_owner)
    }

    /// Get repository information
    pub async fn get_repo_info(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<octocrab::models::Repository> {
        self.client
            .repos(owner, repo)
            .get()
            .await
            .wrap_err("failed to get repository info")
    }

    /// Create a draft pull request
    pub async fn create_draft_pr(
        &self,
        owner: &str,
        repo: &str,
        title: String,
        head: String,
        base: String,
        body: String,
    ) -> Result<octocrab::models::pulls::PullRequest> {
        self.client
            .pulls(owner, repo)
            .create(title, head, base)
            .body(body)
            .draft(true)
            .send()
            .await
            .wrap_err("failed to create pull request")
    }

    /// Check for existing pull requests
    pub async fn find_existing_pr(
        &self,
        owner: &str,
        repo: &str,
        head: &str,
    ) -> Result<Option<octocrab::models::pulls::PullRequest>> {
        let prs = self
            .client
            .pulls(owner, repo)
            .list()
            .head(head)
            .send()
            .await
            .wrap_err("failed to check for existing pull requests")?;

        Ok(prs.items.into_iter().next())
    }
}

/// Handles the GitHub device flow authentication
#[instrument(skip(client_id))]
pub async fn login(client_id: &str) -> Result<String> {
    // Request device code
    let client = reqwest::Client::new();
    let device_code_resp = client
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[("client_id", client_id), ("scope", "public_repo")])
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;

    let device_code = device_code_resp["device_code"]
        .as_str()
        .ok_or_else(|| eyre::eyre!("Device code not found in response"))?;
    let user_code = device_code_resp["user_code"]
        .as_str()
        .ok_or_else(|| eyre::eyre!("User code not found in response"))?;
    let verification_uri = device_code_resp["verification_uri"]
        .as_str()
        .ok_or_else(|| eyre::eyre!("Verification URI not found in response"))?;
    let interval = device_code_resp["interval"].as_u64().unwrap_or(5);

    println!("\nEnter this code at {verification_uri}:\n{user_code}\n");

    poll_for_token(&client, client_id, device_code, interval).await
}

async fn poll_for_token(
    client: &reqwest::Client,
    client_id: &str,
    device_code: &str,
    interval: u64,
) -> Result<String> {
    loop {
        sleep(Duration::from_secs(interval)).await;

        let token_resp = client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&[
                ("client_id", client_id),
                ("device_code", device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;

        // Try to get access token first
        if let Some(access_token) = token_resp["access_token"].as_str() {
            return Ok(access_token.to_string());
        }

        // Handle errors
        let Some(error) = token_resp["error"].as_str() else {
            continue;
        };

        match error {
            "authorization_pending" => {}
            "slow_down" => {
                sleep(Duration::from_secs(5)).await;
            }
            "expired_token" => bail!("Code expired - please try again"),
            "access_denied" => bail!("Access denied by user"),
            _ => bail!("Error: {error}"),
        }
    }
}
