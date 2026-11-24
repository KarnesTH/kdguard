use std::{fs::OpenOptions, io::Write, num::NonZeroU32, path::Path, sync::OnceLock};

use chrono::Local;
use ring::rand::{SecureRandom, SystemRandom};
use ring::{hkdf, pbkdf2};

use crate::config::{LENGTH_RANGE, WORDS_RANGE};
use crate::errors::GeneratorError;
use crate::logging::LoggingManager;

const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const DIGITS: &[u8] = b"0123456789";
pub const SPECIAL: &[u8] = b"!@#$%^&*()-_=+";
const CHARSET: &[u8] =
    b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-_=+";

const MAX_RETRIES: u32 = 1000;
pub const DETERMINISTIC_LENGTH: usize = 20;
const PBKDF2_ITERATIONS: u32 = 600_000;
const DERIVED_BYTES: usize = 64;

struct OutputLength(usize);

impl hkdf::KeyType for OutputLength {
    fn len(&self) -> usize {
        self.0
    }
}

pub struct Generator;

impl Generator {
    /// Draws `length` characters from the charset until the password contains lowercase,
    /// uppercase, digit and special characters.
    pub fn generate_random_password(length: usize) -> Result<String, GeneratorError> {
        let (min, max) = LENGTH_RANGE;
        if !(min..=max).contains(&length) {
            return Err(GeneratorError::InvalidLength(length));
        }

        let rng = SystemRandom::new();
        for _ in 0..MAX_RETRIES {
            let mut password = String::with_capacity(length);
            for _ in 0..length {
                password.push(CHARSET[Self::random_index(&rng, CHARSET.len())?] as char);
            }
            if Self::is_valid_password(&password) {
                LoggingManager::info(&format!("Generated random password, length {length}"));
                return Ok(password);
            }
        }
        Err(GeneratorError::MaxRetriesExceeded)
    }

    /// Draws one character per pattern letter: `U` uppercase, `L` lowercase, `D` digit,
    /// `S` special character.
    pub fn generate_pattern_password(pattern: &str) -> Result<String, GeneratorError> {
        Self::check_pattern(pattern)?;
        let rng = SystemRandom::new();
        let mut password = String::with_capacity(pattern.len());
        for c in pattern.chars() {
            let charset = match c {
                'U' => UPPERCASE,
                'L' => LOWERCASE,
                'D' => DIGITS,
                _ => SPECIAL,
            };
            password.push(charset[Self::random_index(&rng, charset.len())?] as char);
        }
        LoggingManager::info(&format!(
            "Generated pattern password, length {}",
            pattern.len()
        ));
        Ok(password)
    }

    /// Returns an error for an empty pattern or a letter other than `U`, `L`, `D` and `S`.
    pub fn check_pattern(pattern: &str) -> Result<(), GeneratorError> {
        if pattern.is_empty() {
            return Err(GeneratorError::EmptyPattern);
        }
        match pattern
            .chars()
            .find(|c| !matches!(c, 'U' | 'L' | 'D' | 'S'))
        {
            Some(c) => Err(GeneratorError::InvalidPatternCharacter(c)),
            None => Ok(()),
        }
    }

    /// Joins `words_count` random words of the wordlist for `lang` (`de`, otherwise English)
    /// with `-`.
    pub fn generate_phrase_password(
        words_count: usize,
        lang: &str,
    ) -> Result<String, GeneratorError> {
        let (min, max) = WORDS_RANGE;
        if !(min..=max).contains(&words_count) {
            return Err(GeneratorError::InvalidWordCount(words_count));
        }

        let words = Self::wordlist(lang);
        let rng = SystemRandom::new();
        let mut phrase: Vec<&str> = Vec::with_capacity(words_count);
        for _ in 0..words_count {
            phrase.push(words[Self::random_index(&rng, words.len())?]);
        }
        LoggingManager::info(&format!("Generated phrase password, {words_count} words"));
        Ok(phrase.join("-"))
    }

    /// Derives a 20 character password from the seed: PBKDF2-HMAC-SHA256 with 600 000
    /// iterations turns seed and salt into a master key, HKDF-SHA256 expands it per service and
    /// attempt into 64 bytes. Bytes below 228 map to the charset (`byte % 76`), larger ones are
    /// skipped. The first attempt that yields a valid password is returned.
    pub fn generate_deterministic_password(
        seed: &str,
        salt: Option<&str>,
        service: Option<&str>,
    ) -> Result<String, GeneratorError> {
        if seed.is_empty() {
            return Err(GeneratorError::EmptySeed);
        }

        let iterations = NonZeroU32::new(PBKDF2_ITERATIONS).ok_or(GeneratorError::KeyDerivation)?;
        let pbkdf2_salt = format!("kdguard-v2:{}", salt.unwrap_or(""));
        let mut master_key = [0u8; 32];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            iterations,
            pbkdf2_salt.as_bytes(),
            seed.as_bytes(),
            &mut master_key,
        );
        let prk = hkdf::Prk::new_less_safe(hkdf::HKDF_SHA256, &master_key);

