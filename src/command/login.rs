use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{fs, time::Duration};
use tracing::instrument;

#[instrument(skip(client_id))]
pub async fn login(client_id: &str) -> eyre::Result<String> {
    println!("BountyBot CLI is open source. Check https://github.com/ghbountybot/cli to see how we use your GitHub token.\n");

    let multi = MultiProgress::new();
    let spinner_style = ProgressStyle::with_template("{spinner:.green} {msg:.bold.dim}")
        .unwrap()
        .tick_chars("⣾⣽⣻⢿⡿⣟⣯⣷");

    let status_pb = multi.add(ProgressBar::new_spinner());
    status_pb.set_style(spinner_style);
    status_pb.enable_steady_tick(Duration::from_millis(80));

    status_pb.set_message("Requesting device code...");

    // Request device code
    let client = reqwest::Client::new();
    let device_code_resp = client
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id),
            ("scope", "repo"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;

    let device_code = device_code_resp["device_code"].as_str().unwrap();
    let user_code = device_code_resp["user_code"].as_str().unwrap();
    let verification_uri = device_code_resp["verification_uri"].as_str().unwrap();
    let interval = device_code_resp["interval"].as_u64().unwrap_or(5);

    println!("\nPlease visit: {verification_uri}");
    println!("And enter code: {user_code}\n");

    // Try to open the browser but ignore errors
    let _ = open::that(verification_uri);

    status_pb.set_message("Waiting for device authorization...");

    // Poll for token
    loop {
        tokio::time::sleep(Duration::from_secs(interval)).await;

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

        if let Some(error) = token_resp["error"].as_str() {
            match error {
                "authorization_pending" => continue,
                "slow_down" => {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
                "expired_token" => {
                    status_pb.finish_with_message("❌ Device code expired. Please try again.");
                    return Err(eyre::eyre!("Device code expired"));
                }
                "access_denied" => {
                    status_pb.finish_with_message("❌ Authorization denied by user.");
                    return Err(eyre::eyre!("Authorization denied"));
                }
                _ => {
                    status_pb.finish_with_message("❌ Unknown error occurred.");
                    return Err(eyre::eyre!("Unknown error: {}", error));
                }
            }
        }

        if let Some(access_token) = token_resp["access_token"].as_str() {
            // Save token to config
            let config_dir = dirs::home_dir()
                .ok_or_else(|| eyre::eyre!("Could not determine home directory"))?
                .join(".config")
                .join("bountybot");
            fs::create_dir_all(&config_dir)?;

            let config_path = config_dir.join("config.toml");
            let config_content = format!("github_token = \"{access_token}\"");
            fs::write(config_path, config_content)?;

            status_pb.finish_with_message("✨ Successfully logged in!");
            return Ok(access_token.to_string());
        }
    }
} 