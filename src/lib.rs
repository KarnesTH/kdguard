use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

use crate::translation::Translation;

pub mod commands;
pub mod config;
pub mod errors;
#[rustfmt::skip]
pub mod kd_cli;
pub mod logging;
pub mod password;
pub mod translation;
pub mod uninstall;
pub mod update;

#[derive(ValueEnum, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PasswordMode {
    Random,
    Pattern,
    Phrase,
    Deterministic,
}

impl PasswordMode {
    pub const ALL: [PasswordMode; 4] = [
        PasswordMode::Random,
        PasswordMode::Pattern,
        PasswordMode::Phrase,
        PasswordMode::Deterministic,
    ];

    /// Returns the lowercase name used in the config file and on the command line.
    pub fn name(&self) -> &'static str {
        match self {
            PasswordMode::Random => "random",
            PasswordMode::Pattern => "pattern",
            PasswordMode::Phrase => "phrase",
            PasswordMode::Deterministic => "deterministic",
        }
    }
}

#[derive(Parser)]
#[command(
    version,
    about = Translation::get("cli.about"),
    author = "KarnesTH <p_haehnel@hotmail.de>"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    #[arg(short, long, value_enum, help = Translation::get("cli.args.mode_help"))]
    pub mode: Option<PasswordMode>,
    #[arg(short, long, help = Translation::get("cli.args.length_help"))]
    pub length: Option<usize>,
    #[arg(short, long, help = Translation::get("cli.args.count_help"))]
    pub count: Option<usize>,
    #[arg(short, long, conflicts_with = "no_save", help = Translation::get("cli.args.save_help"))]
    pub save: bool,
    #[arg(long, help = Translation::get("cli.args.no_save_help"))]
    pub no_save: bool,
    #[arg(short, long, help = Translation::get("cli.args.output_help"))]
    pub output: Option<PathBuf>,
    #[arg(short, long, help = Translation::get("cli.args.pattern_help"))]
    pub pattern: Option<String>,
    #[arg(short, long, help = Translation::get("cli.args.words_help"))]
    pub words: Option<usize>,
    #[arg(long, help = Translation::get("cli.args.seed_env_help"))]
    pub seed_env: Option<String>,
    #[arg(long, help = Translation::get("cli.args.service_help"))]
    pub service: Option<String>,
    #[arg(long, help = Translation::get("cli.args.salt_help"))]
    pub salt: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = Translation::get("cli.commands.check.about"))]
    Check {
        #[arg(help = Translation::get("cli.commands.check.password_help"))]
        password: Option<String>,
        #[arg(short, long, help = Translation::get("cli.commands.check.detailed_help"))]
        detailed: bool,
    },
    #[command(about = Translation::get("cli.commands.config.about"))]
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    #[command(about = Translation::get("cli.commands.uninstall.about"))]
    Uninstall,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    #[command(about = Translation::get("cli.commands.config.show_about"))]
    Show,
    #[command(about = Translation::get("cli.commands.config.edit_about"))]
    Edit {
        #[arg(short, long, value_enum, help = Translation::get("cli.commands.config.mode_help"))]
        mode: Option<PasswordMode>,
        #[arg(short = 'p', long, help = Translation::get("cli.commands.config.length_help"))]
        password_length: Option<usize>,
        #[arg(short, long, help = Translation::get("cli.commands.config.count_help"))]
        count: Option<usize>,
        #[arg(short, long, help = Translation::get("cli.commands.config.auto_save_help"))]
        auto_save: Option<bool>,
        #[arg(short, long, help = Translation::get("cli.commands.config.language_help"))]
        lang: Option<String>,
    },
}
