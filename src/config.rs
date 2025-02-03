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
        self.github_token.clone().or_else(|| std::env::var("GITHUB_TOKEN").ok())
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
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_config(content: Option<&str>) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join(".config").join("bountybot");
        fs::create_dir_all(&config_dir).unwrap();
        let config_path = config_dir.join("config.toml");

        if let Some(content) = content {
            fs::write(&config_path, content).unwrap();
        }

        (temp_dir, config_path)
    }

    #[test]
    fn test_load_nonexistent_config() {
        let (temp_dir, _) = setup_test_config(None);
        env::set_var("HOME", temp_dir.path());

        let config = Config::load().unwrap();
        assert!(config.github_token.is_none());
    }

    #[test]
    fn test_load_existing_config() {
        let (temp_dir, _) = setup_test_config(Some(
            r#"
            github_token = "test-token"
        "#,
        ));
        env::set_var("HOME", temp_dir.path());

        let config = Config::load().unwrap();
        assert_eq!(config.github_token, Some("test-token".to_string()));
    }

    #[test]
    fn test_get_github_token_from_config() {
        let config = Config {
            github_token: Some("config-token".to_string()),
        };

        assert_eq!(config.get_github_token().unwrap(), "config-token");
    }

    #[test]
    fn test_get_github_token_from_env() {
        let config = Config { github_token: None };

        env::set_var("GITHUB_TOKEN", "env-token");
        assert_eq!(config.get_github_token().unwrap(), "env-token");
        env::remove_var("GITHUB_TOKEN");
    }

    #[test]
    fn test_get_github_token_missing() {
        let config = Config { github_token: None };

        env::remove_var("GITHUB_TOKEN");
        assert!(config.get_github_token().is_err());
    }
}
