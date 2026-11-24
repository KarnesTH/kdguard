use std::fmt;

use super::output::Output;
use super::style::{Mark, Style};
use super::terminal::Terminal;
use super::text::Text;
use super::texts::Texts;
use super::theme::Theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromptError {
    Aborted,
    NotInteractive,
}

pub struct Prompt {
    pub question: String,
    drawn: usize,
}

impl fmt::Display for PromptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let texts = Texts::get();
        f.write_str(match self {
            PromptError::Aborted => texts.aborted,
            PromptError::NotInteractive => texts.not_interactive,
        })
    }
}

impl std::error::Error for PromptError {}

impl Prompt {
    pub fn new(question: &str) -> Self {
        Prompt { question: question.into(), drawn: 0 }
    }

    /// Checks for an interactive terminal and hides the cursor. Without a terminal the error is
    /// printed and returned.
    pub fn begin(&self) -> Result<(), PromptError> {
        if !Terminal::interactive() {
            Output::error(Texts::get().not_interactive, &self.question);
            return Err(PromptError::NotInteractive);
        }
        Terminal::hide_cursor();
        Ok(())
    }

    /// Replaces the previously drawn lines with `lines`. Every line is cut to the terminal width.
    pub fn draw(&mut self, lines: &[String]) {
        self.clear();
        let width = Terminal::width().saturating_sub(1);
        let out: Vec<String> = lines.iter().map(|line| Text::truncate(line, width)).collect();
        Terminal::write(&out.join("\n"));
        self.drawn = lines.len();
    }

    /// Removes the drawn lines and prints the success mark, the question and the answer, then shows
    /// the cursor.
    pub fn succeed(&mut self, answer: &str) {
        self.clear();
        Terminal::write(&format!(
            "{} {} {}\n",
            Style::mark(Mark::Success),
            Style::paint(&self.question, Theme::TEXT, true),
            Style::paint(answer, Theme::ACCENT, false)
        ));
        Terminal::show_cursor();
    }

    /// Removes the drawn lines, prints the warning `Abgebrochen`, shows the cursor and returns `Aborted`.
    pub fn abort(&mut self) -> PromptError {
        self.clear();
        Output::warn(Texts::get().aborted, "");
        Terminal::show_cursor();
        PromptError::Aborted
    }

    /// `? question` with the question bold and a muted suffix (left out when empty).
    pub fn question_line(&self, suffix: &str) -> String {
        let line = Style::paint("›", Theme::ACCENT, true) + " " + &Style::paint(&self.question, Theme::TEXT, true);
        if suffix.is_empty() { line } else { line + " " + &Style::paint(suffix, Theme::MUTED, false) }
    }

    /// `❯ ` in accent followed by `value` and a block cursor.
    pub fn input_line(value: &str) -> String {
        Style::paint("❯", Theme::ACCENT, false) + " " + &Style::paint(value, Theme::TEXT, false) + &Style::inverse(" ")
    }

    pub fn hint_line(hint: &str) -> String {
        Style::paint(hint, Theme::MUTED, false)
    }

    pub fn error_line(message: &str) -> String {
        format!("{} {}", Style::mark(Mark::Error), Style::paint(message, Theme::ERROR, false))
    }

    fn clear(&mut self) {
        if self.drawn == 0 {
            return;
        }
        Terminal::write(&("\r\x1b[2K".to_string() + &"\x1b[A\x1b[2K".repeat(self.drawn - 1)));
        self.drawn = 0;
    }
}

impl Drop for Prompt {
    fn drop(&mut self) {
        Terminal::show_cursor();
    }
}