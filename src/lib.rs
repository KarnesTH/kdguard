use clap::{Parser, Subcommand};
use lazy_static::lazy_static;
use lingua_i18n_rs::prelude::Lingua;

use crate::config::Config;

mod config;
mod generator;
mod health_check;
mod uninstall;

lazy_static! {
    pub static ref CONFIG: Config = Config::load_config().unwrap();
}

#[derive(Parser)]
#[command(
    version,
    about = Lingua::t("cli.about", &[]).unwrap(),
    author = "KarnesTH <p_haehnel@hotmail.de>"
)]
pub struct Cli {
    #[clap(subcommand)]
    pub commands: Option<Commands>,
    #[clap(short, long, help = Lingua::t("cli.args.length_help", &[]).unwrap(), default_value_t = CONFIG.general.default_length)]
    pub length: usize,
    #[clap(short, long, help = Lingua::t("cli.args.count_help", &[]).unwrap(), default_value_t = CONFIG.general.default_count)]
    pub count: usize,
    #[clap(
        short,
        long,
        help = Lingua::t("cli.args.save_help", &[]).unwrap(),
        default_value_t = CONFIG.general.auto_save
    )]
    pub save: bool,
    #[clap(short, long, help = Lingua::t("cli.args.output_help", &[]).unwrap())]
    pub output: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = Lingua::t("cli.cli_commands.check_password.about", &[]).unwrap())]
    Check {
        #[clap(
            help = Lingua::t("cli.cli_commands.check_password.password_help", &[]).unwrap()
        )]
        password: String,
        #[clap(short, long, help = Lingua::t("cli.cli_commands.check_password.detailed_help", &[]).unwrap(), default_value_t = false)]
        detailed: bool,
    },
    #[command(about = Lingua::t("cli.cli_commands.manage_config.about", &[]).unwrap())]
    Config {
        #[clap(subcommand)]
        commands: ConfigCommands,
    },
    #[command(about = Lingua::t("cli.cli_commands.uninstall.about", &[]).unwrap())]
    Uninstall,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    #[command(about = Lingua::t("cli.cli_commands.manage_config.show_about", &[]).unwrap())]
    Show,
    #[command(about = Lingua::t("cli.cli_commands.manage_config.edit_about", &[]).unwrap())]
    Edit {
        #[clap(short, long, help = Lingua::t("cli.cli_commands.manage_config.edit_language_help", &[]).unwrap())]
        lang: Option<String>,
        #[clap(short, long, help = Lingua::t("cli.cli_commands.manage_config.edit_default_length_help", &[]).unwrap())]
        password_length: Option<usize>,
        #[clap(short, long, help = Lingua::t("cli.cli_commands.manage_config.edit_default_count_help", &[]).unwrap())]
        count: Option<usize>,
        #[clap(short, long, help = Lingua::t("cli.cli_commands.manage_config.edit_auto_save_help", &[]).unwrap())]
        auto_save: Option<bool>,
    },
}

pub mod prelude {
    pub use super::CONFIG;
    pub use crate::config::Config;
    pub use crate::generator::Generator;
    pub use crate::health_check::HealthCheck;
    pub use crate::uninstall::UninstallManager;
    pub use crate::{Cli, Commands, ConfigCommands};
}
