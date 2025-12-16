use clap::Parser;
use kdguard::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load_config()?;
    let cli = Cli::parse();

    if let Some(commands) = cli.commands {
        match commands {
            Commands::Check { password, detailed } => {
                HealthCheck::check_password(&password, detailed)?;
            },
            Commands::Config { commands } => {
                match commands {
                    ConfigCommands::Show => {
                        println!("Default length: {}", config.general.default_length);
                        println!("Language: {}", config.language.lang);
                    },
                    ConfigCommands::Edit => {
                        println!("Editing configuration...");
                    },
                }
            },
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
        if count > 1 {
            println!("\x1b[1;36mYour generated passwords:\x1b[0m");
        } else {
            println!("\x1b[1;36mYour generated password:\x1b[0m");
        }

        for password in passwords {
            println!("  {}", password);
        }
    }

    Ok(())
}
