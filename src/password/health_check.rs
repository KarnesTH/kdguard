use lingua_i18n_rs::prelude::Lingua;

use crate::errors::HealthCheckError;
use crate::logging::LoggingManager;

pub struct HealthCheck;

const COMMON_PASSWORDS: &str = include_str!("../../data/10k-most-common-passwords.txt");

#[derive(Debug, Clone)]
pub struct PasswordScore {
    pub total: u32,
    pub length_score: u32,
    pub diversity_score: u32,
    pub complexity_score: u32,
    pub entropy_score: u32,
}

#[derive(Debug, Clone)]
pub struct PasswordAnalysis {
    pub score: PasswordScore,
    pub rating: String,
    pub has_lowercase: bool,
    pub has_uppercase: bool,
    pub has_digit: bool,
    pub has_special: bool,
    pub length: usize,
    pub entropy: f64,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

impl HealthCheck {
    /// Check a given password
    ///
    /// # Arguments
    ///
    /// * `password`: The password to check
    /// * `detailed`: Whether to show detailed analysis
    ///
    /// # Returns
    ///
    /// Returns the analysis of the password
    pub fn check_password(password: &str, detailed: bool) -> Result<(), HealthCheckError> {
        LoggingManager::info(&format!(
            "Checking password health (detailed: {})",
            detailed
        ));

        let analysis = Self::analyze_password(password);

        LoggingManager::info(&format!(
            "Password analysis completed: rating={}, score={}, length={}, entropy={:.2}",
            analysis.rating, analysis.score.total, analysis.length, analysis.entropy
        ));

        Self::print_result(&analysis, detailed);

        Ok(())
    }

    /// Analyze a given password
    ///
    /// # Arguments
    ///
    /// * `password`: The password to analyze
    ///
    /// # Returns
    ///
    /// Returns the analysis of the password
    fn analyze_password(password: &str) -> PasswordAnalysis {
        let length = password.len();
        let length_score = Self::calculate_length_score(length);
        let (diversity_score, has_lowercase, has_uppercase, has_digit, has_special) =
            Self::calculate_diversity_score(password);
        let complexity_score = Self::calculate_complexity_score(password);
        let (entropy_score, entropy) = Self::calculate_entropy_score(password);

        let total = length_score + diversity_score + complexity_score + entropy_score;
        let rating = Self::score_to_rating(total);

        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();

        if length < 8 {
            warnings.push(Lingua::t("commands.check.warnings.password_too_short", &[]).unwrap());
            suggestions
                .push(Lingua::t("commands.check.suggestions.password_to_short", &[]).unwrap());
        }

        if !has_lowercase {
            warnings.push(Lingua::t("commands.check.warnings.no_lowercase", &[]).unwrap());
            suggestions.push(Lingua::t("commands.check.suggestions.add_lowercase", &[]).unwrap());
        }

        if !has_uppercase {
            warnings.push(Lingua::t("commands.check.warnings.no_uppercase", &[]).unwrap());
            suggestions.push(Lingua::t("commands.check.suggestions.add_uppercase", &[]).unwrap());
        }

        if !has_digit {
            warnings.push(Lingua::t("commands.check.warnings.no_digits", &[]).unwrap());
            suggestions.push(Lingua::t("commands.check.suggestions.add_digits", &[]).unwrap());
        }

        if !has_special {
            warnings.push(Lingua::t("commands.check.warnings.no_special", &[]).unwrap());
            suggestions.push(Lingua::t("commands.check.suggestions.add_special", &[]).unwrap());
        }

        if Self::has_common_patterns(password) {
            warnings.push(Lingua::t("commands.check.warnings.common_patterns", &[]).unwrap());
            suggestions
                .push(Lingua::t("commands.check.suggestions.avoid_simple_sequences", &[]).unwrap());
        }

        if Self::has_repetitions(password) {
            warnings.push(Lingua::t("commands.check.warnings.repetitions", &[]).unwrap());
            suggestions
                .push(Lingua::t("commands.check.suggestions.avoid_repetitions", &[]).unwrap());
        }

        PasswordAnalysis {
            score: PasswordScore {
                total,
                length_score,
                diversity_score,
                complexity_score,
                entropy_score,
            },
            rating,
            has_lowercase,
            has_uppercase,
            has_digit,
            has_special,
            length,
            entropy,
            warnings,
            suggestions,
        }
    }

