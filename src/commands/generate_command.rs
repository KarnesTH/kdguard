use std::path::{Path, PathBuf};

use crate::commands::CommandResult;
use crate::config::{COUNT_RANGE, Config, DEFAULT_WORDS, LENGTH_RANGE, WORDS_RANGE};
use crate::errors::{CommandError, GeneratorError};
use crate::kd_cli::choice_prompt::Choice;
use crate::kd_cli::confirm::Confirm;
use crate::kd_cli::input::Input;
use crate::kd_cli::logo::Logo;
use crate::kd_cli::output::Output;
use crate::kd_cli::select::Select;
use crate::kd_cli::style::Style;
use crate::kd_cli::terminal::Terminal;
use crate::kd_cli::theme::Theme;
use crate::password::{Generator, HealthCheck, Rating};
use crate::translation::Translation;
use crate::{Cli, PasswordMode};

const DEFAULT_FILE: &str = "kdguard.txt";

struct Request {
    mode: PasswordMode,
    length: usize,
    count: usize,
    pattern: String,
    words: usize,
    seed: String,
    salt: Option<String>,
    service: Option<String>,
}

pub struct GenerateCommand<'a> {
    config: &'a Config,
}

impl<'a> GenerateCommand<'a> {
    pub fn new(config: &'a Config) -> Self {
        GenerateCommand { config }
    }

    /// Asks for the mode and its values with the config values preselected, prints the
    /// passwords and saves them when auto save is on or the save question is confirmed.
    pub fn run_interactive(&self) -> CommandResult {
        Output::banner(
            &Logo::LINES,
            &Translation::with(
                "generate.subtitle",
                &[("version", env!("CARGO_PKG_VERSION"))],
            ),
        );

        let choices: Vec<Choice> = PasswordMode::ALL
            .iter()
            .map(|mode| {
                Choice::new(
                    mode.name(),
                    &Translation::get(&format!("generate.modes.{}", mode.name())),
                )
            })
            .collect();
        let initial = PasswordMode::ALL
            .iter()
            .position(|mode| *mode == self.config.general.default_mode)
            .unwrap_or(0);
        let mode = PasswordMode::ALL[Select::new(&Translation::get("generate.ask.mode"), choices)
            .initial(initial)
            .run()?];

        let mut request = self.request(mode);
        match mode {
            PasswordMode::Random => {
                let (min, max) = LENGTH_RANGE;
                request.length = Input::new(&Translation::get("generate.ask.length"))
                    .default_value(&request.length.to_string())
                    .number(min as i64, max as i64)? as usize;
            }
            PasswordMode::Pattern => request.pattern = Self::ask_pattern()?,
            PasswordMode::Phrase => {
                let (min, max) = WORDS_RANGE;
                request.words = Input::new(&Translation::get("generate.ask.words"))
                    .default_value(&request.words.to_string())
                    .number(min as i64, max as i64)? as usize;
            }
            PasswordMode::Deterministic => {
                request.seed = Self::ask_seed()?;
                request.service =
                    Some(Input::new(&Translation::get("generate.ask.service")).text()?);
            }
        }
        if mode != PasswordMode::Deterministic {
            let (min, max) = COUNT_RANGE;
            request.count = Input::new(&Translation::get("generate.ask.count"))
                .default_value(&request.count.to_string())
                .number(min as i64, max as i64)? as usize;
        }

        let passwords = Self::generate(&request)?;
        Self::print(&request, &passwords);

        let save = self.config.general.auto_save
            || Confirm::new(&Translation::get("generate.ask.save"), false).ask()?;
        if save {
            let path = if self.config.general.auto_save {
                PathBuf::from(DEFAULT_FILE)
            } else {
                PathBuf::from(
                    Input::new(&Translation::get("generate.ask.file"))
                        .default_value(DEFAULT_FILE)
                        .path(false)?,
                )
            };
            Self::save(&passwords, &path)?;
        }
        Ok(())
    }

    /// Generates with the given flags, taking every missing value from the config. Without
    /// `--mode` the mode follows from `--pattern`, `--words` or the deterministic flags. Auto save
    /// from the config only applies when stdout is a terminal.
    pub fn run(&self, cli: &Cli) -> CommandResult {
        let mode = cli.mode.unwrap_or(if cli.pattern.is_some() {
            PasswordMode::Pattern
        } else if cli.words.is_some() {
            PasswordMode::Phrase
        } else if cli.seed_env.is_some() || cli.service.is_some() || cli.salt.is_some() {
            PasswordMode::Deterministic
        } else {
            self.config.general.default_mode
        });

        let mut request = self.request(mode);
        if let Some(length) = cli.length {
            request.length = length;
        }
        if let Some(count) = cli.count {
            request.count = count;
        }
        if let Some(words) = cli.words {
            request.words = words;
        }
        request.salt = cli.salt.clone();
        request.service = cli.service.clone();

        match mode {
            PasswordMode::Pattern => {
                request.pattern = match &cli.pattern {
                    Some(pattern) => pattern.clone(),
                    None if Terminal::interactive() => Self::ask_pattern()?,
                    None => return Err(GeneratorError::EmptyPattern.into()),
                }
            }
            PasswordMode::Deterministic => {
                request.seed = match &cli.seed_env {
                    Some(name) => {
                        let seed = std::env::var(name).unwrap_or_default();
                        if seed.is_empty() {
                            return Err(CommandError::SeedEnvEmpty(name.clone()).into());
                        }
                        Self::warn_weak_seed(&seed);
                        seed
                    }
                    None if Terminal::interactive() => Self::ask_seed()?,
                    None => return Err(CommandError::SeedMissing.into()),
                };
                request.count = 1;
            }
            _ => {}
        }

        let passwords = Self::generate(&request)?;
        Self::print(&request, &passwords);

        let auto_save = self.config.general.auto_save && Terminal::is_terminal();
        let save = cli.output.is_some() || (cli.save || auto_save) && !cli.no_save;
        if save {
            let path = cli
                .output
                .clone()
                .unwrap_or_else(|| PathBuf::from(DEFAULT_FILE));
            Self::save(&passwords, &path)?;
        }
        Ok(())
    }

