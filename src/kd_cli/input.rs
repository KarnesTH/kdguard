use std::fs;
use std::path::MAIN_SEPARATOR;

use super::prompt::{Prompt, PromptError};
use super::terminal::{KeyType, RawMode, Terminal};
use super::texts::Texts;

const MAX_CANDIDATES: usize = 6;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Text,
    Password,
    Number,
    Path,
}

/// Returns an error message for an invalid value, `None` for a valid one.
pub type Validator = Box<dyn Fn(&str) -> Option<String>>;

pub struct Input {
    prompt: Prompt,
    default: String,
    validator: Option<Validator>,
    pending_error: String,
}

impl Input {
    pub fn new(question: &str) -> Self {
        Input { prompt: Prompt::new(question), default: String::new(), validator: None, pending_error: String::new() }
    }

    /// Value used when Enter is pressed on an empty line, shown as `(value)` after the question.
    pub fn default_value(mut self, value: &str) -> Self {
        self.default = value.into();
        self
    }

    /// Checks the value on Enter. An error message keeps the prompt open and is shown below it.
    pub fn validator(mut self, validator: impl Fn(&str) -> Option<String> + 'static) -> Self {
        self.validator = Some(Box::new(validator));
        self
    }

    /// Reads one line of text. Empty input without default fails with "Eingabe erforderlich".
    pub fn text(mut self) -> Result<String, PromptError> {
        self.read(Mode::Text, 0, 0, false)
    }

    /// Reads a line shown as `•`. With `confirm` the password is asked a second time until both match.
    pub fn password(mut self, confirm: bool) -> Result<String, PromptError> {
        let question = self.prompt.question.clone();
        loop {
            self.prompt.question = question.clone();
            let first = self.read(Mode::Password, 0, 0, false)?;
            if !confirm {
                return Ok(first);
            }
            self.prompt.question = Texts::get().confirm_password.into();
            let second = self.read(Mode::Password, 0, 0, false)?;
            if first == second {
                return Ok(first);
            }
            self.pending_error = Texts::get().password_mismatch.into();
        }
    }

    /// Reads a whole number in `min` to `max`. Up and Down change the value by one.
    pub fn number(mut self, min: i64, max: i64) -> Result<i64, PromptError> {
        let value = self.read(Mode::Number, min, max, false)?;
        Ok(value.parse().unwrap_or(min))
    }

    /// Reads a path. Tab completes file and directory names (only directories with
    /// `directories_only`), `~` stands for the home directory.
    pub fn path(mut self, directories_only: bool) -> Result<String, PromptError> {
        let value = self.read(Mode::Path, 0, 0, directories_only)?;
        Ok(Self::expand(&value))
    }