    /// Calculate the length score of a given password
    ///
    /// # Arguments
    ///
    /// * `length`: The length of the password
    ///
    /// # Returns
    ///
    /// Returns the length score
    fn calculate_length_score(length: usize) -> u32 {
        match length {
            0..=7 => 0,
            8..=12 => 10,
            13..=16 => 20,
            _ => 25,
        }
    }

    /// Calculate the diversity score of a given password
    ///
    /// # Arguments
    ///
    /// * `password`: The password to analyze
    ///
    /// # Returns
    ///
    /// Returns the diversity score
    fn calculate_diversity_score(password: &str) -> (u32, bool, bool, bool, bool) {
        let mut has_lowercase = false;
        let mut has_uppercase = false;
        let mut has_digit = false;
        let mut has_special = false;

        let special_chars: &[char] = &[
            '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '_', '=', '+',
        ];

        for c in password.chars() {
            if c.is_lowercase() {
                has_lowercase = true;
            } else if c.is_uppercase() {
                has_uppercase = true;
            } else if c.is_ascii_digit() {
                has_digit = true;
            } else if special_chars.contains(&c) {
                has_special = true;
            }
        }

        let mut score = 0;
        if has_lowercase {
            score += 5;
        }
        if has_uppercase {
            score += 5;
        }
        if has_digit {
            score += 5;
        }
        if has_special {
            score += 5;
        }

        if has_lowercase && has_uppercase && has_digit && has_special {
            score += 10;
        }

        (score, has_lowercase, has_uppercase, has_digit, has_special)
    }

    /// Calculate the complexity score of a given password
    ///
    /// # Arguments
    ///
    /// * `password`: The password to analyze
    ///
    /// # Returns
    ///
    /// Returns the complexity score
    fn calculate_complexity_score(password: &str) -> u32 {
        let mut score = 0;

        if !Self::has_common_patterns(password) {
            score += 15;
        }

        if !Self::has_repetitions(password) {
            score += 10;
        }

        score
    }

    /// Check if a given password has common patterns
    ///
    /// Checks if the password (or parts of it) appear in the common passwords list
    ///
    /// # Arguments
    ///
    /// * `password`: The password to check
    ///
    /// # Returns
    ///
    /// Returns true if the password has common patterns, otherwise false
    fn has_common_patterns(password: &str) -> bool {
        let password_lower = password.to_lowercase();

        for line in COMMON_PASSWORDS.lines() {
            let common_pw = line.trim().to_lowercase();
            if common_pw.is_empty() {
                continue;
            }

            if password_lower == common_pw
                || password_lower.contains(&common_pw)
                || common_pw.contains(&password_lower)
            {
                return true;
            }
        }

        false
    }

    /// Check if a given password has repetitions
    ///
    /// # Arguments
    ///
    /// * `password`: The password to check
    ///
    /// # Returns
    ///
    /// Returns true if the password has repetitions, otherwise false
    fn has_repetitions(password: &str) -> bool {
        let chars: Vec<char> = password.chars().collect();
        if chars.len() < 3 {
            return false;
        }

        for i in 0..chars.len() - 2 {
            if chars[i] == chars[i + 1] && chars[i + 1] == chars[i + 2] {
                return true;
            }
        }

        false
    }

    /// Calculate the entropy score of a given password
    ///
    /// # Arguments
    ///
    /// * `password`: The password to analyze
    ///
    /// # Returns
    ///
    /// Returns the entropy score
    fn calculate_entropy_score(password: &str) -> (u32, f64) {
        let mut charset_size = 0;

        let mut has_lowercase = false;
        let mut has_uppercase = false;
        let mut has_digit = false;
        let mut has_special = false;

        let special_chars: &[char] = &[
            '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '_', '=', '+',
        ];

        for c in password.chars() {
            if c.is_lowercase() && !has_lowercase {
                charset_size += 26;
                has_lowercase = true;
            } else if c.is_uppercase() && !has_uppercase {
                charset_size += 26;
                has_uppercase = true;
            } else if c.is_ascii_digit() && !has_digit {
                charset_size += 10;
                has_digit = true;
            } else if special_chars.contains(&c) && !has_special {
                charset_size += special_chars.len();
                has_special = true;
            }
        }

        if charset_size == 0 {
            return (0, 0.0);
        }

        let length = password.len() as f64;
        let entropy = length * (charset_size as f64).log2();

        let score = match entropy {
            e if e < 30.0 => 5,
            e if e < 40.0 => 10,
            e if e < 50.0 => 15,
            _ => 20,
        };

        (score, entropy)
    }

