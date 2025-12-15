use std::{fs::OpenOptions, io::Write, path::Path};

use chrono::Local;
use ring::rand::{SecureRandom, SystemRandom};

pub struct Generator;

impl Generator {
    /// Generate password
    ///
    /// # Arguments
    ///
    /// * `length`: length of a password
    ///
    /// # Returns
    ///
    /// Returns the generated password as String, else returns an error
    pub fn generate_password(length: usize) -> Result<String, Box<dyn std::error::Error>> {
        if !(8..=64).contains(&length) {
            return Err("Invalid password length".into());
        }

        let charset: &[u8] =
            b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-_=+";
        let rng = SystemRandom::new();
        const MAX_RETRIES: u32 = 100;

        for _ in 0..MAX_RETRIES {
            let mut password = String::with_capacity(length);

            for _ in 0..length {
                let mut bytes = [0u8; 4];
                rng.fill(&mut bytes).expect("Failed to fill bytes");
                let random_u32 = u32::from_be_bytes(bytes);
                let idx = (random_u32 as usize) % charset.len();
                password.push(charset[idx] as char);
            }

            if Self::is_valid_password(&password) {
                return Ok(password);
            }
        }

        Err("Failed to generate valid password after maximum retries".into())
    }

    /// Check valid password
    ///
    /// # Arguments
    ///
    /// * `password`: The password string to check
    ///
    /// # Returns
    ///
    /// Returns true if valid, otherwise false
    fn is_valid_password(password: &str) -> bool {
        let mut has_lower = false;
        let mut has_upper = false;
        let mut has_digit = false;
        let mut has_special = false;

        let special_chars: &[char] = &[
            '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '_', '=', '+',
        ];

        for c in password.chars() {
            if c.is_lowercase() {
                has_lower = true;
            } else if c.is_uppercase() {
                has_upper = true;
            } else if c.is_ascii_digit() {
                has_digit = true;
            } else if special_chars.contains(&c) {
                has_special = true;
            }
        }

        has_lower && has_upper && has_digit && has_special
    }

    /// Save passwords to a file
    ///
    /// # Arguments
    ///
    /// * `password`: The password string to save
    /// * `output_path`: The path to save the password to
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if successful, otherwise an error
    pub fn save_to_file(
        passwords: Vec<String>,
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(output_path)?;
        let header = format!(
            "Generated with kdguard\nDate: {}\nGenerated passwords:\n",
            Local::now().format("%d.%m.%Y %H:%M:%S")
        );
        file.write_all(header.as_bytes())?;
        for password in passwords {
            file.write_all(password.as_bytes())?;
            file.write_all(b"\n")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_password() {
        let password = Generator::generate_password(10).expect("Failed to generate password");
        assert_eq!(password.len(), 10);
        assert!(Generator::is_valid_password(&password));
    }

    #[test]
    fn test_is_valid_password() {
        assert!(Generator::is_valid_password("(123P@ssw0rd"));
        assert!(!Generator::is_valid_password("password"));
    }

    #[test]
    fn test_error_generate_password() {
        assert!(Generator::generate_password(7).is_err());
        assert!(Generator::generate_password(65).is_err());
        assert!(Generator::generate_password(8).is_ok());
        assert!(Generator::generate_password(64).is_ok());
    }
}
