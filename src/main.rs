use clap::Parser;
use kdguard::prelude::*;
use lingua_i18n_rs::prelude::Lingua;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for update
    UpdateManager::check_update().await?;

    // Configure language
    Lingua::new("languages").init()?;
    let config_path = Config::get_config_path()?;
    let lang = Lingua::load_lang_from_config(&config_path, "lang")?;
    Lingua::set_language(&lang)?;

    // Load config
    let config = &CONFIG;

    // Parse CLI
    let cli = Cli::parse();

    if let Some(commands) = cli.commands {
        match commands {
            Commands::Check { password, detailed } => {
                HealthCheck::check_password(&password, detailed)?;
            }
            Commands::Config { commands } => match commands {
                ConfigCommands::Show => {
                    Config::print_config(config);
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
        let password = Generator::generate_password(cli.length)?;

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
