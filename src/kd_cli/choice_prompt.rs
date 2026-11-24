use super::prompt::Prompt;
use super::style::Style;
use super::text::Text;
use super::texts::Texts;
use super::theme::Theme;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    pub label: String,
    pub hint: String,
}

pub struct ChoicePrompt {
    pub prompt: Prompt,
    pub choices: Vec<Choice>,
    pub page_size: usize,
    label_width: usize,
}

impl Choice {
    pub fn new(label: &str, hint: &str) -> Self {
        Choice { label: label.into(), hint: hint.into() }
    }
}

impl From<&str> for Choice {
    fn from(label: &str) -> Self {
        Choice::new(label, "")
    }
}

impl ChoicePrompt {
    pub fn new(question: &str, choices: Vec<Choice>) -> Self {
        let label_width = choices.iter().map(|choice| Text::width(&choice.label)).max().unwrap_or(0);
        ChoicePrompt { prompt: Prompt::new(question), choices, page_size: 7, label_width }
    }

    /// Rows for the choices in `visible` (indices into the choices) with the cursor at `cursor`
    /// (index into `visible`): `❯` marker, mark from `mark` (left out when empty), label from `label`
    /// padded to the longest label, hint muted. Hidden rows above and below are counted as
    /// `↑ n weitere` and `↓ n weitere`.
    pub fn rows(
        &self,
        visible: &[usize],
        cursor: usize,
        mark: impl Fn(usize, bool) -> String,
        label: impl Fn(usize, bool) -> String,
    ) -> Vec<String> {
        let count = visible.len();
        let page = self.page_size.max(1).min(count);
        let top = cursor.saturating_sub(page / 2).min(count - page);
        let mut lines = Vec::new();
        if top > 0 {
            lines.push(Self::more("↑", top));
        }
        for (position, &index) in visible.iter().enumerate().skip(top).take(page) {
            let current = position == cursor;
            let choice = &self.choices[index];
            let mut line = if current { Style::paint("❯", Theme::ACCENT, false) + " " } else { "  ".to_string() };
            let marked = mark(index, current);
            if !marked.is_empty() {
                line += &(marked + " ");
            }
            line += &label(index, current);
            line += &" ".repeat(self.label_width.saturating_sub(Text::width(&choice.label)));
            if !choice.hint.is_empty() {
                line += &format!("   {}", Style::paint(&choice.hint, Theme::MUTED, false));
            }
            lines.push(line);
        }
        if top + page < count {
            lines.push(Self::more("↓", count - top - page));
        }
        lines
    }

    fn more(arrow: &str, count: usize) -> String {
        format!("  {}", Style::paint(&format!("{arrow} {count} {}", Texts::get().more), Theme::MUTED, false))
    }
}