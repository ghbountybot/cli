use eyre::Result;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Issue {
    number: i32,
    title: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Quest {
    organization: String,
    repository: String,
    issue: Issue,
}

#[derive(Debug, Serialize, Deserialize)]
struct QuestResponse {
    #[serde(rename = "activeQuests")]
    active_quests: Vec<Quest>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GraphQLResponse {
    data: QuestResponse,
}

/// A quest issue that can be selected by the user
#[derive(Debug)]
pub struct QuestIssue {
    pub title: String,
    pub repo_ref: String,
}

impl std::fmt::Display for QuestIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.title.bold(), self.repo_ref.dimmed(),)
    }
}

/// Client for interacting with the BountyBot API
#[derive(Default)]
pub struct BountyBotClient {
    client: reqwest::Client,
}

impl BountyBotClient {
    /// Create a new BountyBot API client
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Fetch active quests from the BountyBot API
    pub async fn fetch_active_quests(&self) -> Result<Vec<QuestIssue>> {
        // language=graphql
        let query = r"
            query {
                activeQuests(limit: 50) {
                    organization
                    repository
                    issue {
                        number
                        title
                    }
                }
            }
        ";

        let response = self
            .client
            .post("https://bountybot.dev/api/graphql")
            .json(&serde_json::json!({
                "query": query,
                "variables": {}
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<GraphQLResponse>()
            .await?;

        Ok(response
            .data
            .active_quests
            .into_iter()
            .map(|quest| QuestIssue {
                title: quest.issue.title,
                repo_ref: format!(
                    "{}/{}#{}",
                    quest.organization, quest.repository, quest.issue.number
                ),
            })
            .collect())
    }
}
