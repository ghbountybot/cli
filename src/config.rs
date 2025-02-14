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

    /// Try to get the GitHub token from config file or environment
    /// Returns None if no token is found
    #[must_use]
    pub fn try_get_github_token(&self) -> Option<String> {
        self.github_token
            .clone()
            .or_else(|| std::env::var("GITHUB_TOKEN").ok())
    }

    /// Get the GitHub token, returning an error if not found
    pub fn get_github_token(&self) -> Result<String> {
        self.try_get_github_token()
            .ok_or_else(|| eyre::eyre!("GitHub token not found in config file or environment"))
    }
}

fn config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| eyre::eyre!("Could not determine home directory"))?;

    Ok(home.join(".config").join("bountybot").join("config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_github_token_from_config() {
        let config = Config {
            github_token: Some("config-token".to_string()),
        };

        assert_eq!(config.get_github_token().unwrap(), "config-token");
    }
}
