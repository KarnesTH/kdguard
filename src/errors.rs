use std::path::PathBuf;

use thiserror::Error;

use crate::translation::Translation;

fn text(key: &str) -> String {
    Translation::get(&format!("errors.{key}"))
}

fn text_with(key: &str, name: &str, value: &str) -> String {
    Translation::with(&format!("errors.{key}"), &[(name, value)])
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("{}", text("config.no_directory"))]
    NoConfigDirectory,
    #[error("{}  {}", text_with("config.read", "path", &.0.display().to_string()), .1)]
    Read(PathBuf, String),
    #[error("{}  {}", text_with("config.parse", "path", &.0.display().to_string()), .1)]
    Parse(PathBuf, String),
    #[error("{}  {}", text_with("config.write", "path", &.0.display().to_string()), .1)]
    Write(PathBuf, String),
    #[error("{}  {}", text("config.serialize"), .0)]
    Serialize(String),
    #[error("{}", text_with("config.invalid_length", "value", &.0.to_string()))]
    InvalidLength(usize),
    #[error("{}", text_with("config.invalid_count", "value", &.0.to_string()))]
    InvalidCount(usize),
    #[error("{}", text_with("config.invalid_language", "value", .0))]
    InvalidLanguage(String),
}

#[derive(Error, Debug)]
pub enum LoggingError {
    #[error("{}", text("logging.no_directory"))]
    NoDirectory,
    #[error("{}  {}", text("logging.create_directory"), .0)]
    CreateDirectory(String),
    #[error("{}  {}", text("logging.create_file"), .0)]
    CreateFile(String),
    #[error("{}  {}", text("logging.initialize"), .0)]
    Initialize(String),
}

#[derive(Error, Debug)]
pub enum GeneratorError {
    #[error("{}", text_with("generator.invalid_length", "value", &.0.to_string()))]
    InvalidLength(usize),
    #[error("{}", text_with("generator.invalid_count", "value", &.0.to_string()))]
    InvalidCount(usize),
    #[error("{}", text("generator.empty_pattern"))]
    EmptyPattern,
    #[error("{}", text_with("generator.invalid_pattern_character", "value", &.0.to_string()))]
    InvalidPatternCharacter(char),
    #[error("{}", text_with("generator.invalid_word_count", "value", &.0.to_string()))]
    InvalidWordCount(usize),
    #[error("{}", text("generator.empty_seed"))]
    EmptySeed,
    #[error("{}", text("generator.max_retries"))]
    MaxRetriesExceeded,
    #[error("{}", text("generator.random_bytes"))]
    RandomBytes,
    #[error("{}", text("generator.key_derivation"))]
    KeyDerivation,
    #[error("{}", text_with("generator.file_exists", "path", &.0.display().to_string()))]
    FileExists(PathBuf),
    #[error("{}  {}", text_with("generator.save_file", "path", &.0.display().to_string()), .1)]
    SaveFile(PathBuf, String),
}

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("{}", text("command.seed_missing"))]
    SeedMissing,
    #[error("{}", text_with("command.seed_env_empty", "name", .0))]
    SeedEnvEmpty(String),
    #[error("{}  {}", text("command.read_stdin"), .0)]
    ReadStdin(String),
    #[error("{}", text("command.empty_password"))]
    EmptyPassword,
}

#[derive(Error, Debug)]
pub enum UninstallError {
    #[error("{}  {}", text("uninstall.executable"), .0)]
    Executable(String),
    #[error("{}  {}", text_with("uninstall.remove", "path", &.0.display().to_string()), .1)]
    Remove(PathBuf, String),
}

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("{}  {}", text("update.curl"), .0)]
    Curl(String),
    #[error("{}", text("update.invalid_response"))]
    InvalidResponse,
    #[error("{}", text_with("update.unsupported_platform", "value", .0))]
    UnsupportedPlatform(String),
    #[error("{}", text("update.checksum_missing"))]
    ChecksumMissing,
    #[error("{}", text("update.checksum_mismatch"))]
    ChecksumMismatch,
    #[error("{}  {}", text("update.install"), .0)]
    Install(String),
}
