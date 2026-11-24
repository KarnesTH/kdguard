use std::io::{self, BufRead};

use crate::commands::CommandResult;
use crate::errors::CommandError;
use crate::kd_cli::input::Input;
use crate::kd_cli::output::Output;
use crate::kd_cli::style::Style;
use crate::kd_cli::terminal::Terminal;
use crate::kd_cli::text::Text;
use crate::kd_cli::theme::Theme;
use crate::password::{HealthCheck, PasswordAnalysis};
use crate::translation::Translation;

pub struct CheckCommand;

impl CheckCommand {
    /// Takes the password from the argument, from stdin for `-` or without a terminal, otherwise
    /// asks for it hidden, then prints the analysis.
    pub fn run(password: Option<String>, detailed: bool) -> CommandResult {
        let password = match password {
            Some(password) if password == "-" => Self::read_stdin()?,
            Some(password) => {
                Output::warn(
                    &Translation::get("check.history_warning"),
                    &Translation::get("check.history_hint"),
                );
                password
            }
            None if Terminal::interactive() => {
                Input::new(&Translation::get("check.ask.password")).password(false)?
            }
            None => Self::read_stdin()?,
        };
        if password.is_empty() {
            return Err(CommandError::EmptyPassword.into());
        }

        Self::print(&HealthCheck::analyze_password(&password), detailed);
        Ok(())
    }

    /// Reads the first line of stdin without the line ending.
    fn read_stdin() -> Result<String, CommandError> {
        let mut line = String::new();
        io::stdin()
            .lock()
            .read_line(&mut line)
            .map_err(|e| CommandError::ReadStdin(e.to_string()))?;
        Ok(line.trim_end_matches(['\r', '\n']).to_string())
    }

    fn print(analysis: &PasswordAnalysis, detailed: bool) {
        let rating = Translation::get(&format!("check.rating.{}", analysis.rating.key()));
        Output::section(
            &Translation::get("check.title"),
            &format!("{}/100 · {rating}", analysis.score.total),
        );

        let length = Translation::with(
            "check.value.length",
            &[("length", &analysis.length.to_string())],
        );
        let entropy = Translation::with(
            "check.value.entropy",
            &[("entropy", &Text::decimal(analysis.entropy))],
        );
        let classes = [
            ("a-z", analysis.has_lowercase),
            ("A-Z", analysis.has_uppercase),
            ("0-9", analysis.has_digit),
            ("!@#", analysis.has_special),
        ]
        .iter()
        .map(|(label, present)| {
            let mark = if *present {
                Style::paint("✓", Theme::SUCCESS, false)
            } else {
                Style::paint("✗", Theme::ERROR, false)
            };
            format!("{label} {mark}")
        })
        .collect::<Vec<_>>()
        .join("  ");

        let length_label = Translation::get("check.label.length");
        let entropy_label = Translation::get("check.label.entropy");
        let classes_label = Translation::get("check.label.classes");
        let rows: Vec<(&str, String)> = vec![
            (&length_label, length),
            (&entropy_label, entropy),
            (&classes_label, classes),
        ];

        let rows: Vec<(&str, &str)> = rows.iter().map(|(k, v)| (*k, v.as_str())).collect();
        Output::key_value(&rows);

        if detailed {
            Output::section(&Translation::get("check.score_title"), "");
            let labels = [
                Translation::get("check.label.length"),
                Translation::get("check.label.diversity"),
                Translation::get("check.label.complexity"),
                Translation::get("check.label.entropy"),
            ];
            let scores = [
                format!("{}/25", analysis.score.length_score),
                format!("{}/30", analysis.score.diversity_score),
                format!("{}/25", analysis.score.complexity_score),
                format!("{}/20", analysis.score.entropy_score),
            ];
            let rows: Vec<(&str, &str)> = labels
                .iter()
                .zip(scores.iter())
                .map(|(label, score)| (label.as_str(), score.as_str()))
                .collect();
            Output::key_value(&rows);
        }
        Terminal::write("\n");

        if analysis.issues.is_empty() {
            Output::success(&Translation::get("check.no_issues"), "");
            return;
        }
        for issue in &analysis.issues {
            Output::warn(
                &Translation::get(&format!("check.issues.{}.warning", issue.key())),
                &Translation::get(&format!("check.issues.{}.suggestion", issue.key())),
            );
        }
    }
}
