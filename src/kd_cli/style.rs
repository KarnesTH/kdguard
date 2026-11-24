use super::terminal::Terminal;
use super::theme::{Rgb, Theme};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    Success,
    Error,
    Warning,
    Info,
}

pub struct Style;

impl Style {
    /// Wraps text in a truecolor foreground (and bold) sequence plus reset. Without colors the text
    /// is returned unchanged.
    pub fn paint(text: &str, color: Rgb, bold: bool) -> String {
        if !Terminal::colors() {
            return text.to_string();
        }
        let weight = if bold { "\x1b[1m" } else { "" };
        format!("{weight}\x1b[38;2;{};{};{}m{text}\x1b[0m", color.0, color.1, color.2)
    }

    /// Wraps text in bold plus reset, unchanged without colors.
    pub fn bold(text: &str) -> String {
        if Terminal::colors() { format!("\x1b[1m{text}\x1b[0m") } else { text.to_string() }
    }

    /// Wraps text in reverse video plus reset, unchanged without colors.
    pub fn inverse(text: &str) -> String {
        if Terminal::colors() { format!("\x1b[7m{text}\x1b[0m") } else { text.to_string() }
    }

    /// Returns a `▌` bar in the color of the mark (success accent, error red, warning yellow, info
    /// blue). Without colors the mark is written as `✓`, `✗`, `!` or `i`.
    pub fn mark(mark: Mark) -> String {
        if !Terminal::colors() {
            return match mark {
                Mark::Success => "✓",
                Mark::Error => "✗",
                Mark::Warning => "!",
                Mark::Info => "i",
            }
                .to_string();
        }
        let color = match mark {
            Mark::Success => Theme::ACCENT,
            Mark::Error => Theme::ERROR,
            Mark::Warning => Theme::WARNING,
            Mark::Info => Theme::INFO,
        };
        Self::paint("▌", color, false)
    }
}