        let limit = (256 / CHARSET.len()) * CHARSET.len();
        for attempt in 0..MAX_RETRIES {
            let mut info = b"kdguard-password\0".to_vec();
            info.extend_from_slice(service.unwrap_or("").as_bytes());
            info.push(0);
            info.extend_from_slice(&attempt.to_be_bytes());
            let info_parts = [info.as_slice()];

            let mut output = [0u8; DERIVED_BYTES];
            prk.expand(&info_parts, OutputLength(DERIVED_BYTES))
                .and_then(|okm| okm.fill(&mut output))
                .map_err(|_| GeneratorError::KeyDerivation)?;

            let password: String = output
                .iter()
                .filter(|byte| (**byte as usize) < limit)
                .take(DETERMINISTIC_LENGTH)
                .map(|byte| CHARSET[*byte as usize % CHARSET.len()] as char)
                .collect();

            if password.len() == DETERMINISTIC_LENGTH && Self::is_valid_password(&password) {
                LoggingManager::info(&format!(
                    "Generated deterministic password (service: {}, salt: {})",
                    service.is_some(),
                    salt.is_some()
                ));
                return Ok(password);
            }
        }
        Err(GeneratorError::MaxRetriesExceeded)
    }

    /// Writes a header with the date and one password per line to a new file. The file is only
    /// readable by the owner (Unix: mode 0600). An existing file is replaced only with `overwrite`.
    pub fn save_to_file(
        passwords: &[String],
        path: &Path,
        overwrite: bool,
    ) -> Result<(), GeneratorError> {
        let mut options = OpenOptions::new();
        options.write(true);
        if overwrite {
            options.create(true).truncate(true);
        } else {
            options.create_new(true);
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }

        let mut file = options.open(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                GeneratorError::FileExists(path.to_path_buf())
            } else {
                GeneratorError::SaveFile(path.to_path_buf(), e.to_string())
            }
        })?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
                .map_err(|e| GeneratorError::SaveFile(path.to_path_buf(), e.to_string()))?;
        }

        let mut content = format!(
            "Generated with kdguard\nDate: {}\nGenerated passwords:\n",
            Local::now().format("%d.%m.%Y %H:%M:%S")
        );
        for password in passwords {
            content.push_str(password);
            content.push('\n');
        }
        file.write_all(content.as_bytes())
            .map_err(|e| GeneratorError::SaveFile(path.to_path_buf(), e.to_string()))?;

        LoggingManager::info(&format!("Saved {} passwords to a file", passwords.len()));
        Ok(())
    }

    /// Returns a uniformly distributed index below `len`: random u32 values at or above the
    /// largest multiple of `len` are drawn again.
    fn random_index(rng: &SystemRandom, len: usize) -> Result<usize, GeneratorError> {
        let len = len as u32;
        let zone = (u32::MAX / len) * len;
        loop {
            let mut bytes = [0u8; 4];
            rng.fill(&mut bytes)
                .map_err(|_| GeneratorError::RandomBytes)?;
            let value = u32::from_be_bytes(bytes);
            if value < zone {
                return Ok((value % len) as usize);
            }
        }
    }

    /// Returns the cached wordlist for `de`, otherwise the English one.
    fn wordlist(lang: &str) -> &'static [&'static str] {
        static WORDLIST_EN: OnceLock<Vec<&'static str>> = OnceLock::new();
        static WORDLIST_DE: OnceLock<Vec<&'static str>> = OnceLock::new();
        let (cell, content) = match lang {
            "de" => (&WORDLIST_DE, include_str!("../../data/wordlist_de.txt")),
            _ => (&WORDLIST_EN, include_str!("../../data/wordlist_en.txt")),
        };
        cell.get_or_init(|| {
            content
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .collect()
        })
    }

    /// Returns true when the password contains a lowercase letter, an uppercase letter, a digit
    /// and a special character of the charset.
    fn is_valid_password(password: &str) -> bool {
        let bytes = password.as_bytes();
        bytes.iter().any(u8::is_ascii_lowercase)
            && bytes.iter().any(u8::is_ascii_uppercase)
            && bytes.iter().any(u8::is_ascii_digit)
            && bytes.iter().any(|byte| SPECIAL.contains(byte))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_password() {
        let password = Generator::generate_random_password(10).unwrap();
        assert_eq!(password.len(), 10);
        assert!(Generator::is_valid_password(&password));
    }

    #[test]
    fn test_random_password_length_range() {
        assert!(Generator::generate_random_password(7).is_err());
        assert!(Generator::generate_random_password(65).is_err());
        assert!(Generator::generate_random_password(8).is_ok());
        assert!(Generator::generate_random_password(64).is_ok());
    }

    #[test]
    fn test_generate_pattern_password() {
        let password = Generator::generate_pattern_password("UULDDS").unwrap();
        let chars: Vec<char> = password.chars().collect();
        assert_eq!(chars.len(), 6);
        assert!(chars[0].is_ascii_uppercase() && chars[1].is_ascii_uppercase());
        assert!(chars[2].is_ascii_lowercase());
        assert!(chars[3].is_ascii_digit() && chars[4].is_ascii_digit());
        assert!(SPECIAL.contains(&(chars[5] as u8)));
    }

    #[test]
    fn test_pattern_errors() {
        assert!(matches!(
            Generator::generate_pattern_password(""),
            Err(GeneratorError::EmptyPattern)
        ));
        assert!(matches!(
            Generator::generate_pattern_password("ULX"),
            Err(GeneratorError::InvalidPatternCharacter('X'))
        ));
    }

    #[test]
    fn test_generate_phrase_password() {
        let phrase = Generator::generate_phrase_password(5, "en").unwrap();
        assert_eq!(phrase.split('-').count(), 5);
        let phrase = Generator::generate_phrase_password(3, "de").unwrap();
        assert_eq!(phrase.split('-').count(), 3);
        assert!(Generator::generate_phrase_password(2, "en").is_err());
        assert!(Generator::generate_phrase_password(21, "en").is_err());
    }

    #[test]
    fn test_wordlists_are_complete() {
        assert_eq!(Generator::wordlist("en").len(), 7776);
        assert_eq!(Generator::wordlist("de").len(), 7776);
    }

    #[test]
    fn test_random_index_stays_in_range() {
        let rng = SystemRandom::new();
        for _ in 0..1000 {
            assert!(Generator::random_index(&rng, 76).unwrap() < 76);
        }
    }

    #[test]
    fn test_deterministic_password() {
        let first = Generator::generate_deterministic_password("test-seed", None, None).unwrap();
        let second = Generator::generate_deterministic_password("test-seed", None, None).unwrap();
        assert_eq!(first.len(), DETERMINISTIC_LENGTH);
        assert_eq!(first, second);
        assert!(Generator::is_valid_password(&first));
    }

    #[test]
    fn test_deterministic_inputs_change_password() {
        let base = Generator::generate_deterministic_password("seed", Some("salt"), Some("github"))
            .unwrap();
        let other_seed =
            Generator::generate_deterministic_password("seed2", Some("salt"), Some("github"))
                .unwrap();
        let other_salt =
            Generator::generate_deterministic_password("seed", Some("salt2"), Some("github"))
                .unwrap();
        let other_service =
            Generator::generate_deterministic_password("seed", Some("salt"), Some("gitlab"))
                .unwrap();
        assert_ne!(base, other_seed);
        assert_ne!(base, other_salt);
        assert_ne!(base, other_service);
    }

    #[test]
    fn test_deterministic_empty_seed() {
        assert!(matches!(
            Generator::generate_deterministic_password("", None, None),
            Err(GeneratorError::EmptySeed)
        ));
    }

    #[test]
    fn test_is_valid_password() {
        assert!(Generator::is_valid_password("(123P@ssw0rd"));
        assert!(!Generator::is_valid_password("password"));
    }

    #[test]
    fn test_save_to_file() {
        let path = std::env::temp_dir().join(format!("kdguard_test_{}.txt", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let passwords = vec!["a1B!aaaa".to_string(), "b2C@bbbb".to_string()];

        Generator::save_to_file(&passwords, &path, false).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.ends_with("a1B!aaaa\nb2C@bbbb\n"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }

        assert!(matches!(
            Generator::save_to_file(&passwords, &path, false),
            Err(GeneratorError::FileExists(_))
        ));
        assert!(Generator::save_to_file(&passwords, &path, true).is_ok());
        std::fs::remove_file(&path).unwrap();
    }
}
