use chrono::Local;
use clap::Parser;
use passgen::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let count = cli.count.unwrap_or(1);

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
            let today = Local::now();
            let formatted_date = today.format("%d_%m_%Y").to_string();
            format!("passwords_{}.txt", formatted_date)
        };

        let output_path = base_path.join(file_name);

        Generator::save_to_file(passwords, &output_path)?;

        println!("Passwords saved to: {}", output_path.display());
    } else {
        println!("Your generated passwords:");
        for password in passwords {
            println!("{}", password);
        }
    }

    Ok(())
}
