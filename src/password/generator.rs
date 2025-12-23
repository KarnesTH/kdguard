use std::{fs::OpenOptions, io::Write, path::Path, sync::OnceLock};

use chrono::Local;
use ring::hkdf;
use ring::rand::{SecureRandom, SystemRandom};

use crate::CONFIG;
use crate::errors::GeneratorError;
use crate::logging::LoggingManager;

const CHARSET: &[u8] =
    b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-_=+";

pub struct Generator;

impl Generator {
    /// Generate random password
    ///
    /// # Arguments
    ///
    /// * `length`: length of a password
    ///
    /// # Returns
    ///
    /// Returns the generated password as String, else returns an error
    pub fn generate_random_password(length: usize) -> Result<String, GeneratorError> {
        LoggingManager::info(&format!(
            "Generating random password with length: {}",
            length
        ));

        if !(8..=64).contains(&length) {
            let error = format!("Password length must be between 8 and 64, got: {}", length);
            LoggingManager::error(&error);
            return Err(GeneratorError::InvalidLength(error));
        }

        let rng = SystemRandom::new();
        const MAX_RETRIES: u32 = 100;

        for attempt in 0..MAX_RETRIES {
            let mut password = String::with_capacity(length);

            for _ in 0..length {
                let mut bytes = [0u8; 4];
                rng.fill(&mut bytes).map_err(|e| {
                    let error = format!("Failed to fill random bytes: {}", e);
                    LoggingManager::error(&error);
                    GeneratorError::RandomBytesError(error)
                })?;
                let random_u32 = u32::from_be_bytes(bytes);
                let idx = (random_u32 as usize) % CHARSET.len();
                password.push(CHARSET[idx] as char);
            }

            if Self::is_valid_password(&password) {
                LoggingManager::info(&format!(
                    "Successfully generated random password (attempt {})",
                    attempt + 1
                ));
                return Ok(password);
            }
        }

        LoggingManager::error("Failed to generate valid password after maximum retries");
        Err(GeneratorError::MaxRetriesExceeded)
    }

    /// Generate pattern based password
    ///
    /// # Arguments
    ///
    /// * `pattern`: The pattern to generate the password from
    ///
    /// # Returns
    ///
    /// Returns the generated password as String, else returns an error
    pub fn generate_pattern_password(pattern: &str) -> Result<String, GeneratorError> {
        LoggingManager::info(&format!(
            "Generating pattern password with pattern: {}",
            pattern
        ));

        if pattern.is_empty() {
            LoggingManager::error("Pattern cannot be empty");
            return Err(GeneratorError::EmptyPattern);
        }

        const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
        const DIGITS: &[u8] = b"0123456789";
        const SPECIAL: &[u8] = b"!@#$%^&*()-_=+";

        let mut password = String::with_capacity(pattern.len());
        let rng = SystemRandom::new();

        for c in pattern.chars() {
            let charset: &[u8] = match c {
                'U' => UPPERCASE,
                'L' => LOWERCASE,
                'D' => DIGITS,
                'S' => SPECIAL,
                _ => {
                    let error = format!(
                        "Invalid pattern character: '{}'. Only U, L, D, S are allowed",
                        c
                    );
                    LoggingManager::error(&error);
                    return Err(GeneratorError::InvalidPatternCharacter(c));
                }
            };

            let mut bytes = [0u8; 4];
            rng.fill(&mut bytes).map_err(|e| {
                let error = format!("Failed to fill random bytes: {}", e);
                LoggingManager::error(&error);
                GeneratorError::RandomBytesError(error)
            })?;
            let random_u32 = u32::from_be_bytes(bytes);
            let idx = (random_u32 as usize) % charset.len();
            password.push(charset[idx] as char);
        }

        LoggingManager::info("Successfully generated pattern password");
        Ok(password)
    }

    /// Get cached wordlist for a given language
    fn get_wordlist(lang: &str) -> &'static Vec<&'static str> {
        static WORDLIST_EN: OnceLock<Vec<&'static str>> = OnceLock::new();
        static WORDLIST_DE: OnceLock<Vec<&'static str>> = OnceLock::new();

        match lang {
            "de" => WORDLIST_DE.get_or_init(|| {
                include_str!("../../data/wordlist_de.txt")
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .collect()
            }),
            _ => WORDLIST_EN.get_or_init(|| {
                include_str!("../../data/wordlist_en.txt")
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .collect()
            }),
        }
    }

