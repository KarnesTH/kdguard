use std::{env::consts::OS, fs};

use inquire::Confirm;
use lingua_i18n_rs::prelude::Lingua;

use crate::config::Config;
use crate::errors::UninstallError;
use crate::logging::LoggingManager;

pub struct UninstallManager;

impl UninstallManager {
    /// Uninstall kdguard
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if successful, otherwise an error
    pub fn uninstall() -> Result<(), UninstallError> {
        LoggingManager::info("Starting uninstall process");

        let config_path = Config::get_config_path().map_err(|e| {
            let error = format!("Failed to get config path: {}", e);
            LoggingManager::error(&error);
            UninstallError::GetConfigPathError(error)
        })?;
        let install_path = std::env::current_exe().map_err(|e| {
            let error = format!("Failed to get current executable path: {}", e);
            LoggingManager::error(&error);
            UninstallError::GetExecutablePathError(error)
        })?;

        LoggingManager::info(&format!("Config path: {}", config_path.display()));
        LoggingManager::info(&format!("Install path: {}", install_path.display()));

        let confirm_msg = Lingua::t("cli.cli_commands.uninstall.confirm", &[]).unwrap();

        let confirm = Confirm::new(&confirm_msg)
            .with_default(false)
            .prompt()
            .map_err(|e| {
                let error = format!("Failed to get user confirmation: {}", e);
                LoggingManager::error(&error);
                UninstallError::GetConfigPathError(error)
            })?;

        if !confirm {
            LoggingManager::info("Uninstall cancelled by user");
            println!(
                "{}",
                Lingua::t("cli.cli_commands.uninstall.cancelled", &[]).unwrap()
            );
            return Ok(());
        }

        LoggingManager::info("Removing config directory");
        let config_dir = config_path
            .parent()
            .ok_or(UninstallError::InvalidConfigPath)?;
        fs::remove_dir_all(config_dir).map_err(|e| {
            let error = format!("Failed to remove config directory: {}", e);
            LoggingManager::error(&error);
            UninstallError::RemoveConfigDirectoryError(error)
        })?;
        LoggingManager::info("Config directory removed successfully");

        if OS == "Windows" {
            LoggingManager::info("Removing Windows installation");
            let install_dir = install_path
                .parent()
                .ok_or(UninstallError::InvalidInstallPath)?;
            let alias_path = install_dir.join("kdg.exe");
            if alias_path.exists() {
                LoggingManager::info("Removing alias file");
                fs::remove_file(&alias_path).map_err(|e| {
                    let error = format!("Failed to remove alias file: {}", e);
                    LoggingManager::error(&error);
                    UninstallError::RemoveAliasError(error)
                })?;
            }
            fs::remove_dir_all(install_dir).map_err(|e| {
                let error = format!("Failed to remove install directory: {}", e);
                LoggingManager::error(&error);
                UninstallError::RemoveInstallDirectoryError(error)
            })?;
        } else {
            LoggingManager::info("Removing Unix installation");
            let install_dir = install_path
                .parent()
                .ok_or(UninstallError::InvalidInstallPath)?;
            let alias_path = install_dir.join("kdg");
            if alias_path.exists() {
                LoggingManager::info("Removing alias file");
                fs::remove_file(&alias_path).map_err(|e| {
                    let error = format!("Failed to remove alias file: {}", e);
                    LoggingManager::error(&error);
                    UninstallError::RemoveAliasError(error)
                })?;
            }
            fs::remove_file(install_path).map_err(|e| {
                let error = format!("Failed to remove executable: {}", e);
                LoggingManager::error(&error);
                UninstallError::RemoveExecutableError(error)
            })?;
        }

        LoggingManager::info("Uninstall completed successfully");
        println!(
            "\n\x1b[1;32m{}\x1b[0m",
            Lingua::t("cli.cli_commands.uninstall.success", &[]).unwrap()
        );
        println!("{}", "=".repeat(50));

        Ok(())
    }
}
