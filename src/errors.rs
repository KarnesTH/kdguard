use lingua_i18n_rs::prelude::LinguaError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Invalid config path: {0}")]
    InvalidConfigPath(String),
    #[error("Failed to load config: {0}")]
    LoadConfigurationError(String),
    #[error("Failed to save config: {0}")]
    SaveConfigurationError(String),
    #[error("Failed to parse config: {0}")]
    ParseConfigurationError(String),
    #[error("Failed to update config: {0}")]
    UpdateConfigurationError(String),
    #[error("Failed to print config: {0}")]
    PrintConfigurationError(String),
    #[error("Failed to get config path: {0}")]
    GetConfigPathError(String),
    #[error("Failed to get languages path: {0}")]
    GetLanguagesPathError(String),
    #[error("Failed to get config directory: {0}")]
    GetConfigDirectoryError(String),
    #[error("Failed to get languages directory: {0}")]
    GetLanguagesDirectoryError(String),
    #[error("Failed to create config directory: {0}")]
    CreateConfigDirectoryError(String),
    #[error("Failed to read config file: {0}")]
    ReadConfigFileError(String),
    #[error("Failed to write config file: {0}")]
    WriteConfigFileError(String),
    #[error("Failed to serialize config: {0}")]
    SerializeConfigurationError(String),
    #[error("Failed to set language: {0}")]
    SetLanguageError(#[from] LinguaError),
}

#[derive(Error, Debug)]
pub enum LoggingError {
    #[error("Failed to initialize logging: {0}")]
    Initialize(String),
    #[error("Failed to get logging directory: {0}")]
    GetDirectory(String),
    #[error("Failed to create logging directory: {0}")]
    CreateDirectory(String),
    #[error("Failed to write system info: {0}")]
    WriteSystemInfo(String),
    #[error("Failed to create log file: {0}")]
    CreateFile(String),
    #[error("Failed to cleanup old logs: {0}")]
    Cleanup(String),
}
