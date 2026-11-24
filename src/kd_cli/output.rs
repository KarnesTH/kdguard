use super::style::{Mark, Style};
use super::terminal::Terminal;
use super::text::{Align, Text};
use super::theme::Theme;

const SECTION_WIDTH: usize = 64;
const LABEL_WIDTH: usize = 10;

pub struct Output;

impl Output {
    /// Prints the logo lines in bold accent, the subtitle muted below (skipped when empty) and an
    /// empty line.
    pub fn banner(logo: &[&str], subtitle: &str) {
        let mut out: Vec<String> = logo.iter().map(|line| Style::paint(line, Theme::ACCENT, true)).collect();
        if !subtitle.is_empty() {
            out.push(Style::paint(subtitle, Theme::MUTED, false));
        }
        Terminal::write(&(out.join("\n") + "\n\n"));
    }

    /// Prints an empty line, `━━ title ━━━…` in bold accent filled to the terminal width (at most
    /// 64 columns) with the detail muted behind it (left out when empty), and another empty line.
    pub fn section(title: &str, detail: &str) {
        let width = Terminal::width().min(SECTION_WIDTH);
        let used = Text::width(title) + 4;
        let rest = if width > used + 3 { width - used } else { 3 };
        let mut line = Style::paint(&format!("━━ {title} {}", "━".repeat(rest)), Theme::ACCENT, true);
        if !detail.is_empty() {
            line += &format!("  {}", Style::paint(detail, Theme::MUTED, false));
        }
        Terminal::write(&format!("\n{line}\n\n"));
    }

    /// Prints the success mark, the text and the detail muted.
    pub fn success(text: &str, detail: &str) {
        Terminal::write(&Self::status(Mark::Success, text, detail));
    }

    /// Prints an empty line and the error mark, the text and the detail muted to stderr.
    pub fn error(text: &str, detail: &str) {
        Terminal::write_error(&format!("\n{}", Self::status(Mark::Error, text, detail)));
    }

    /// Prints the warning mark, the text and the detail muted.
    pub fn warn(text: &str, detail: &str) {
        Terminal::write(&Self::status(Mark::Warning, text, detail));
    }

    /// Prints the info mark, the text and the detail muted.
    pub fn info(text: &str, detail: &str) {
        Terminal::write(&Self::status(Mark::Info, text, detail));
    }

    /// Prints the text in info indented by two and an empty line.
    pub fn value(text: &str) {
        Terminal::write(&format!("  {}\n\n", Style::paint(text, Theme::INFO, false)));
    }

    /// Prints the text and the detail muted, indented by two.
    pub fn note(text: &str, detail: &str) {
        let mut line = format!("  {}", Style::paint(text, Theme::MUTED, false));
        if !detail.is_empty() {
            line += &format!("  {}", Style::paint(detail, Theme::MUTED, false));
        }
        Terminal::write(&(line + "\n"));
    }

    /// Prints the text muted, indented by two.
    pub fn muted(text: &str) {
        Terminal::write(&format!("  {}\n", Style::paint(text, Theme::MUTED, false)));
    }

    /// Prints `• text  (detail)` with an accent bullet and the detail muted, indented by two.
    pub fn item(text: &str, detail: &str) {
        let mut line = format!(
            "  {} {}",
            Style::paint("•", Theme::ACCENT, false),
            Style::paint(text, Theme::TEXT, false)
        );
        if !detail.is_empty() {
            line += &format!("  {}", Style::paint(&format!("({detail})"), Theme::MUTED, false));
        }
        Terminal::write(&(line + "\n"));
    }

    /// Prints a line indented by four with the label muted and padded to the label column.
    pub fn detail(label: &str, value: &str) {
        Terminal::write(&Self::detail_line(label, value, LABEL_WIDTH));
    }

    /// Prints one detail line per row with the labels padded to the longest label plus two, at
    /// least to the label column.
    pub fn key_value(rows: &[(&str, &str)]) {
        let width = rows.iter().map(|(key, _)| Text::width(key) + 2).fold(LABEL_WIDTH, usize::max);
        let out: String = rows.iter().map(|(key, value)| Self::detail_line(key, value, width)).collect();
        Terminal::write(&out);
    }

    fn status(mark: Mark, text: &str, detail: &str) -> String {
        let mut line = format!("{} {}", Style::mark(mark), Style::paint(text, Theme::TEXT, false));
        if !detail.is_empty() {
            line += &format!("  {}", Style::paint(detail, Theme::MUTED, false));
        }
        line + "\n"
    }

    fn detail_line(label: &str, value: &str, width: usize) -> String {
        format!(
            "    {}{}\n",
            Style::paint(&Text::pad(label, width, Align::Left), Theme::MUTED, false),
            Style::paint(value, Theme::TEXT, false)
        )
    }
}