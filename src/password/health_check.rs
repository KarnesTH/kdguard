use std::sync::OnceLock;

use crate::password::generator::SPECIAL;

const COMMON_PASSWORDS: &str = include_str!("../../data/10k-most-common-passwords.txt");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rating {
    Weak,
    Medium,
    Strong,
    VeryStrong,
}

impl Rating {
    /// Returns the translation key below `commands.check.rating`.
    pub fn key(&self) -> &'static str {
        match self {
            Rating::Weak => "weak",
            Rating::Medium => "medium",
            Rating::Strong => "strong",
            Rating::VeryStrong => "very_strong",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Issue {
    TooShort,
    NoLowercase,
    NoUppercase,
    NoDigits,
    NoSpecial,
    CommonPassword,
    CommonPattern,
    Repetitions,
}

impl Issue {
    /// Returns the translation key below `commands.check.issues`.
    pub fn key(&self) -> &'static str {
        match self {
            Issue::TooShort => "too_short",
            Issue::NoLowercase => "no_lowercase",
            Issue::NoUppercase => "no_uppercase",
            Issue::NoDigits => "no_digits",
            Issue::NoSpecial => "no_special",
            Issue::CommonPassword => "common_password",
            Issue::CommonPattern => "common_pattern",
            Issue::Repetitions => "repetitions",
        }
    }
}

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
    pub rating: Rating,
    pub has_lowercase: bool,
    pub has_uppercase: bool,
    pub has_digit: bool,
    pub has_special: bool,
    pub length: usize,
    pub entropy: f64,
    pub issues: Vec<Issue>,
}

struct CharacterClasses {
    lowercase: bool,
    uppercase: bool,
    digit: bool,
    special: bool,
    other: bool,
}

pub struct HealthCheck;

impl HealthCheck {
    /// Scores the password from length (0 to 25), character classes (0 to 30), common
    /// passwords and repetitions (0 to 25) and entropy (0 to 20) and lists the found issues.
    pub fn analyze_password(password: &str) -> PasswordAnalysis {
        let length = password.chars().count();
        let classes = Self::character_classes(password);
        let common = Self::common_match(password);
        let repetitions = Self::has_repetitions(password);
        let entropy = Self::entropy(length, &classes);

        let length_score = Self::length_score(length);
        let diversity_score = Self::diversity_score(&classes);
        let complexity_score = match common {
            Some(_) => 0,
            None => 15,
        } + if repetitions { 0 } else { 10 };
        let entropy_score = Self::entropy_score(entropy);
        let mut total = length_score + diversity_score + complexity_score + entropy_score;
        if common == Some(Issue::CommonPassword) {
            total = total.min(20);
        }

        let mut issues = Vec::new();
        if length < 8 {
            issues.push(Issue::TooShort);
        }
        if !classes.lowercase {
            issues.push(Issue::NoLowercase);
        }
        if !classes.uppercase {
            issues.push(Issue::NoUppercase);
        }
        if !classes.digit {
            issues.push(Issue::NoDigits);
        }
        if !classes.special {
            issues.push(Issue::NoSpecial);
        }
        if let Some(issue) = common {
            issues.push(issue);
        }
        if repetitions {
            issues.push(Issue::Repetitions);
        }

        PasswordAnalysis {
            score: PasswordScore {
                total,
                length_score,
                diversity_score,
                complexity_score,
                entropy_score,
            },
            rating: Self::rating(total),
            has_lowercase: classes.lowercase,
            has_uppercase: classes.uppercase,
            has_digit: classes.digit,
            has_special: classes.special,
            length,
            entropy,
            issues,
        }
    }

    fn character_classes(password: &str) -> CharacterClasses {
        let mut classes = CharacterClasses {
            lowercase: false,
            uppercase: false,
            digit: false,
            special: false,
            other: false,
        };
        for c in password.chars() {
            if c.is_lowercase() {
                classes.lowercase = true;
            } else if c.is_uppercase() {
                classes.uppercase = true;
            } else if c.is_ascii_digit() {
                classes.digit = true;
            } else if c.is_ascii() && SPECIAL.contains(&(c as u8)) {
                classes.special = true;
            } else {
                classes.other = true;
            }
        }
        classes
    }

    fn length_score(length: usize) -> u32 {
        match length {
            0..=7 => 0,
            8..=12 => 10,
            13..=16 => 20,
            _ => 25,
        }
    }

    /// 5 points per present class (lowercase, uppercase, digit, special), 10 more when all four
    /// are present.
    fn diversity_score(classes: &CharacterClasses) -> u32 {
        let present = [
            classes.lowercase,
            classes.uppercase,
            classes.digit,
            classes.special,
        ]
        .iter()
        .filter(|present| **present)
        .count() as u32;
        present * 5 + if present == 4 { 10 } else { 0 }
    }

