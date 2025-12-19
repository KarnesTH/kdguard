use std::{env::consts::OS, fs};

use inquire::Confirm;
use lingua_i18n_rs::prelude::Lingua;

use crate::config::Config;

pub struct UninstallManager;

impl UninstallManager {
    /// Uninstall kdguard
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if successful, otherwise an error
    pub fn uninstall() -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Config::get_config_path()?;
        let install_path = std::env::current_exe()?;

        let confirm_msg = Lingua::t("cli.cli_commands.uninstall.confirm", &[]).unwrap();

        let confirm = Confirm::new(&confirm_msg).with_default(false).prompt()?;

        if !confirm {
            println!(
                "{}",
                Lingua::t("cli.cli_commands.uninstall.cancelled", &[]).unwrap()
            );
            return Ok(());
        }

        let config_dir = config_path
            .parent()
            .ok_or_else(|| "Invalid config path: no parent directory".to_string())?;
        fs::remove_dir_all(config_dir)?;

        if OS == "Windows" {
            let install_dir = install_path
                .parent()
                .ok_or_else(|| "Invalid install path: no parent directory".to_string())?;
            let alias_path = install_dir.join("kdg.exe");
            if alias_path.exists() {
                fs::remove_file(&alias_path)?;
            }
            fs::remove_dir_all(install_dir)?;
        } else {
            let install_dir = install_path
                .parent()
                .ok_or_else(|| "Invalid install path: no parent directory".to_string())?;
            let alias_path = install_dir.join("kdg");
            if alias_path.exists() {
                fs::remove_file(&alias_path)?;
            }
            fs::remove_file(install_path)?;
        }

        println!(
            "\n\x1b[1;32m{}\x1b[0m",
            Lingua::t("cli.cli_commands.uninstall.success", &[]).unwrap()
        );
        println!("{}", "=".repeat(50));

        Ok(())
    }
}