    /// Generate phrase based password
    ///
    /// # Arguments
    ///
    /// * `words_count`: Number of words to use in the phrase
    ///
    /// # Returns
    ///
    /// Returns the generated password phrase as String, else returns an error
    pub fn generate_phrase_password(words_count: usize) -> Result<String, GeneratorError> {
        LoggingManager::info(&format!(
            "Generating phrase password with {} words",
            words_count
        ));

        if !(3..=20).contains(&words_count) {
            LoggingManager::error(&format!(
                "Word count must be between 3 and 20, got: {}",
                words_count
            ));
            return Err(GeneratorError::InvalidWordCount);
        }

        let lang = CONFIG.language.lang.as_str();
        let words = Self::get_wordlist(lang);

        if words.is_empty() {
            LoggingManager::error("Wordlist is empty");
            return Err(GeneratorError::EmptyWordlist);
        }

        let rng = SystemRandom::new();
        let mut phrase = String::new();

        for i in 0..words_count {
            let mut bytes = [0u8; 4];
            rng.fill(&mut bytes).map_err(|e| {
                let error = format!("Failed to fill random bytes: {}", e);
                LoggingManager::error(&error);
                GeneratorError::RandomBytesError(error)
            })?;
            let random_u32 = u32::from_be_bytes(bytes);
            let idx = (random_u32 as usize) % words.len();

            if i > 0 {
                phrase.push('-');
            }
            phrase.push_str(words[idx]);
        }

        LoggingManager::info("Successfully generated phrase password");
        Ok(phrase)
    }

