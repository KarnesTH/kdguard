use super::choice_prompt::{Choice, ChoicePrompt};
use super::prompt::{Prompt, PromptError};
use super::style::Style;
use super::terminal::{KeyType, RawMode, Terminal};
use super::texts::Texts;
use super::theme::Theme;

pub struct Select {
    list: ChoicePrompt,
    initial: usize,
}

impl Select {
    pub fn new(question: &str, choices: Vec<Choice>) -> Self {
        Select { list: ChoicePrompt::new(question, choices), initial: 0 }
    }

    /// Index the cursor starts on.
    pub fn initial(mut self, index: usize) -> Self {
        self.initial = index;
        self
    }

    /// Number of rows shown at once, the list scrolls around the cursor.
    pub fn page_size(mut self, size: usize) -> Self {
        self.list.page_size = size.max(1);
        self
    }

    /// Moves the cursor with Up/Down (Home/End jump to the ends) and returns the chosen index on Enter.
    pub fn run(mut self) -> Result<usize, PromptError> {
        self.list.prompt.begin()?;
        let count = self.list.choices.len();
        if count == 0 {
            return Err(self.list.prompt.abort());
        }
        let all: Vec<usize> = (0..count).collect();
        let mut cursor = self.initial.min(count - 1);
        let _raw = RawMode::new();
        loop {
            let mut lines = vec![self.list.prompt.question_line("")];
            lines.extend(self.list.rows(&all, cursor, |_, _| String::new(), |index, current| {
                Style::paint(&self.list.choices[index].label, if current { Theme::ACCENT } else { Theme::TEXT }, current)
            }));
            lines.extend([String::new(), Prompt::hint_line(Texts::get().select_hint)]);
            self.list.prompt.draw(&lines);
            match Terminal::read_key().kind {
                KeyType::Interrupt | KeyType::Escape => return Err(self.list.prompt.abort()),
                KeyType::Up => cursor = (cursor + count - 1) % count,
                KeyType::Down => cursor = (cursor + 1) % count,
                KeyType::Home => cursor = 0,
                KeyType::End => cursor = count - 1,
                KeyType::Enter => {
                    let label = self.list.choices[cursor].label.clone();
                    self.list.prompt.succeed(&label);
                    return Ok(cursor);
                }
                _ => {}
            }
        }
    }
}