    fn request(&self, mode: PasswordMode) -> Request {
        Request {
            mode,
            length: self.config.general.default_length,
            count: self.config.general.default_count,
            pattern: String::new(),
            words: DEFAULT_WORDS,
            seed: String::new(),
            salt: None,
            service: None,
        }
    }

    fn ask_pattern() -> Result<String, Box<dyn std::error::Error>> {
        Ok(Input::new(&Translation::get("generate.ask.pattern"))
            .validator(|value| Generator::check_pattern(value).err().map(|e| e.to_string()))
            .text()?)
    }

    /// Asks for the master seed twice (hidden) and warns when it is weak.
    fn ask_seed() -> Result<String, Box<dyn std::error::Error>> {
        let seed = Input::new(&Translation::get("generate.ask.seed")).password(true)?;
        Self::warn_weak_seed(&seed);
        Ok(seed)
    }

    fn warn_weak_seed(seed: &str) {
        let rating = HealthCheck::analyze_password(seed).rating;
        if matches!(rating, Rating::Weak | Rating::Medium) {
            Output::warn(
                &Translation::get("generate.seed_weak"),
                &Translation::get("generate.seed_weak_hint"),
            );
        }
    }

    fn generate(request: &Request) -> Result<Vec<String>, GeneratorError> {
        let (min, max) = COUNT_RANGE;
        if !(min..=max).contains(&request.count) {
            return Err(GeneratorError::InvalidCount(request.count));
        }
        (0..request.count)
            .map(|_| match request.mode {
                PasswordMode::Random => Generator::generate_random_password(request.length),
                PasswordMode::Pattern => Generator::generate_pattern_password(&request.pattern),
                PasswordMode::Phrase => {
                    Generator::generate_phrase_password(request.words, &Translation::language())
                }
                PasswordMode::Deterministic => Generator::generate_deterministic_password(
                    &request.seed,
                    request.salt.as_deref(),
                    request.service.as_deref(),
                ),
            })
            .collect()
    }

    /// Prints a section line with mode and detail and the passwords in the value color. Without a
    /// terminal only the passwords are printed, one per line.
    fn print(request: &Request, passwords: &[String]) {
        if !Terminal::is_terminal() {
            Terminal::write(&(passwords.join("\n") + "\n"));
            return;
        }

        let detail = match request.mode {
            PasswordMode::Random => Translation::with(
                "generate.detail.length",
                &[("length", &request.length.to_string())],
            ),
            PasswordMode::Pattern => request.pattern.clone(),
            PasswordMode::Phrase => Translation::with(
                "generate.detail.words",
                &[("words", &request.words.to_string())],
            ),
            PasswordMode::Deterministic => request
                .service
                .clone()
                .unwrap_or_else(|| Translation::get("generate.detail.no_service")),
        };
        let title = if passwords.len() == 1 {
            Translation::get("generate.title_one")
        } else {
            Translation::get("generate.title_many")
        };
        Output::section(&title, &format!("{} · {detail}", request.mode.name()));

        let lines: String = passwords
            .iter()
            .map(|password| format!("  {}\n", Style::paint(password, Theme::INFO, false)))
            .collect();
        Terminal::write(&(lines + "\n"));
        if request.mode != PasswordMode::Deterministic {
            Output::success(
                &Translation::with(
                    if passwords.len() == 1 {
                        "generate.generated_one"
                    } else {
                        "generate.generated_many"
                    },
                    &[("count", &passwords.len().to_string())],
                ),
                "",
            );
        }
    }

    /// Saves to a new file. An existing file is replaced after confirmation; without a terminal
    /// the existing file is an error.
    fn save(passwords: &[String], path: &Path) -> CommandResult {
        match Generator::save_to_file(passwords, path, false) {
            Ok(()) => {}
            Err(GeneratorError::FileExists(_)) if Terminal::interactive() => {
                let question = Translation::with(
                    "generate.ask.overwrite",
                    &[("path", &path.display().to_string())],
                );
                if !Confirm::new(&question, false).ask()? {
                    Output::warn(&Translation::get("generate.not_saved"), "");
                    return Ok(());
                }
                Generator::save_to_file(passwords, path, true)?;
            }
            Err(error) => return Err(error.into()),
        }
        if Terminal::is_terminal() {
            Output::success(
                &Translation::get("generate.saved"),
                &path.display().to_string(),
            );
        }
        Ok(())
    }
}
