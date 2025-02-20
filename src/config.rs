use eyre::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    pub github_token: Option<String>,
    pub has_completed_first_time_setup: bool,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = config_path()?;

        // If the config file exists, try to read it
        let config = if config_path.exists() {
            let contents = std::fs::read_to_string(&config_path)?;
            toml::from_str(&contents)?
        } else {
            Self::default()
        };

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = config_path()?;
        let config_dir = config_path.parent().unwrap();
        std::fs::create_dir_all(config_dir)?;

        let contents = toml::to_string_pretty(self)?;
        std::fs::write(config_path, contents)?;
        Ok(())
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

    /// Set the GitHub token in the config
    pub fn set_github_token(&mut self, token: String) -> Result<()> {
        self.github_token = Some(token);
        self.save()
    }

    /// Mark first-time setup as completed
    pub fn complete_first_time_setup(&mut self) -> Result<()> {
        self.has_completed_first_time_setup = true;
        self.save()
    }

    /// Check if this is the first time running the CLI
    #[must_use]
    pub const fn is_first_time(&self) -> bool {
        !self.has_completed_first_time_setup
    }
}

fn config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| eyre::eyre!("Could not determine home directory"))?;
    Ok(home.join(".config").join("bounty").join("config.toml"))
}
