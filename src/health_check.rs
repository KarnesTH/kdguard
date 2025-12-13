pub struct HealthCheck;

const COMMON_PASSWORDS: &str = include_str!("../data/10k-most-common-passwords.txt");

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
    pub fn check_password(
        password: &str,
        detailed: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let analysis = Self::analyze_password(password);

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
            warnings.push("Password is too short".to_string());
            suggestions.push("Use at least 8 characters".to_string());
        }

        if !has_lowercase {
            warnings.push("No lowercase letters present".to_string());
            suggestions.push("Add lowercase letters".to_string());
        }

        if !has_uppercase {
            warnings.push("No uppercase letters present".to_string());
            suggestions.push("Add uppercase letters".to_string());
        }

        if !has_digit {
            warnings.push("No digits present".to_string());
            suggestions.push("Add digits".to_string());
        }

        if !has_special {
            warnings.push("No special characters present".to_string());
            suggestions.push("Add special characters".to_string());
        }

        if Self::has_common_patterns(password) {
            warnings.push("Common patterns detected".to_string());
            suggestions.push("Avoid simple sequences like '123' or 'abc'".to_string());
        }

        if Self::has_repetitions(password) {
            warnings.push("Repetitions detected".to_string());
            suggestions.push("Avoid character repetitions".to_string());
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
            0..=40 => "weak".to_string(),
            41..=60 => "medium".to_string(),
            61..=80 => "strong".to_string(),
            _ => "very strong".to_string(),
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
        let color = match analysis.rating.as_str() {
            "weak" => "\x1b[1;31m",
            "medium" => "\x1b[1;33m",
            "strong" => "\x1b[1;32m",
            "very strong" => "\x1b[1;32m",
            _ => "\x1b[0m",
        };

        println!("\n\x1b[1;36mPassword Health Check\x1b[0m");
        println!("{}", "=".repeat(50));
        println!(
            "Rating: {}{}\x1b[0m ({} points)",
            color, analysis.rating, analysis.score.total
        );
        println!("Length: {} characters", analysis.length);

        if detailed {
            println!("\n\x1b[1;33mDetailed Analysis:\x1b[0m");
            println!("  Length Score: {} / 25", analysis.score.length_score);
            println!(
                "  Character Diversity Score: {} / 30",
                analysis.score.diversity_score
            );
            println!(
                "  Complexity Score: {} / 25",
                analysis.score.complexity_score
            );
            println!("  Entropy Score: {} / 20", analysis.score.entropy_score);
            println!("  Entropy: {:.2} bits", analysis.entropy);

            println!("\n\x1b[1;33mCharacter Categories:\x1b[0m");
            println!(
                "  Lowercase: {}",
                if analysis.has_lowercase {
                    "\x1b[1;32m✓\x1b[0m"
                } else {
                    "\x1b[1;31m✗\x1b[0m"
                }
            );
            println!(
                "  Uppercase: {}",
                if analysis.has_uppercase {
                    "\x1b[1;32m✓\x1b[0m"
                } else {
                    "\x1b[1;31m✗\x1b[0m"
                }
            );
            println!(
                "  Digits: {}",
                if analysis.has_digit {
                    "\x1b[1;32m✓\x1b[0m"
                } else {
                    "\x1b[1;31m✗\x1b[0m"
                }
            );
            println!(
                "  Special Characters: {}",
                if analysis.has_special {
                    "\x1b[1;32m✓\x1b[0m"
                } else {
                    "\x1b[1;31m✗\x1b[0m"
                }
            );

            if !analysis.warnings.is_empty() {
                println!("\n\x1b[1;31mWarnings:\x1b[0m");
                for warning in &analysis.warnings {
                    println!("  ⚠️\t{}", warning);
                }
            }

            if !analysis.suggestions.is_empty() && analysis.score.total < 80 {
                println!("\n\x1b[1;33mSuggestions:\x1b[0m");
                for suggestion in &analysis.suggestions {
                    println!("  💡\t{}", suggestion);
                }
            }
        }

        println!("{}", "=".repeat(50));
    }
}
