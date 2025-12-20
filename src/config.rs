use std::{fs, path::PathBuf};

use crate::errors::ConfigError;
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
    pub default_count: usize,
    pub default_mode: String,
    pub auto_save: bool,
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
    pub fn load_config() -> Result<Config, ConfigError> {
        let config_path = Self::get_config_path()?;
        let config_dir = config_path.parent().ok_or(ConfigError::InvalidConfigPath(
            "Invalid config path: no parent directory".to_string(),
        ))?;

        fs::create_dir_all(config_dir)
            .map_err(|e| ConfigError::CreateConfigDirectoryError(e.to_string()))?;

        if !config_path.exists() {
            let config = Config {
                general: GeneralConfig {
                    default_length: 16,
                    default_count: 1,
                    default_mode: "random".to_string(),
                    auto_save: false,
                },
                language: LanguageConfig {
                    lang: "en".to_string(),
                },
            };
            Self::save_config(&config)?;
            Ok(config)
        } else {
            let config_str = fs::read_to_string(&config_path)
                .map_err(|e| ConfigError::ReadConfigFileError(e.to_string()))?;
            let config: Config = toml::from_str(&config_str)
                .map_err(|e| ConfigError::ParseConfigurationError(e.to_string()))?;
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
    pub fn save_config(config: &Config) -> Result<(), ConfigError> {
        let config_path = Self::get_config_path()?;
        let config_dir = config_path.parent().ok_or(ConfigError::InvalidConfigPath(
            "Invalid config path: no parent directory".to_string(),
        ))?;
        fs::create_dir_all(config_dir)
            .map_err(|e| ConfigError::CreateConfigDirectoryError(e.to_string()))?;
        let config_str = toml::to_string_pretty(config)
            .map_err(|e| ConfigError::SerializeConfigurationError(e.to_string()))?;
        fs::write(config_path, config_str)
            .map_err(|e| ConfigError::WriteConfigFileError(e.to_string()))?;
        Ok(())
    }

    /// Get the path to the config file
    ///
    /// # Returns
    ///
    /// Returns the path to the config file
    pub fn get_config_path() -> Result<PathBuf, ConfigError> {
        let config_dir = dirs::config_dir().ok_or(ConfigError::GetConfigDirectoryError(
            "Failed to get config directory".to_string(),
        ))?;
        let config_path = config_dir.join("kdguard").join("config.toml");

        Ok(config_path)
    }

    /// Get the path to the languages directory
    ///
    /// Languages are embedded in the binary, but this function returns
    /// a temporary directory path for Lingua initialization.
    ///
    /// # Returns
    ///
    /// Returns the path to the languages directory
    pub fn get_languages_path() -> Result<PathBuf, ConfigError> {
        const EN_JSON: &str = include_str!("../languages/en.json");
        const DE_JSON: &str = include_str!("../languages/de.json");

        let config_dir = dirs::config_dir().ok_or(ConfigError::GetConfigDirectoryError(
            "Failed to get config directory".to_string(),
        ))?;
        let languages_dir = config_dir.join("kdguard").join("languages");

        fs::create_dir_all(&languages_dir)
            .map_err(|e| ConfigError::CreateConfigDirectoryError(e.to_string()))?;
        fs::write(languages_dir.join("en.json"), EN_JSON)
            .map_err(|e| ConfigError::WriteConfigFileError(e.to_string()))?;
        fs::write(languages_dir.join("de.json"), DE_JSON)
            .map_err(|e| ConfigError::WriteConfigFileError(e.to_string()))?;

        Ok(languages_dir)
    }

    /// Update the config file
    ///
    /// # Arguments
    ///
    /// * `lang`: The language to set
    /// * `password_length`: The password length to set
    /// * `count`: The count to set
    /// * `auto_save`: The auto save to set
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if successful, otherwise an error
    pub fn update_config(
        lang: Option<String>,
        password_length: Option<usize>,
        count: Option<usize>,
        auto_save: Option<bool>,
    ) -> Result<(), ConfigError> {
        let mut new_config = Config::load_config()?;

        if let Some(lang) = lang {
            new_config.language.lang = lang;
            Lingua::set_language(&new_config.language.lang)
                .map_err(ConfigError::SetLanguageError)?;
        }
        if let Some(length) = password_length {
            new_config.general.default_length = length;
        }
        if let Some(count) = count {
            new_config.general.default_count = count;
        }
        if let Some(auto_save) = auto_save {
            new_config.general.auto_save = auto_save;
        }

        Self::save_config(&new_config)?;

        println!(
            "\n\x1b[1;32m{}\x1b[0m",
            Lingua::t("config.edit.success", &[]).unwrap()
        );
        println!("{}", "=".repeat(50));

        Ok(())
    }

    /// Print the config to the console
    ///
    /// # Arguments
    ///
    /// * `config`: The config to print
    pub fn print_config(config: &Config) {
        let default_length = config.general.default_length.to_string();
        let language = config.language.lang.to_string();
        let default_count = config.general.default_count.to_string();
        let auto_save = config.general.auto_save.to_string();

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
            "  {}",
            Lingua::t(
                "config.show.default_count",
                &[("default_count", default_count.as_str())]
            )
            .unwrap()
        );
        println!(
            "  {}",
            Lingua::t(
                "config.show.auto_save",
                &[("auto_save", auto_save.to_string().as_str())]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config_path() {
        let path = Config::get_config_path();
        assert!(path.is_ok());
        let path_buf = path.unwrap();
        let path_str = path_buf.to_string_lossy();
        assert!(path_str.contains("kdguard"));
        assert!(path_str.contains("config.toml"));
    }

    #[test]
    fn test_get_languages_path() {
        let path = Config::get_languages_path();
        assert!(path.is_ok());
        let path_buf = path.unwrap();
        let path_str = path_buf.to_string_lossy();
        assert!(path_str.contains("languages"));
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            general: GeneralConfig {
                default_length: 20,
                default_count: 5,
                default_mode: "phrase".to_string(),
                auto_save: true,
            },
            language: LanguageConfig {
                lang: "de".to_string(),
            },
        };

        let config_str = toml::to_string(&config);
        assert!(config_str.is_ok());
        let config_str = config_str.unwrap();
        assert!(config_str.contains("default_length = 20"));
        assert!(config_str.contains("lang = \"de\""));
        assert!(config_str.contains("default_mode = \"phrase\""));

        let parsed: Result<Config, _> = toml::from_str(&config_str);
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.general.default_length, 20);
        assert_eq!(parsed.language.lang, "de");
    }
}