    /// Generate deterministic password from seed word
    ///
    /// # Arguments
    ///
    /// * `seed`: The seed word to generate the password from
    /// * `salt`: Optional salt for additional entropy (default: "kdguard")
    /// * `service`: Optional service name to derive service-specific passwords
    ///
    /// # Returns
    ///
    /// Returns the generated password as String (always 20 characters), else returns an error
    pub fn generate_deterministic_password(
        seed: &str,
        salt: Option<&str>,
        service: Option<&str>,
    ) -> Result<String, GeneratorError> {
        const DEFAULT_LENGTH: usize = 20;

        LoggingManager::info(&format!(
            "Generating deterministic password (seed length: {}, salt: {}, service: {})",
            seed.len(),
            salt.is_some(),
            service.is_some()
        ));

        if seed.is_empty() {
            LoggingManager::error("Seed cannot be empty");
            return Err(GeneratorError::EmptySeed);
        }

        let salt_bytes = salt.unwrap_or("kdguard").as_bytes();
        let seed_bytes = seed.as_bytes();

        let salt_key = hkdf::Salt::new(hkdf::HKDF_SHA256, salt_bytes);
        let prk = salt_key.extract(seed_bytes);

        const MAX_RETRIES: u32 = 1000;
        const OUTPUT_SIZE: usize = 32;

        for retry in 0..MAX_RETRIES {
            let mut output = [0u8; OUTPUT_SIZE];
            let mut info = b"kdguard-password".to_vec();
            if let Some(service_name) = service {
                info.extend_from_slice(b"-");
                info.extend_from_slice(service_name.as_bytes());
            }
            info.extend_from_slice(b"-");
            info.extend_from_slice(&retry.to_be_bytes());
            let info_slice: &[u8] = &info;
            let info_array = [info_slice];

            let okm = prk.expand(&info_array, hkdf::HKDF_SHA256).map_err(|_| {
                LoggingManager::error("Failed to expand HKDF");
                GeneratorError::HkdfExpandError
            })?;

            let output_slice = &mut output[..];
            okm.fill(output_slice).map_err(|_| {
                LoggingManager::error("Failed to fill HKDF output");
                GeneratorError::HkdfFillError
            })?;

            let mut password = String::with_capacity(DEFAULT_LENGTH);
            let offset = (retry as usize * 13) % OUTPUT_SIZE;

            for i in 0..DEFAULT_LENGTH {
                let byte_idx = (offset + i) % OUTPUT_SIZE;
                let idx = (output[byte_idx] as usize) % CHARSET.len();
                password.push(CHARSET[idx] as char);
            }

            if Self::is_valid_password(&password) {
                LoggingManager::info(&format!(
                    "Successfully generated deterministic password (retry {})",
                    retry
                ));
                return Ok(password);
            }
        }

        LoggingManager::error(
            "Failed to generate valid deterministic password after maximum retries",
        );
        Err(GeneratorError::MaxRetriesExceeded)
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
    pub fn save_to_file(passwords: Vec<String>, output_path: &Path) -> Result<(), GeneratorError> {
        LoggingManager::info(&format!(
            "Saving {} passwords to file: {}",
            passwords.len(),
            output_path.display()
        ));

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(output_path)
            .map_err(|e| {
                let error = format!("Failed to open file for writing: {}", e);
                LoggingManager::error(&error);
                GeneratorError::SaveFileError(error)
            })?;
        let header = format!(
            "Generated with kdguard\nDate: {}\nGenerated passwords:\n",
            Local::now().format("%d.%m.%Y %H:%M:%S")
        );
        file.write_all(header.as_bytes()).map_err(|e| {
            let error = format!("Failed to write header to file: {}", e);
            LoggingManager::error(&error);
            GeneratorError::SaveFileError(error)
        })?;
        for password in passwords {
            file.write_all(password.as_bytes()).map_err(|e| {
                let error = format!("Failed to write password to file: {}", e);
                LoggingManager::error(&error);
                GeneratorError::SaveFileError(error)
            })?;
            file.write_all(b"\n").map_err(|e| {
                let error = format!("Failed to write newline to file: {}", e);
                LoggingManager::error(&error);
                GeneratorError::SaveFileError(error)
            })?;
        }

        LoggingManager::info("Successfully saved passwords to file");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_password() {
        let password =
            Generator::generate_random_password(10).expect("Failed to generate password");
        assert_eq!(password.len(), 10);
        assert!(Generator::is_valid_password(&password));
    }

    #[test]
    fn test_generate_pattern_password() {
        let password =
            Generator::generate_pattern_password("UDDL").expect("Failed to generate password");
        assert_eq!(password.len(), 4)
    }

    #[test]
    fn test_is_valid_password() {
        assert!(Generator::is_valid_password("(123P@ssw0rd"));
        assert!(!Generator::is_valid_password("password"));
    }

    #[test]
    fn test_error_generate_random_password() {
        assert!(Generator::generate_random_password(7).is_err());
        assert!(Generator::generate_random_password(65).is_err());
        assert!(Generator::generate_random_password(8).is_ok());
        assert!(Generator::generate_random_password(64).is_ok());
    }

    #[test]
    fn test_generate_phrase_password() {
        let phrase = Generator::generate_phrase_password(5).expect("Failed to generate phrase");
        let words: Vec<&str> = phrase.split('-').collect();
        assert_eq!(words.len(), 5);
        assert!(!phrase.is_empty());
    }

    #[test]
    fn test_error_generate_phrase_password() {
        assert!(Generator::generate_phrase_password(2).is_err());
        assert!(Generator::generate_phrase_password(21).is_err());
        assert!(Generator::generate_phrase_password(3).is_ok());
        assert!(Generator::generate_phrase_password(20).is_ok());
    }

    #[test]
    fn test_generate_deterministic_password() {
        let password1 = Generator::generate_deterministic_password("test-seed", None, None)
            .expect("Failed to generate deterministic password");
        let password2 = Generator::generate_deterministic_password("test-seed", None, None)
            .expect("Failed to generate deterministic password");

        assert_eq!(password1.len(), 20);
        assert_eq!(password1, password2);
        assert!(Generator::is_valid_password(&password1));
    }

    #[test]
    fn test_deterministic_password_different_seeds() {
        let password1 = Generator::generate_deterministic_password("seed1", None, None)
            .expect("Failed to generate deterministic password");
        let password2 = Generator::generate_deterministic_password("seed2", None, None)
            .expect("Failed to generate deterministic password");

        assert_ne!(password1, password2);
    }

    #[test]
    fn test_deterministic_password_with_salt() {
        let password1 =
            Generator::generate_deterministic_password("test-seed", Some("salt1"), None)
                .expect("Failed to generate deterministic password");
        let password2 =
            Generator::generate_deterministic_password("test-seed", Some("salt1"), None)
                .expect("Failed to generate deterministic password");
        let password3 =
            Generator::generate_deterministic_password("test-seed", Some("salt2"), None)
                .expect("Failed to generate deterministic password");

        assert_eq!(password1, password2);
        assert_ne!(password1, password3);
    }

    #[test]
    fn test_deterministic_password_with_service() {
        let password1 =
            Generator::generate_deterministic_password("test-seed", None, Some("github"))
                .expect("Failed to generate deterministic password");
        let password2 =
            Generator::generate_deterministic_password("test-seed", None, Some("github"))
                .expect("Failed to generate deterministic password");
        let password3 =
            Generator::generate_deterministic_password("test-seed", None, Some("gitlab"))
                .expect("Failed to generate deterministic password");

        assert_eq!(password1, password2);
        assert_ne!(password1, password3);
    }

    #[test]
    fn test_error_generate_deterministic_password() {
        assert!(Generator::generate_deterministic_password("", None, None).is_err());
        assert!(Generator::generate_deterministic_password("seed", None, None).is_ok());
    }
}
