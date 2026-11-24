use super::prompt::{Prompt, PromptError};
use super::style::Style;
use super::terminal::{KeyType, RawMode, Terminal};
use super::texts::Texts;

pub struct Confirm {
    prompt: Prompt,
    default: bool,
}

impl Confirm {
    /// Creates a yes/no question. The default is used on Enter and shown as capital letter in
    /// `(j/N)` or `(J/n)`.
    pub fn new(question: &str, default: bool) -> Self {
        Confirm { prompt: Prompt::new(question), default }
    }

    /// Waits for `j`/`y`, `n` or Enter. Esc and Ctrl+C abort.
    pub fn ask(mut self) -> Result<bool, PromptError> {
        self.prompt.begin()?;
        let texts = Texts::get();
        let suffix = if self.default {
            format!("({}/n)", texts.yes_key.to_ascii_uppercase())
        } else {
            format!("({}/N)", texts.yes_key)
        };
        let _raw = RawMode::new();
        loop {
            let line = self.prompt.question_line(&suffix) + " " + &Style::inverse(" ");
            self.prompt.draw(&[line]);
            let key = Terminal::read_key();
            let answer = match key.kind {
                KeyType::Interrupt | KeyType::Escape => return Err(self.prompt.abort()),
                KeyType::Enter => self.default,
                KeyType::Char if key.text.to_lowercase() == texts.yes_key.to_string() => true,
                KeyType::Char if key.text.to_lowercase() == "n" => false,
                _ => continue,
            };
            self.prompt.succeed(if answer { texts.yes } else { texts.no });
            return Ok(answer);
        }
    }
}