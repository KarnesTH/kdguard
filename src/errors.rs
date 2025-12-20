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

#[derive(Error, Debug)]
pub enum GeneratorError {
    #[error("Invalid password length: {0}")]
    InvalidLength(String),
    #[error("Pattern cannot be empty")]
    EmptyPattern,
    #[error("Invalid pattern character: {0}")]
    InvalidPatternCharacter(char),
    #[error("Word count must be between 3 and 20")]
    InvalidWordCount,
    #[error("Wordlist is empty")]
    EmptyWordlist,
    #[error("Seed cannot be empty")]
    EmptySeed,
    #[error("Failed to generate valid password after maximum retries")]
    MaxRetriesExceeded,
    #[error("Failed to fill random bytes: {0}")]
    RandomBytesError(String),
    #[error("Failed to expand HKDF")]
    HkdfExpandError,
    #[error("Failed to fill HKDF output")]
    HkdfFillError,
    #[error("Failed to save passwords to file: {0}")]
    SaveFileError(String),
}

#[derive(Error, Debug)]
pub enum HealthCheckError {
    #[error("Failed to analyze password: {0}")]
    AnalysisError(String),
}

#[derive(Error, Debug)]
pub enum UninstallError {
    #[error("Failed to get config path: {0}")]
    GetConfigPathError(String),
    #[error("Failed to get current executable path: {0}")]
    GetExecutablePathError(String),
    #[error("Invalid config path: no parent directory")]
    InvalidConfigPath,
    #[error("Invalid install path: no parent directory")]
    InvalidInstallPath,
    #[error("Failed to remove config directory: {0}")]
    RemoveConfigDirectoryError(String),
    #[error("Failed to remove alias file: {0}")]
    RemoveAliasError(String),
    #[error("Failed to remove install directory: {0}")]
    RemoveInstallDirectoryError(String),
    #[error("Failed to remove executable: {0}")]
    RemoveExecutableError(String),
}

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("Failed to get latest tag from GitHub: {0}")]
    GetLatestTag(String),
    #[error("GitHub API returned error: {0}")]
    GitHubApi(String),
    #[error("Failed to parse JSON response: {0}")]
    ParseJson(String),
    #[error("Failed to parse tag_name from GitHub API response")]
    ParseTagName,
    #[error("Failed to update kdguard: {0}")]
    Update(String),
    #[error("Failed to spawn update process: {0}")]
    SpawnProcess(String),
    #[error("Update process failed: {0}")]
    UpdateProcess(String),
}
