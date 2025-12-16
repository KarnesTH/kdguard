use std::{fs, path::PathBuf};

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
                general: GeneralConfig {
                    default_length: 16,
                },
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
    fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir().ok_or("Failed to get config directory")?;
        let config_path = config_dir.join("kdguard").join("config.toml");

        Ok(config_path)
    }
}