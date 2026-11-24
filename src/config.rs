use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::PasswordMode;
use crate::errors::ConfigError;
use crate::logging::LoggingManager;
use crate::translation::Translation;

pub const LENGTH_RANGE: (usize, usize) = (8, 64);
pub const COUNT_RANGE: (usize, usize) = (1, 100);
pub const WORDS_RANGE: (usize, usize) = (3, 20);
pub const DEFAULT_WORDS: usize = 6;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub general: GeneralConfig,
    pub language: LanguageConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GeneralConfig {
    pub default_length: usize,
    pub default_count: usize,
    pub default_mode: PasswordMode,
    pub auto_save: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LanguageConfig {
    pub lang: String,
}

impl Default for Config {
    /// Length 16, one password, random mode, no auto save and the system language.
    fn default() -> Self {
        Config {
            general: GeneralConfig {
                default_length: 16,
                default_count: 1,
                default_mode: PasswordMode::Random,
                auto_save: false,
            },
            language: LanguageConfig {
                lang: Translation::system_language().to_string(),
            },
        }
    }
}

impl Config {
    /// Reads and validates the config file. A missing file is created with the defaults.
    pub fn load() -> Result<Config, ConfigError> {
        let path = Self::path()?;
        if !path.exists() {
            LoggingManager::info("Config file not found, creating default config");
            let config = Config::default();
            config.save()?;
            return Ok(config);
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| ConfigError::Read(path.clone(), e.to_string()))?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| ConfigError::Parse(path.clone(), e.to_string()))?;
        config.validate()?;
        LoggingManager::info(&format!(
            "Config loaded: mode={}, length={}, count={}, auto_save={}, lang={}",
            config.general.default_mode.name(),
            config.general.default_length,
            config.general.default_count,
            config.general.auto_save,
            config.language.lang
        ));
        Ok(config)
    }

    /// Validates the config and writes it to the config file, creating the directory if needed.
    pub fn save(&self) -> Result<(), ConfigError> {
        self.validate()?;
        let path = Self::path()?;
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| ConfigError::Write(path.clone(), e.to_string()))?;
        }
        let content =
            toml::to_string_pretty(self).map_err(|e| ConfigError::Serialize(e.to_string()))?;
        fs::write(&path, content).map_err(|e| ConfigError::Write(path.clone(), e.to_string()))?;
        LoggingManager::info("Config saved");
        Ok(())
    }

    /// Checks length, count and language against their allowed values.
    pub fn validate(&self) -> Result<(), ConfigError> {
        let (min_length, max_length) = LENGTH_RANGE;
        if !(min_length..=max_length).contains(&self.general.default_length) {
            return Err(ConfigError::InvalidLength(self.general.default_length));
        }
        let (min_count, max_count) = COUNT_RANGE;
        if !(min_count..=max_count).contains(&self.general.default_count) {
            return Err(ConfigError::InvalidCount(self.general.default_count));
        }
        if !Translation::is_supported(&self.language.lang) {
            return Err(ConfigError::InvalidLanguage(self.language.lang.clone()));
        }
        Ok(())
    }

    /// Returns `<config dir>/kdguard/config.toml`.
    pub fn path() -> Result<PathBuf, ConfigError> {
        let dir = dirs::config_dir().ok_or(ConfigError::NoConfigDirectory)?;
        Ok(dir.join("kdguard").join("config.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        Config {
            general: GeneralConfig {
                default_length: 20,
                default_count: 5,
                default_mode: PasswordMode::Phrase,
                auto_save: true,
            },
            language: LanguageConfig {
                lang: "de".to_string(),
            },
        }
    }

    #[test]
    fn test_config_path() {
        let path = Config::path().unwrap();
        assert!(path.ends_with("kdguard/config.toml"));
    }

    #[test]
    fn test_config_roundtrip() {
        let content = toml::to_string(&config()).unwrap();
        assert!(content.contains("default_length = 20"));
        assert!(content.contains("default_mode = \"phrase\""));
        assert!(content.contains("lang = \"de\""));
        let parsed: Config = toml::from_str(&content).unwrap();
        assert_eq!(parsed, config());
    }

    #[test]
    fn test_old_config_file_still_parses() {
        let content = "[general]\ndefault_length = 16\ndefault_count = 1\ndefault_mode = \"random\"\nauto_save = false\n\n[language]\nlang = \"en\"\n";
        let parsed: Config = toml::from_str(content).unwrap();
        assert_eq!(parsed.general.default_mode, PasswordMode::Random);
    }

    #[test]
    fn test_validate() {
        assert!(config().validate().is_ok());
        let mut invalid = config();
        invalid.general.default_length = 7;
        assert!(invalid.validate().is_err());
        let mut invalid = config();
        invalid.general.default_count = 0;
        assert!(invalid.validate().is_err());
        let mut invalid = config();
        invalid.language.lang = "fr".to_string();
        assert!(invalid.validate().is_err());
    }
}
