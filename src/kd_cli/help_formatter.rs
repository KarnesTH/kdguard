use super::style::Style;
use super::terminal::Terminal;
use super::text::{Align, Text};
use super::texts::Texts;
use super::theme::Theme;

pub struct HelpFormatter {
    name: String,
    version: String,
    description: String,
    usages: Vec<String>,
    commands: Vec<(String, String)>,
    options: Vec<(String, String)>,
}

impl HelpFormatter {
    /// Creates the help with program name, version and a one-line description.
    pub fn new(name: &str, version: &str, description: &str) -> Self {
        HelpFormatter {
            name: name.into(),
            version: version.into(),
            description: description.into(),
            usages: Vec::new(),
            commands: Vec::new(),
            options: Vec::new(),
        }
    }

    /// Adds a usage line, e.g. `<command> [options]`, printed after the program name.
    pub fn usage(mut self, line: &str) -> Self {
        self.usages.push(line.into());
        self
    }

    /// Adds a command with its description.
    pub fn command(mut self, name: &str, description: &str) -> Self {
        self.commands.push((name.into(), description.into()));
        self
    }

    /// Adds an option, e.g. `-n, --dry-run`, with a value name shown as `<value>` (left out when empty).
    pub fn option(mut self, flags: &str, description: &str, value_name: &str) -> Self {
        let mut flags = flags.to_string();
        if !value_name.is_empty() {
            flags += &format!(" {}", Style::paint(&format!("<{value_name}>"), Theme::MUTED, false));
        }
        self.options.push((flags, description.into()));
        self
    }

    /// Builds the help: name and version, description, then the USAGE, COMMANDS and OPTIONS sections
    /// with the descriptions aligned in one column per section.
    pub fn render(&self) -> String {
        let texts = Texts::get();
        let mut out = format!(
            "{} {}\n",
            Style::paint(&self.name, Theme::TEXT, true),
            Style::paint(&self.version, Theme::MUTED, false)
        );
        if !self.description.is_empty() {
            out += &(Style::paint(&self.description, Theme::TEXT, false) + "\n");
        }
        if !self.usages.is_empty() {
            out += &format!("\n{}\n", Style::paint(texts.usage, Theme::ACCENT, true));
            for line in &self.usages {
                out += &format!("  {} {}\n", Style::paint(&self.name, Theme::TEXT, true), Style::paint(line, Theme::MUTED, false));
            }
        }
        out += &Self::section(texts.commands, &self.commands);
        out += &Self::section(texts.options, &self.options);
        out
    }

    /// Writes `render()` to stdout.
    pub fn print(&self) {
        Terminal::write(&self.render());
    }

    fn section(title: &str, entries: &[(String, String)]) -> String {
        if entries.is_empty() {
            return String::new();
        }
        let width = entries.iter().map(|(name, _)| Text::width(name)).max().unwrap_or(0);
        let mut out = format!("\n{}\n", Style::paint(title, Theme::ACCENT, true));
        for (name, description) in entries {
            out += &format!(
                "  {}   {}\n",
                Style::paint(&Text::pad(name, width, Align::Left), Theme::TEXT, true),
                Style::paint(description, Theme::MUTED, false)
            );
        }
        out
    }
}