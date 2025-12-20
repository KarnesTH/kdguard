use std::{
    env::consts::{ARCH, OS},
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use inquire::Confirm;
use lingua_i18n_rs::prelude::Lingua;

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
                Self::update(&latest_tag).await?;
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

        if let Some(tag_value) = json.get("tag_name") {
            LoggingManager::info(&format!("Raw tag_name from API: {:?}", tag_value));
        }

        let latest_tag = json
            .get("tag_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                LoggingManager::error("Failed to parse tag_name from GitHub API response");
                UpdateError::ParseTagName
            })?
            .trim()
            .to_string();

        if latest_tag.contains("Full Changelog")
            || latest_tag.contains("http")
            || latest_tag.contains("compare")
        {
            let error = format!("Invalid tag format received: {}", latest_tag);
            LoggingManager::error(&error);
            return Err(UpdateError::ParseTagName);
        }

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

    /// Detect the platform for the binary download
    ///
    /// # Returns
    ///
    /// Returns the platform if successful, otherwise an error
    fn detect_platform() -> Result<String, UpdateError> {
        let os = match OS {
            "linux" => "linux",
            "macos" | "darwin" => "macos",
            "windows" => "windows",
            _ => {
                let error = format!("Unsupported OS: {}", OS);
                LoggingManager::error(&error);
                return Err(UpdateError::Update(error));
            }
        };

        let arch = match ARCH {
            "x86_64" | "amd64" => "x86_64",
            "aarch64" | "arm64" => {
                if os == "macos" {
                    "aarch64"
                } else {
                    let error = format!("Unsupported architecture for {}: {}", os, ARCH);
                    LoggingManager::error(&error);
                    return Err(UpdateError::Update(error));
                }
            }
            _ => {
                let error = format!("Unsupported architecture: {}", ARCH);
                LoggingManager::error(&error);
                return Err(UpdateError::Update(error));
            }
        };

        let platform = if os == "macos" && arch == "aarch64" {
            "macos-aarch64"
        } else if os == "macos" && arch == "x86_64" {
            "macos-x86_64"
        } else if os == "linux" && arch == "x86_64" {
            "linux-x86_64"
        } else if os == "windows" && arch == "x86_64" {
            "windows-x86_64"
        } else {
            let error = format!("Unsupported platform combination: {}-{}", os, arch);
            LoggingManager::error(&error);
            return Err(UpdateError::Update(error));
        };

        Ok(platform.to_string())
    }

    /// Get the install directory path
    ///
    /// # Returns
    ///
    /// Returns the install directory path if successful, otherwise an error
    fn get_install_dir() -> Result<PathBuf, UpdateError> {
        if OS == "Windows" {
            let local_app_data = dirs::data_local_dir().ok_or_else(|| {
                let error = "Failed to get LocalAppData directory".to_string();
                LoggingManager::error(&error);
                UpdateError::Update(error)
            })?;
            Ok(local_app_data.join("Karnes Development").join("kdguard"))
        } else {
            let home = dirs::home_dir().ok_or_else(|| {
                let error = "Failed to get home directory".to_string();
                LoggingManager::error(&error);
                UpdateError::Update(error)
            })?;
            Ok(home.join(".local").join("bin"))
        }
    }

    /// Update kdguard by downloading and installing the binary directly
    ///
    /// # Arguments
    ///
    /// * `tag` - The tag version to install
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if successful, otherwise an error
    async fn update(tag: &str) -> Result<(), UpdateError> {
        LoggingManager::info(&format!("Starting update process for tag: {}", tag));

        let platform = Self::detect_platform()?;
        let version = Self::extract_version(tag);
        let install_dir = Self::get_install_dir()?;

        LoggingManager::info(&format!("Platform: {}, Version: {}", platform, version));
        LoggingManager::info(&format!("Install directory: {}", install_dir.display()));

        fs::create_dir_all(&install_dir).map_err(|e| {
            let error = format!("Failed to create install directory: {}", e);
            LoggingManager::error(&error);
            UpdateError::Update(error)
        })?;

        let binary_name = if OS == "Windows" {
            format!("kdguard_{}-{}.exe", version, platform)
        } else {
            format!("kdguard_{}-{}", version, platform)
        };
        let download_url = format!(
            "https://github.com/KarnesTH/kdguard/releases/download/{}/{}",
            tag, binary_name
        );

        LoggingManager::info(&format!("Downloading from: {}", download_url));

        let client = reqwest::Client::builder()
            .user_agent("kdguard-update-checker")
            .build()
            .map_err(|e| {
                let error = format!("Failed to create HTTP client: {}", e);
                LoggingManager::error(&error);
                UpdateError::GetLatestTag(error)
            })?;

        let response = client.get(&download_url).send().await.map_err(|e| {
            let error = format!("Failed to download binary: {}", e);
            LoggingManager::error(&error);
            UpdateError::Update(error)
        })?;

        if !response.status().is_success() {
            let error = format!("Failed to download binary: HTTP {}", response.status());
            LoggingManager::error(&error);
            return Err(UpdateError::Update(error));
        }

        let binary_data = response.bytes().await.map_err(|e| {
            let error = format!("Failed to read binary data: {}", e);
            LoggingManager::error(&error);
            UpdateError::Update(error)
        })?;

        let binary_name_final = if OS == "Windows" {
            "kdguard.exe"
        } else {
            "kdguard"
        };
        let binary_path = install_dir.join(binary_name_final);
        let temp_binary_path = install_dir.join(format!("{}.new", binary_name_final));
        let old_binary_path = install_dir.join(format!("{}.old", binary_name_final));

        LoggingManager::info(&format!("Installing binary to: {}", binary_path.display()));

        let mut file = File::create(&temp_binary_path).map_err(|e| {
            let error = format!("Failed to create temporary binary file: {}", e);
            LoggingManager::error(&error);
            UpdateError::Update(error)
        })?;

        file.write_all(&binary_data).map_err(|e| {
            let error = format!("Failed to write binary data: {}", e);
            LoggingManager::error(&error);
            UpdateError::Update(error)
        })?;
        drop(file);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&temp_binary_path)
                .map_err(|e| {
                    let error = format!("Failed to get file metadata: {}", e);
                    LoggingManager::error(&error);
                    UpdateError::Update(error)
                })?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&temp_binary_path, perms).map_err(|e| {
                let error = format!("Failed to set file permissions: {}", e);
                LoggingManager::error(&error);
                UpdateError::Update(error)
            })?;
        }

        if binary_path.exists() {
            if old_binary_path.exists() {
                fs::remove_file(&old_binary_path)
                    .map_err(|e| {
                        let error = format!("Failed to remove old backup: {}", e);
                        LoggingManager::warn(&error);
                    })
                    .ok();
            }
            fs::rename(&binary_path, &old_binary_path).map_err(|e| {
                let error = format!("Failed to rename old binary: {}", e);
                LoggingManager::error(&error);
                UpdateError::Update(error)
            })?;
        }

        fs::rename(&temp_binary_path, &binary_path).map_err(|e| {
            let error = format!("Failed to rename new binary: {}", e);
            LoggingManager::error(&error);
            if old_binary_path.exists() {
                fs::rename(&old_binary_path, &binary_path).ok();
            }
            UpdateError::Update(error)
        })?;

        if old_binary_path.exists() {
            fs::remove_file(&old_binary_path)
                .map_err(|e| {
                    let error = format!("Failed to remove old backup: {}", e);
                    LoggingManager::warn(&error);
                })
                .ok();
        }

        let alias_name = if OS == "Windows" { "kdg.exe" } else { "kdg" };
        let alias_path = install_dir.join(alias_name);

        if alias_path.exists() {
            fs::remove_file(&alias_path)
                .map_err(|e| {
                    let error = format!("Failed to remove existing alias: {}", e);
                    LoggingManager::warn(&error);
                })
                .ok();
        }

        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_file;
            if symlink_file(&binary_path, &alias_path).is_err() {
                fs::copy(&binary_path, &alias_path).map_err(|e| {
                    let error = format!("Failed to create alias: {}", e);
                    LoggingManager::error(&error);
                    UpdateError::Update(error)
                })?;
            }
        }

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&binary_path, &alias_path).map_err(|e| {
                let error = format!("Failed to create symlink: {}", e);
                LoggingManager::error(&error);
                UpdateError::Update(error)
            })?;
        }

        LoggingManager::info("Update process completed successfully");
        LoggingManager::info(&format!(
            "Binary installed to: {}\nAlias created at: {}",
            binary_path.display(),
            alias_path.display()
        ));

        Ok(())
    }
}
