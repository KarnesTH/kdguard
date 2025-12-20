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

    let count = cli.count;

    let mut passwords = Vec::new();

    for _ in 0..count {
        let password = match cli.mode {
            PasswordMode::Random => Generator::generate_random_password(cli.length)?,
            PasswordMode::Pattern => {
                let pattern = cli
                    .pattern
                    .as_ref()
                    .ok_or("Pattern is required for pattern mode. Use --pattern <PATTERN>")?;
                Generator::generate_pattern_password(pattern)?
            }
            PasswordMode::Phrase => {
                let words = cli
                    .words
                    .ok_or("Word count is required for phrase mode. Use --words <COUNT>")?;
                Generator::generate_phrase_password(words)?
            }
            PasswordMode::Deterministic => {
                let seed_env_var = cli
                    .seed_env
                    .as_ref()
                    .ok_or("--seed-env is required for deterministic mode")?;

                let seed = std::env::var(seed_env_var)
                    .map_err(|_| format!("Environment variable '{}' not found", seed_env_var))?;

                let salt = cli.salt.as_deref();
                let service = cli.service.as_deref();

                Generator::generate_deterministic_password(&seed, salt, service)?
            }
        };

        passwords.push(password);
    }

    if cli.save {
        let base_path = dirs::document_dir().ok_or("Failed to get document directory")?;
        let file_name = if let Some(output) = cli.output {
            output
        } else {
            "kdguard.txt".to_string()
        };

        let output_path = base_path.join(file_name);

        Generator::save_to_file(passwords, &output_path)?;

        println!("Passwords saved to: {}", output_path.display());
    } else {
        println!(
            "\n\x1b[1;36m{}\x1b[0m",
            Lingua::t("commands.generate.title", &[]).unwrap()
        );
        println!("{}", "=".repeat(50));

        for (idx, password) in passwords.iter().enumerate() {
            println!("  {:<5} {}", idx + 1, password);
        }
        println!("{}", "=".repeat(50));
    }

    Ok(())
}
