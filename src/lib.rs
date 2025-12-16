use clap::{Parser, Subcommand};

mod generator;
mod health_check;
mod config;

#[derive(Parser)]
#[command(
    version,
    about = "A CLI tool to generate secure and random passwords",
    author = "KarnesTH <p_haehnel@hotmail.de>"
)]
pub struct Cli {
    #[clap(subcommand)]
    pub commands: Option<Commands>,
    #[clap(short, long, help = "Length of your password", default_value_t = 16)]
    pub length: usize,
    #[clap(short, long, help = "Amount of passwords", default_value_t = 1)]
    pub count: usize,
    #[clap(
        short,
        long,
        help = "If you want to save the password to a file",
        default_value_t = false
    )]
    pub save: bool,
    #[clap(short, long, help = "Output name to save a file (e.g. kdguard.txt)")]
    pub output: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Check a password")]
    Check {
        #[clap(
            help = "The password to check. Use '' for passwords with special characters like '$', '!', '(', ')'"
        )]
        password: String,
        #[clap(short, long, help = "Show detailed analysis", default_value_t = false)]
        detailed: bool,
    },
    #[command(about = "Manage configuration")]
    Config {
        #[clap(subcommand)]
        commands: ConfigCommands,
    }
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    #[command(about = "Show current configuration")]
    Show,
    #[command(about = "Edit configuration")]
    Edit,
}

pub mod prelude {
    pub use crate::generator::Generator;
    pub use crate::health_check::HealthCheck;
    pub use crate::{Cli, Commands, ConfigCommands};
    pub use crate::config::Config;
}
