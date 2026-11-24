use std::process::ExitCode;

use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{Command, CommandFactory};

use crate::Cli;
use crate::kd_cli::help_formatter::HelpFormatter;
use crate::kd_cli::output::Output;
use crate::kd_cli::style::Style;
use crate::kd_cli::terminal::Terminal;
use crate::kd_cli::theme::Theme;
use crate::translation::Translation;

pub struct HelpCommand;

impl HelpCommand {
    /// True when the arguments contain `-h` or `--help` or start with `help`.
    pub fn requested(args: &[String]) -> bool {
        args.iter()
            .skip(1)
            .any(|arg| arg == "-h" || arg == "--help")
            || args.get(1).is_some_and(|arg| arg == "help")
    }

    /// Prints the help of the deepest subcommand named in the arguments: name and version,
    /// description, usage, commands and options with their possible values.
    pub fn print(args: &[String]) {
        let mut root = Cli::command();
        root.build();

        let mut path = vec!["kdg".to_string()];
        let mut command = &root;
        for arg in args
            .iter()
            .skip(1)
            .filter(|arg| !arg.starts_with('-') && *arg != "help")
        {
            if let Some(sub) = command.find_subcommand(arg) {
                path.push(arg.clone());
                command = sub;
            }
        }

        let about = command
            .get_about()
            .map(|about| about.to_string())
            .unwrap_or_default();
        let mut help = HelpFormatter::new(&path.join(" "), env!("CARGO_PKG_VERSION"), &about);
        for usage in Self::usages(command) {
            help = help.usage(&usage);
        }
        for sub in Self::subcommands(command) {
            let about = sub
                .get_about()
                .map(|about| about.to_string())
                .unwrap_or_default();
            help = help.command(sub.get_name(), &about);
        }
        for arg in command.get_arguments().filter(|arg| {
            !arg.is_positional() && !matches!(arg.get_id().as_str(), "help" | "version")
        }) {
            let long = arg
                .get_long()
                .map(|long| format!("--{long}"))
                .unwrap_or_default();
            let flags = match arg.get_short() {
                Some(short) => format!("-{short}, {long}"),
                None => format!("    {long}"),
            };
            let value_name = if arg.get_action().takes_values() {
                Self::value_name(arg.get_id().as_str())
            } else {
                String::new()
            };
            let mut description = arg
                .get_help()
                .map(|text| text.to_string())
                .unwrap_or_default();
            let values: Vec<String> = arg
                .get_possible_values()
                .iter()
                .map(|value| value.get_name().to_string())
                .collect();
            if arg.get_action().takes_values() && !values.is_empty() {
                description += &format!(" ({})", values.join(", "));
            }
            help = help.option(&flags, &description, &value_name);
        }
        help = help.option("-h, --help", &Translation::get("help.help"), "");
        if path.len() == 1 {
            help = help.option("-V, --version", &Translation::get("help.version"), "");
        }
        help.print();
    }

    /// Prints the version for `-V`, otherwise the error in the active language (the first line
    /// of the clap message for kinds without own text) and a hint to the help. Returns the exit
    /// code.
    pub fn argument_error(error: &clap::Error) -> ExitCode {
        if error.kind() == ErrorKind::DisplayVersion {
            Terminal::write(&format!(
                "{} {}\n",
                Style::paint("kdguard", Theme::TEXT, true),
                Style::paint(env!("CARGO_PKG_VERSION"), Theme::MUTED, false)
            ));
            return ExitCode::SUCCESS;
        }
        let context = |kind: ContextKind| {
            error
                .get(kind)
                .map(|value| match value {
                    ContextValue::String(text) => text.clone(),
                    ContextValue::Strings(texts) => texts.join(", "),
                    other => other.to_string(),
                })
                .unwrap_or_default()
        };
        let arg = context(ContextKind::InvalidArg)
            .split(' ')
            .next()
            .unwrap_or_default()
            .to_string();
        let value = context(ContextKind::InvalidValue);
        let detail = match error.kind() {
            ErrorKind::UnknownArgument => {
                Translation::with("errors.argument.unknown", &[("arg", &arg)])
            }
            ErrorKind::InvalidSubcommand => Translation::with(
                "errors.argument.unknown_command",
                &[("command", &context(ContextKind::InvalidSubcommand))],
            ),
            ErrorKind::InvalidValue | ErrorKind::ValueValidation => Translation::with(
                "errors.argument.invalid_value",
                &[("value", &value), ("arg", &arg)],
            ),
            ErrorKind::ArgumentConflict => Translation::with(
                "errors.argument.conflict",
                &[("arg", &arg), ("other", &context(ContextKind::PriorArg))],
            ),
            ErrorKind::MissingSubcommand | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
                Translation::get("errors.argument.missing_command")
            }
            _ => {
                let message = error.to_string();
                let first = message.lines().next().unwrap_or_default();
                first.trim_start_matches("error: ").to_string()
            }
        };
        Output::error(&Translation::get("errors.arguments"), &detail);
        Output::note(&Translation::get("help.hint"), "");
        ExitCode::FAILURE
    }

    /// Usage lines: `[options]` with the positional arguments, plus `<command>` when the command
    /// has subcommands.
    fn usages(command: &Command) -> Vec<String> {
        let mut line: Vec<String> = command
            .get_positionals()
            .map(|arg| format!("[{}]", Self::value_name(arg.get_id().as_str())))
            .collect();
        line.push(format!("[{}]", Translation::get("help.options")));
        let mut usages = Vec::new();
        let has_options = command.get_arguments().any(|arg| {
            !arg.is_positional() && !matches!(arg.get_id().as_str(), "help" | "version")
        });
        if has_options || command.get_positionals().next().is_some() {
            usages.push(line.join(" "));
        }
        if Self::subcommands(command).next().is_some() {
            usages.push(format!("<{}>", Translation::get("help.command")));
        }
        usages
    }

    /// Returns the translated value name for an argument id (`help.values.<id>`), otherwise the
    /// id with `-` instead of `_`.
    fn value_name(id: &str) -> String {
        let key = format!("help.values.{id}");
        let name = Translation::get(&key);
        if name == key {
            id.replace('_', "-")
        } else {
            name
        }
    }

    fn subcommands(command: &Command) -> impl Iterator<Item = &Command> {
        command
            .get_subcommands()
            .filter(|sub| sub.get_name() != "help" && !sub.is_hide_set())
    }
}
