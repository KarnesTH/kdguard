use clap::Parser;
use kdguard::prelude::*;
use lingua_i18n_rs::prelude::Lingua;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging first
    LoggingManager::init()?;

    // Ensure config directory and file exist (CONFIG lazy_static needs this)
    let _ = &CONFIG;

    // Initialize languages first (before CLI parsing)
    let languages_path = Config::get_languages_path()?;
    Lingua::new(languages_path.to_str().ok_or("Invalid languages path")?).init()?;

    // Load language from config
    let config_path = Config::get_config_path()?;
    let lang =
        Lingua::load_lang_from_config(&config_path, "lang").unwrap_or_else(|_| "en".to_string());
    Lingua::set_language(&lang)?;

    // Check for update
    UpdateManager::check_update().await?;

    // Parse CLI
    let cli = Cli::parse();

    if let Some(commands) = cli.commands {
        match commands {
            Commands::Check { password, detailed } => {
                HealthCheck::check_password(&password, detailed)?;
            }
            Commands::Config { commands } => match commands {
                ConfigCommands::Show => {
                    Config::print_config(&CONFIG);
                }
                ConfigCommands::Edit {
                    lang,
                    password_length,
                    count,
                    auto_save,
                } => {
                    Config::update_config(lang, password_length, count, auto_save)?;
                }
            },
            Commands::Uninstall => {
                UninstallManager::uninstall()?;
            }
        }
        return Ok(());
    }

    // Start TUI if no commands provided
    kdguard::tui::run()?;
    Ok(())
}