    fn read(&mut self, mode: Mode, min: i64, max: i64, directories_only: bool) -> Result<String, PromptError> {
        self.prompt.begin()?;
        let texts = Texts::get();
        let suffix = match mode {
            Mode::Number if self.default.is_empty() => format!("({min} {} {max})", texts.range_to),
            Mode::Number => format!("({min} {} {max}, {} {})", texts.range_to, texts.standard, self.default),
            Mode::Password => String::new(),
            _ if self.default.is_empty() => String::new(),
            _ => format!("({})", self.default),
        };
        let mut buffer = String::new();
        let mut error = std::mem::take(&mut self.pending_error);
        let mut candidates = String::new();
        let _raw = RawMode::new();
        loop {
            let shown = if mode == Mode::Password { "•".repeat(buffer.chars().count()) } else { buffer.clone() };
            let mut lines = vec![self.prompt.question_line(&suffix), Prompt::input_line(&shown)];
            if !error.is_empty() {
                lines.push(Prompt::error_line(&error));
            }
            if !candidates.is_empty() {
                lines.push(Prompt::hint_line(&format!("  {candidates}")));
            }
            match mode {
                Mode::Number => lines.extend([String::new(), Prompt::hint_line(texts.number_hint)]),
                Mode::Path => lines.extend([String::new(), Prompt::hint_line(texts.path_hint)]),
                _ => {}
            }
            self.prompt.draw(&lines);

            let key = Terminal::read_key();
            match key.kind {
                KeyType::Interrupt | KeyType::Escape => return Err(self.prompt.abort()),
                KeyType::Backspace => {
                    buffer.pop();
                    error.clear();
                }
                KeyType::Up | KeyType::Down if mode == Mode::Number => {
                    let source = if buffer.is_empty() { &self.default } else { &buffer };
                    let current = source.parse::<i64>().unwrap_or(min);
                    let step = if key.kind == KeyType::Up { 1 } else { -1 };
                    buffer = (current + step).clamp(min, max).to_string();
                    error.clear();
                }
                KeyType::Tab if mode == Mode::Path => candidates = Self::complete(&mut buffer, directories_only),
                KeyType::Space | KeyType::Char => {
                    let digit = key.text.chars().all(|c| c.is_ascii_digit());
                    let sign = key.text == "-" && buffer.is_empty() && min < 0;
                    if mode == Mode::Number && !digit && !sign {
                        continue;
                    }
                    buffer += &key.text;
                    error.clear();
                    candidates.clear();
                }
                KeyType::Enter => {
                    let value = if buffer.is_empty() { self.default.clone() } else { buffer.clone() };
                    if let Some(problem) = self.check(mode, &value, min, max) {
                        error = problem;
                        continue;
                    }
                    let answer = if mode == Mode::Password { "•".repeat(8) } else { value.clone() };
                    self.prompt.succeed(&answer);
                    return Ok(value);
                }
                _ => {}
            }
        }
    }

    fn check(&self, mode: Mode, value: &str, min: i64, max: i64) -> Option<String> {
        let texts = Texts::get();
        if value.is_empty() {
            return Some(texts.required.into());
        }
        if mode == Mode::Number {
            let Ok(number) = value.parse::<i64>() else {
                return Some(texts.not_a_number.into());
            };
            if number < min {
                return Some(format!("{} {min}", texts.minimum));
            }
            if number > max {
                return Some(format!("{} {max}", texts.maximum));
            }
        }
        self.validator.as_ref().and_then(|validator| validator(value))
    }

    fn expand(value: &str) -> String {
        let home = std::env::var(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).unwrap_or_default();
        match value.strip_prefix('~') {
            Some(rest) => home + rest,
            None => value.to_string(),
        }
    }

    fn complete(buffer: &mut String, directories_only: bool) -> String {
        let expanded = Self::expand(buffer);
        let cut = expanded.rfind(['/', MAIN_SEPARATOR]).map_or(0, |index| index + 1);
        let (directory, prefix) = expanded.split_at(cut);
        let Ok(entries) = fs::read_dir(if directory.is_empty() { "." } else { directory }) else {
            return String::new();
        };
        let mut matches: Vec<String> = entries
            .flatten()
            .filter_map(|entry| {
                let name = entry.file_name().to_string_lossy().into_owned();
                let is_directory = entry.file_type().is_ok_and(|kind| kind.is_dir());
                let hidden = prefix.is_empty() && name.starts_with('.');
                (name.starts_with(prefix) && (is_directory || !directories_only) && !hidden)
                    .then(|| if is_directory { name + "/" } else { name })
            })
            .collect();
        if matches.is_empty() {
            return String::new();
        }
        matches.sort();
        let mut common = matches[0].clone();
        for candidate in &matches {
            while !candidate.starts_with(&common) {
                common.pop();
            }
        }
        buffer.truncate(buffer.len() - prefix.len());
        buffer.push_str(&common);
        if matches.len() == 1 {
            return String::new();
        }
        let mut shown = matches.iter().take(MAX_CANDIDATES).cloned().collect::<Vec<_>>().join("   ");
        if matches.len() > MAX_CANDIDATES {
            shown += &format!("   +{}", matches.len() - MAX_CANDIDATES);
        }
        shown
    }
}