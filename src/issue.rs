use crate::bountybot::BountyBotClient;
use eyre::Result;
use inquire::{Select, Text};

/// Prompts the user to select or enter an issue reference
pub async fn prompt_issue_reference() -> Result<String> {
    let client = BountyBotClient::new();
    let quests = client.fetch_active_quests().await?;

    if quests.is_empty() {
        return Ok(Text::new("Enter the issue reference (e.g., owner/repo#123):").prompt()?);
    }

    println!("🔍 Select an active quest or enter your own:");

    let selection = Select::new("Choose a quest:", quests)
        .with_help_message("↑↓ to move, enter to select, type to filter")
        .with_vim_mode(true);

    match selection.prompt() {
        Ok(selected) => Ok(selected.repo_ref),
        Err(inquire::InquireError::OperationCanceled) => {
            Text::new("Enter the issue reference (e.g., owner/repo#123):")
                .prompt()
                .map_err(Into::into)
        }
        Err(e) => Err(e.into()),
    }
}
