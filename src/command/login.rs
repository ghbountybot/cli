use eyre::Result;

pub async fn handle(client_id: &str) -> Result<()> {
    let token = crate::github::login(client_id).await?;
    // Update config with the new token
    let mut config = crate::config::Config::load()?;
    config.set_github_token(token)?;
    Ok(())
}
