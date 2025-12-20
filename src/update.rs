use std::{env::consts::OS, process::Command};

use inquire::Confirm;
use lingua_i18n_rs::prelude::Lingua;

use crate::config::Config;
use crate::errors::UpdateError;
use crate::logging::LoggingManager;

pub struct UpdateManager;

impl UpdateManager {
    /// Check for update
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if successful, otherwise an error
    pub async fn check_update() -> Result<(), UpdateError> {
        LoggingManager::info("Checking for updates");
        let current_version = env!("CARGO_PKG_VERSION");
        LoggingManager::info(&format!("Current version: {}", current_version));

        let latest_tag = Self::get_latest_tag().await?;
        let latest_version = Self::extract_version(&latest_tag);
        LoggingManager::info(&format!("Latest version: {}", latest_version));

        if Self::compare_versions(&latest_version, current_version) > 0 {
            LoggingManager::info("Update available");
            let confirm = Confirm::new(&Lingua::t("cli.cli_commands.update.confirm", &[]).unwrap())
                .with_default(false)
                .prompt()
                .map_err(|e| {
                    let error = format!("Failed to get user confirmation: {}", e);
                    LoggingManager::error(&error);
                    UpdateError::GetLatestTag(error)
                })?;
            if confirm {
                LoggingManager::info("User confirmed update, starting update process");
                Self::update()?;
            } else {
                LoggingManager::info("Update cancelled by user");
            }
        } else {
            LoggingManager::info("Already on latest version");
        }

        Ok(())
    }

    /// Get the latest tag from GitHub
    ///
    /// # Returns
    ///
    /// Returns the latest tag if successful, otherwise an error
    async fn get_latest_tag() -> Result<String, UpdateError> {
        LoggingManager::info("Fetching latest tag from GitHub API");

        let client = reqwest::Client::builder()
            .user_agent("kdguard-update-checker")
            .build()
            .map_err(|e| {
                let error = format!("Failed to create HTTP client: {}", e);
                LoggingManager::error(&error);
                UpdateError::GetLatestTag(error)
            })?;

        let res = client
            .get("https://api.github.com/repos/KarnesTH/kdguard/releases/latest")
            .send()
            .await
            .map_err(|e| {
                let error = format!("Failed to send request to GitHub API: {}", e);
                LoggingManager::error(&error);
                UpdateError::GetLatestTag(error)
            })?;

        let status = res.status();
        let text = res.text().await.map_err(|e| {
            let error = format!("Failed to read response body: {}", e);
            LoggingManager::error(&error);
            UpdateError::GetLatestTag(error)
        })?;

        if !status.is_success() {
            let error = format!("GitHub API returned status {}: {}", status, text);
            LoggingManager::error(&error);
            return Err(UpdateError::GitHubApi(error));
        }

        let json: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
            let error = format!("Failed to parse JSON response: {}. Response: {}", e, text);
            LoggingManager::error(&error);
            UpdateError::ParseJson(error)
        })?;

        let latest_tag = json
            .get("tag_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                LoggingManager::error("Failed to parse tag_name from GitHub API response");
                UpdateError::ParseTagName
            })?
            .to_string();

        LoggingManager::info(&format!("Successfully fetched latest tag: {}", latest_tag));
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
    fn update() -> Result<(), UpdateError> {
        LoggingManager::info("Starting update process");

        if OS == "Windows" {
            LoggingManager::info("Running Windows update script");
            let mut command = Command::new("powershell")
                .arg("-ExecutionPolicy")
                .arg("ByPass")
                .arg("-c")
                .arg(
                    "irm https://raw.githubusercontent.com/KarnesTH/kdguard/main/install.ps1 | iex",
                )
                .spawn()
                .map_err(|e| {
                    let error = format!("Failed to spawn PowerShell process: {}", e);
                    LoggingManager::error(&error);
                    UpdateError::SpawnProcess(error)
                })?;
            let status = command.wait().map_err(|e| {
                let error = format!("Update process failed: {}", e);
                LoggingManager::error(&error);
                UpdateError::UpdateProcess(error)
            })?;
            if !status.success() {
                let error = format!(
                    "Update script exited with non-zero status: {:?}",
                    status.code()
                );
                LoggingManager::error(&error);
                return Err(UpdateError::UpdateProcess(error));
            }
        } else {
            LoggingManager::info("Running Unix update script");
            let mut command = Command::new("sh")
                .arg("-c")
                .arg("curl -LsSf https://raw.githubusercontent.com/KarnesTH/kdguard/main/install.sh | sh")
                .spawn()
                .map_err(|e| {
                    let error = format!("Failed to spawn shell process: {}", e);
                    LoggingManager::error(&error);
                    UpdateError::SpawnProcess(error)
                })?;
            let status = command.wait().map_err(|e| {
                let error = format!("Update process failed: {}", e);
                LoggingManager::error(&error);
                UpdateError::UpdateProcess(error)
            })?;
            if !status.success() {
                let error = format!(
                    "Update script exited with non-zero status: {:?}",
                    status.code()
                );
                LoggingManager::error(&error);
                return Err(UpdateError::UpdateProcess(error));
            }
        }

        LoggingManager::info("Update process completed successfully");
        let _ = Config::get_languages_path();

        Ok(())
    }
}
