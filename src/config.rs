use std::{fs, path::PathBuf};

use lingua_i18n_rs::prelude::Lingua;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub language: LanguageConfig,
}

#[derive(Serialize, Deserialize)]
pub struct GeneralConfig {
    pub default_length: usize,
}

#[derive(Serialize, Deserialize)]
pub struct LanguageConfig {
    pub lang: String,
}

impl Config {
    /// Load the config file
    ///
    /// # Returns
    ///
    /// Returns the config if successful, otherwise an error
    pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            fs::create_dir_all(config_path.parent().unwrap())?;
            let config = Config {
                general: GeneralConfig { default_length: 16 },
                language: LanguageConfig {
                    lang: "en".to_string(),
                },
            };
            Self::save_config(&config)?;
            Ok(config)
        } else {
            let config_str = fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&config_str)?;
            Ok(config)
        }
    }

    /// Save the config file
    ///
    /// # Arguments
    ///
    /// * `config`: The config to save
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if successful, otherwise an error
    pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path()?;
        let config_str = toml::to_string_pretty(config)?;
        fs::write(config_path, config_str)?;
        Ok(())
    }

    /// Get the path to the config file
    ///
    /// # Returns
    ///
    /// Returns the path to the config file
    pub fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir().ok_or("Failed to get config directory")?;
        let config_path = config_dir.join("kdguard").join("config.toml");

        Ok(config_path)
    }

    /// Print the config to the console
    ///
    /// # Arguments
    ///
    /// * `config`: The config to print
    pub fn print_config(config: &Config) {
        let default_length = config.general.default_length.to_string();
        let language = config.language.lang.to_string();

        println!(
            "\n\x1b[1;36m{}\x1b[0m",
            Lingua::t("config.show.title", &[]).unwrap()
        );
        println!("{}", "=".repeat(50));
        println!(
            "\x1b[1;33m{}\x1b[0m",
            Lingua::t("config.show.subtitle_general", &[]).unwrap()
        );
        println!(
            "  {}",
            Lingua::t(
                "config.show.default_length",
                &[("default_length", default_length.as_str())]
            )
            .unwrap()
        );
        println!(
            "\n\x1b[1;33m{}\x1b[0m",
            Lingua::t("config.show.subtitle_language", &[]).unwrap()
        );
        println!(
            "  {}",
            Lingua::t("config.show.language", &[("language", language.as_str())]).unwrap()
        );
        println!("{}", "=".repeat(50));
    }
}
