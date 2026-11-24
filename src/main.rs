use std::process::ExitCode;

use clap::Parser;
use kdguard::commands::{CheckCommand, ConfigCommand, GenerateCommand, HelpCommand};
use kdguard::config::Config;
use kdguard::kd_cli::output::Output;
use kdguard::kd_cli::prompt::PromptError;
use kdguard::kd_cli::terminal::Terminal;
use kdguard::logging::LoggingManager;
use kdguard::translation::Translation;
use kdguard::uninstall::UninstallManager;
use kdguard::update::UpdateManager;
use kdguard::{Cli, Commands, ConfigCommands};

fn main() -> ExitCode {
    let logging = LoggingManager::init();

    let config = match Config::load() {
        Ok(config) => config,
        Err(error) => {
            let _ = Translation::init(Translation::system_language());
            Output::error(&Translation::get("errors.config.load"), &error.to_string());
            return ExitCode::FAILURE;
        }
    };
    if let Err(error) = Translation::init(&config.language.lang) {
        Output::error("Failed to load language files", &error.to_string());
        return ExitCode::FAILURE;
    }
    if let Err(error) = logging {
        Output::warn(
            &Translation::get("errors.logging.disabled"),
            &error.to_string(),
        );
    }

    let args: Vec<String> = std::env::args().collect();
    if HelpCommand::requested(&args) {
        HelpCommand::print(&args);
        return ExitCode::SUCCESS;
    }
    let interactive = args.len() == 1 && Terminal::interactive();
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => return HelpCommand::argument_error(&error),
    };
    if !matches!(cli.command, Some(Commands::Uninstall)) {
        UpdateManager::check();
    }

    let result = match cli.command {
        Some(Commands::Check {
            ref password,
            detailed,
        }) => CheckCommand::run(password.clone(), detailed),
        Some(Commands::Config {
            command: ConfigCommands::Show,
        }) => ConfigCommand::show(&config),
        Some(Commands::Config {
            command:
                ConfigCommands::Edit {
                    mode,
                    password_length,
                    count,
                    auto_save,
                    ref lang,
                },
        }) => ConfigCommand::edit(
            &config,
            mode,
            password_length,
            count,
            auto_save,
            lang.clone(),
        ),
        Some(Commands::Uninstall) => UninstallManager::uninstall(),
        None if interactive => GenerateCommand::new(&config).run_interactive(),
        None => GenerateCommand::new(&config).run(&cli),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => match error.downcast_ref::<PromptError>() {
            Some(PromptError::Aborted) => ExitCode::SUCCESS,
            Some(PromptError::NotInteractive) => ExitCode::FAILURE,
            None => {
                LoggingManager::error(&error.to_string());
                Output::error(&Translation::get("errors.title"), &error.to_string());
                ExitCode::FAILURE
            }
        },
    }
}