    /// Convert a score to a rating
    ///
    /// # Arguments
    ///
    /// * `score`: The score to convert
    ///
    /// # Returns
    ///
    /// Returns the rating
    fn score_to_rating(score: u32) -> String {
        match score {
            0..=40 => Lingua::t("commands.check.score_rating.weak", &[]).unwrap(),
            41..=60 => Lingua::t("commands.check.score_rating.medium", &[]).unwrap(),
            61..=80 => Lingua::t("commands.check.score_rating.strong", &[]).unwrap(),
            _ => Lingua::t("commands.check.score_rating.very_strong", &[]).unwrap(),
        }
    }

    /// Print the result of a password analysis
    ///
    /// # Arguments
    ///
    /// * `analysis`: The analysis to print
    /// * `detailed`: Whether to show detailed analysis
    ///
    /// # Returns
    ///
    /// Returns nothing
    fn print_result(analysis: &PasswordAnalysis, detailed: bool) {
        let weak = Lingua::t("commands.check.score_rating.weak", &[]).unwrap();
        let medium = Lingua::t("commands.check.score_rating.medium", &[]).unwrap();
        let strong = Lingua::t("commands.check.score_rating.strong", &[]).unwrap();
        let very_strong = Lingua::t("commands.check.score_rating.very_strong", &[]).unwrap();

        let color = match analysis.rating.as_str() {
            s if s == weak => "\x1b[1;31m",
            s if s == medium => "\x1b[1;33m",
            s if s == strong => "\x1b[1;32m",
            s if s == very_strong => "\x1b[1;32m",
            _ => "\x1b[0m",
        };

        println!(
            "\n\x1b[1;36m{}\x1b[0m",
            Lingua::t("commands.check.title", &[]).unwrap()
        );
        println!("{}", "=".repeat(50));
        println!(
            "{}",
            Lingua::t(
                "commands.check.rating",
                &[
                    (
                        "rating",
                        format!("{}{}\x1b[0m", color, analysis.rating).as_str()
                    ),
                    ("points", analysis.score.total.to_string().as_str())
                ]
            )
            .unwrap()
        );
        println!(
            "{}",
            Lingua::t(
                "commands.check.length",
                &[("length", analysis.length.to_string().as_str())]
            )
            .unwrap()
        );

        if detailed {
            println!(
                "\n\x1b[1;33m{}\x1b[0m",
                Lingua::t("commands.check.subtitle_detailed", &[]).unwrap()
            );
            println!(
                "  {}",
                Lingua::t(
                    "commands.check.length_score",
                    &[(
                        "length_score",
                        analysis.score.length_score.to_string().as_str()
                    )]
                )
                .unwrap()
            );
            println!(
                "  {}",
                Lingua::t(
                    "commands.check.diversity_score",
                    &[(
                        "diversity_score",
                        analysis.score.diversity_score.to_string().as_str()
                    )]
                )
                .unwrap()
            );
            println!(
                "  {}",
                Lingua::t(
                    "commands.check.complexity_score",
                    &[(
                        "complexity_score",
                        analysis.score.complexity_score.to_string().as_str()
                    )]
                )
                .unwrap()
            );
            println!(
                "  {}",
                Lingua::t(
                    "commands.check.entropy_score",
                    &[(
                        "entropy_score",
                        analysis.score.entropy_score.to_string().as_str()
                    )]
                )
                .unwrap()
            );
            println!(
                "  {}",
                Lingua::t(
                    "commands.check.entropy",
                    &[("entropy", format!("{:.2}", analysis.entropy).as_str())]
                )
                .unwrap()
            );

            println!(
                "\n\x1b[1;33m{}\x1b[0m",
                Lingua::t("commands.check.subtitle_categories", &[]).unwrap()
            );
            println!(
                "  {}",
                Lingua::t(
                    "commands.check.lowercase",
                    &[(
                        "lowercase",
                        format!(
                            "{}\x1b[0m",
                            if analysis.has_lowercase {
                                "\x1b[1;32m✓"
                            } else {
                                "\x1b[1;31m✗"
                            }
                        )
                        .as_str()
                    )]
                )
                .unwrap()
            );
            println!(
                "  {}",
                Lingua::t(
                    "commands.check.uppercase",
                    &[(
                        "uppercase",
                        format!(
                            "{}\x1b[0m",
                            if analysis.has_uppercase {
                                "\x1b[1;32m✓"
                            } else {
                                "\x1b[1;31m✗"
                            }
                        )
                        .as_str()
                    )]
                )
                .unwrap()
            );
            println!(
                "  {}",
                Lingua::t(
                    "commands.check.digits",
                    &[(
                        "digits",
                        format!(
                            "{}\x1b[0m",
                            if analysis.has_digit {
                                "\x1b[1;32m✓"
                            } else {
                                "\x1b[1;31m✗"
                            }
                        )
                        .as_str()
                    )]
                )
                .unwrap()
            );
            println!(
                "  {}",
                Lingua::t(
                    "commands.check.special",
                    &[(
                        "special",
                        format!(
                            "{}\x1b[0m",
                            if analysis.has_special {
                                "\x1b[1;32m✓"
                            } else {
                                "\x1b[1;31m✗"
                            }
                        )
                        .as_str()
                    )]
                )
                .unwrap()
            );

            if !analysis.warnings.is_empty() {
                println!(
                    "\n\x1b[1;31m{}\x1b[0m",
                    Lingua::t("commands.check.warnings.title", &[]).unwrap()
                );
                for warning in &analysis.warnings {
                    println!("  ⚠️\t{}", warning);
                }
            }

            if !analysis.suggestions.is_empty() && analysis.score.total < 80 {
                println!(
                    "\n\x1b[1;33m{}\x1b[0m",
                    Lingua::t("commands.check.suggestions.title", &[]).unwrap()
                );
                for suggestion in &analysis.suggestions {
                    println!("  💡\t{}", suggestion);
                }
            }
        }

        println!("{}", "=".repeat(50));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_length_score() {
        assert_eq!(HealthCheck::calculate_length_score(5), 0);
        assert_eq!(HealthCheck::calculate_length_score(8), 10);
        assert_eq!(HealthCheck::calculate_length_score(12), 10);
        assert_eq!(HealthCheck::calculate_length_score(13), 20);
        assert_eq!(HealthCheck::calculate_length_score(16), 20);
        assert_eq!(HealthCheck::calculate_length_score(20), 25);
    }

    #[test]
    fn test_calculate_diversity_score() {
        let (score, has_lower, has_upper, has_digit, has_special) =
            HealthCheck::calculate_diversity_score("Abc123!");
        assert!(has_lower);
        assert!(has_upper);
        assert!(has_digit);
        assert!(has_special);
        assert_eq!(score, 30);

        let (score2, _, _, _, _) = HealthCheck::calculate_diversity_score("abc");
        assert_eq!(score2, 5);
    }

    #[test]
    fn test_has_repetitions() {
        assert!(HealthCheck::has_repetitions("aaa"));
        assert!(HealthCheck::has_repetitions("abc111"));
        assert!(!HealthCheck::has_repetitions("abc123"));
        assert!(!HealthCheck::has_repetitions("ab"));
    }

    #[test]
    fn test_has_common_patterns() {
        assert!(HealthCheck::has_common_patterns("password"));
        assert!(HealthCheck::has_common_patterns("123456"));
        assert!(!HealthCheck::has_common_patterns("Xy9$mK2@nP7#qW"));
    }

    #[test]
    fn test_calculate_entropy_score() {
        let (score, entropy) = HealthCheck::calculate_entropy_score("Abc123!");
        assert!(entropy > 0.0);
        assert!(score > 0);

        let (score2, entropy2) = HealthCheck::calculate_entropy_score("");
        assert_eq!(score2, 0);
        assert_eq!(entropy2, 0.0);
    }

    fn init_lingua_for_tests() {
        use std::sync::Once;
        static INIT: Once = Once::new();

        INIT.call_once(|| {
            if let Ok(languages_path) = crate::config::Config::get_languages_path() {
                if let Some(path_str) = languages_path.to_str() {
                    let lingua = Lingua::new(path_str);
                    let _ = lingua.init();
                    let _ = Lingua::set_language("en");
                }
            }
        });
    }

    #[test]
    fn test_score_to_rating() {
        init_lingua_for_tests();

        let rating = HealthCheck::score_to_rating(30);
        assert!(!rating.is_empty());

        let rating2 = HealthCheck::score_to_rating(50);
        assert!(!rating2.is_empty());

        let rating3 = HealthCheck::score_to_rating(70);
        assert!(!rating3.is_empty());

        let rating4 = HealthCheck::score_to_rating(90);
        assert!(!rating4.is_empty());
    }

    #[test]
    fn test_analyze_password() {
        init_lingua_for_tests();

        let analysis = HealthCheck::analyze_password("Test123!");
        assert_eq!(analysis.length, 8);
        assert!(analysis.has_lowercase);
        assert!(analysis.has_uppercase);
        assert!(analysis.has_digit);
        assert!(analysis.has_special);
        assert!(analysis.score.total > 0);
    }
}
