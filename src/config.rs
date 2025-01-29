use eyre::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub github_token: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = config_path()?;

        // If the config file exists, try to read it
        let config = if config_path.exists() {
            let contents = std::fs::read_to_string(&config_path)?;
            toml::from_str(&contents)?
        } else {
            Self { github_token: None }
        };

        Ok(config)
    }

    pub fn get_github_token(&self) -> Result<String> {
        // Then try environment variable
        let token = self
            .github_token
            .clone()
            .or_else(|| std::env::var("GITHUB_TOKEN").ok())
            .ok_or_else(|| eyre::eyre!("GitHub token not found in config file or environment"))?;

        Ok(token)
    }
}

fn config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| eyre::eyre!("Could not determine home directory"))?;

    Ok(home.join(".config").join("bountybot").join("config.toml"))
}
