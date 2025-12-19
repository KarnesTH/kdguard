use std::{env::consts::OS, process::Command};

use inquire::Confirm;
use lingua_i18n_rs::prelude::Lingua;

use crate::config::Config;

pub struct UpdateManager;

impl UpdateManager {
    /// Check for update
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if successful, otherwise an error
    pub async fn check_update() -> Result<(), Box<dyn std::error::Error>> {
        let latest_tag = Self::get_latest_tag().await?;
        let current_version = env!("CARGO_PKG_VERSION");
        let latest_version = Self::extract_version(&latest_tag);

        if Self::compare_versions(&latest_version, current_version) > 0 {
            let confirm = Confirm::new(&Lingua::t("cli.cli_commands.update.confirm", &[]).unwrap())
                .with_default(false)
                .prompt()?;
            if confirm {
                Self::update()?;
            }
        }

        Ok(())
    }

    /// Get the latest tag from GitHub
    ///
    /// # Returns
    ///
    /// Returns the latest tag if successful, otherwise an error
    async fn get_latest_tag() -> Result<String, Box<dyn std::error::Error>> {
        let client = reqwest::Client::builder()
            .user_agent("kdguard-update-checker")
            .build()?;

        let res = client
            .get("https://api.github.com/repos/KarnesTH/kdguard/releases/latest")
            .send()
            .await?;

        let status = res.status();
        let text = res.text().await?;

        if !status.is_success() {
            return Err(format!("GitHub API returned status {}: {}", status, text).into());
        }

        let json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("Failed to parse JSON response: {}. Response: {}", e, text))?;

        let latest_tag = json
            .get("tag_name")
            .and_then(|v| v.as_str())
            .ok_or("Failed to parse tag_name from GitHub API response")?
            .to_string();
        Ok(latest_tag)
    }

    /// Extract the version from the tag
    ///
    /// # Returns
    ///
    /// Returns the version if successful, otherwise an error
    fn extract_version(tag: &str) -> String {
        tag.strip_prefix('v').unwrap_or(tag).to_string()
    }

    /// Compare two versions
    ///
    /// # Returns
    ///
    /// Returns 1 if v1 is greater than v2, -1 if v1 is less than v2, and 0 if they are equal
    fn compare_versions(v1: &str, v2: &str) -> i32 {
        let v1_parts: Vec<u32> = v1.split('.').map(|s| s.parse().unwrap_or(0)).collect();
        let v2_parts: Vec<u32> = v2.split('.').map(|s| s.parse().unwrap_or(0)).collect();

        let max_len = v1_parts.len().max(v2_parts.len());
        for i in 0..max_len {
            let v1_part = v1_parts.get(i).copied().unwrap_or(0);
            let v2_part = v2_parts.get(i).copied().unwrap_or(0);

            if v1_part > v2_part {
                return 1;
            } else if v1_part < v2_part {
                return -1;
            }
        }

        0
    }

    /// Update kdguard
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if successful, otherwise an error
    fn update() -> Result<(), Box<dyn std::error::Error>> {
        if OS == "Windows" {
            let mut command = Command::new("powershell")
                .arg("-ExecutionPolicy")
                .arg("ByPass")
                .arg("-c")
                .arg(
                    "irm https://raw.githubusercontent.com/KarnesTH/kdguard/main/install.ps1 | iex",
                )
                .spawn()?;
            command.wait()?;
        } else {
            let mut command = Command::new("bash")
                .arg("curl -LsSf https://raw.githubusercontent.com/KarnesTH/kdguard/main/install.sh | sh")
                .spawn()?;
            command.wait()?;
        }
        
        let _ = Config::get_languages_path();

        Ok(())
    }
}