    /// Returns `CommonPassword` when the password equals an entry of the common password list
    /// (case insensitive), `CommonPattern` when it contains one, otherwise `None`.
    fn common_match(password: &str) -> Option<Issue> {
        static COMMON: OnceLock<Vec<String>> = OnceLock::new();
        let common = COMMON.get_or_init(|| {
            COMMON_PASSWORDS
                .lines()
                .map(|line| line.trim().to_lowercase())
                .filter(|line| !line.is_empty())
                .collect()
        });

        let lower = password.to_lowercase();
        if common.contains(&lower) {
            Some(Issue::CommonPassword)
        } else if common.iter().any(|entry| lower.contains(entry.as_str())) {
            Some(Issue::CommonPattern)
        } else {
            None
        }
    }

    /// True when the same character appears three times in a row.
    fn has_repetitions(password: &str) -> bool {
        let chars: Vec<char> = password.chars().collect();
        chars.windows(3).any(|w| w[0] == w[1] && w[1] == w[2])
    }

    /// Length times log2 of the pool size: 26 per letter case, 10 digits, 14 special characters,
    /// 32 for any other character.
    fn entropy(length: usize, classes: &CharacterClasses) -> f64 {
        let pool = [
            (classes.lowercase, 26),
            (classes.uppercase, 26),
            (classes.digit, 10),
            (classes.special, SPECIAL.len()),
            (classes.other, 32),
        ]
        .iter()
        .filter(|(present, _)| *present)
        .map(|(_, size)| size)
        .sum::<usize>();
        if pool == 0 {
            return 0.0;
        }
        length as f64 * (pool as f64).log2()
    }

    fn entropy_score(entropy: f64) -> u32 {
        match entropy {
            e if e <= 0.0 => 0,
            e if e < 30.0 => 5,
            e if e < 40.0 => 10,
            e if e < 50.0 => 15,
            _ => 20,
        }
    }

    fn rating(score: u32) -> Rating {
        match score {
            0..=40 => Rating::Weak,
            41..=60 => Rating::Medium,
            61..=80 => Rating::Strong,
            _ => Rating::VeryStrong,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_score() {
        assert_eq!(HealthCheck::length_score(5), 0);
        assert_eq!(HealthCheck::length_score(8), 10);
        assert_eq!(HealthCheck::length_score(13), 20);
        assert_eq!(HealthCheck::length_score(20), 25);
    }

    #[test]
    fn test_length_counts_characters() {
        assert_eq!(HealthCheck::analyze_password("äöüß").length, 4);
    }

    #[test]
    fn test_diversity_score() {
        let all = HealthCheck::character_classes("Abc123!");
        assert_eq!(HealthCheck::diversity_score(&all), 30);
        let lower = HealthCheck::character_classes("abc");
        assert_eq!(HealthCheck::diversity_score(&lower), 5);
    }

    #[test]
    fn test_has_repetitions() {
        assert!(HealthCheck::has_repetitions("aaa"));
        assert!(HealthCheck::has_repetitions("abc111"));
        assert!(!HealthCheck::has_repetitions("abc123"));
        assert!(!HealthCheck::has_repetitions("ab"));
    }

    #[test]
    fn test_common_match() {
        assert_eq!(
            HealthCheck::common_match("Password"),
            Some(Issue::CommonPassword)
        );
        assert_eq!(
            HealthCheck::common_match("MyPassword!7"),
            Some(Issue::CommonPattern)
        );
        assert_eq!(HealthCheck::common_match("Xy9$mK2@nP7#qW"), None);
    }

    #[test]
    fn test_common_password_is_weak() {
        let analysis = HealthCheck::analyze_password("password");
        assert_eq!(analysis.rating, Rating::Weak);
        assert!(analysis.issues.contains(&Issue::CommonPassword));
    }

    #[test]
    fn test_entropy() {
        let classes = HealthCheck::character_classes("Abc123!");
        assert!(HealthCheck::entropy(7, &classes) > 0.0);
        let empty = HealthCheck::character_classes("");
        assert_eq!(HealthCheck::entropy(0, &empty), 0.0);
        assert_eq!(HealthCheck::entropy_score(0.0), 0);
    }

    #[test]
    fn test_rating() {
        assert_eq!(HealthCheck::rating(30), Rating::Weak);
        assert_eq!(HealthCheck::rating(50), Rating::Medium);
        assert_eq!(HealthCheck::rating(70), Rating::Strong);
        assert_eq!(HealthCheck::rating(90), Rating::VeryStrong);
    }

    #[test]
    fn test_analyze_strong_password() {
        let analysis = HealthCheck::analyze_password("Xy9$mK2@nP7#qWv4");
        assert_eq!(analysis.length, 16);
        assert!(analysis.has_lowercase && analysis.has_uppercase);
        assert!(analysis.has_digit && analysis.has_special);
        assert!(analysis.issues.is_empty());
        assert_eq!(analysis.rating, Rating::VeryStrong);
    }
}
