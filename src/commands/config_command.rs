use crate::PasswordMode;
use crate::commands::CommandResult;
use crate::config::{COUNT_RANGE, Config, LENGTH_RANGE};
use crate::kd_cli::choice_prompt::Choice;
use crate::kd_cli::confirm::Confirm;
use crate::kd_cli::input::Input;
use crate::kd_cli::output::Output;
use crate::kd_cli::select::Select;
use crate::kd_cli::terminal::Terminal;
use crate::translation::{LANGUAGES, Translation};

pub struct ConfigCommand;

impl ConfigCommand {
    /// Prints the config values below a section line with the config file path.
    pub fn show(config: &Config) -> CommandResult {
        Output::section(
            &Translation::get("config.title"),
            &Config::path()?.display().to_string(),
        );
        let labels = [
            Translation::get("config.label.mode"),
            Translation::get("config.label.length"),
            Translation::get("config.label.count"),
            Translation::get("config.label.auto_save"),
            Translation::get("config.label.language"),
        ];
        let values = [
            config.general.default_mode.name().to_string(),
            config.general.default_length.to_string(),
            config.general.default_count.to_string(),
            Self::yes_no(config.general.auto_save),
            Self::language_name(&config.language.lang).to_string(),
        ];
        let rows: Vec<(&str, &str)> = labels
            .iter()
            .zip(values.iter())
            .map(|(label, value)| (label.as_str(), value.as_str()))
            .collect();
        Output::key_value(&rows);
        Terminal::write("\n");
        Ok(())
    }

    /// Applies the given values, or asks for every value with the current one preselected when
    /// none is given, then saves the config.
    pub fn edit(
        config: &Config,
        mode: Option<PasswordMode>,
        length: Option<usize>,
        count: Option<usize>,
        auto_save: Option<bool>,
        lang: Option<String>,
    ) -> CommandResult {
        let mut updated = config.clone();
        let no_values = mode.is_none()
            && length.is_none()
            && count.is_none()
            && auto_save.is_none()
            && lang.is_none();

        if no_values && Terminal::interactive() {
            Self::ask(&mut updated)?;
        } else {
            if let Some(mode) = mode {
                updated.general.default_mode = mode;
            }
            if let Some(length) = length {
                updated.general.default_length = length;
            }
            if let Some(count) = count {
                updated.general.default_count = count;
            }
            if let Some(auto_save) = auto_save {
                updated.general.auto_save = auto_save;
            }
            if let Some(lang) = lang {
                updated.language.lang = lang;
            }
        }

        updated.save()?;
        Translation::set_language(&updated.language.lang);
        Output::success(
            &Translation::get("config.saved"),
            &Config::path()?.display().to_string(),
        );
        Ok(())
    }

    fn ask(config: &mut Config) -> Result<(), Box<dyn std::error::Error>> {
        let modes: Vec<Choice> = PasswordMode::ALL
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
            .position(|mode| *mode == config.general.default_mode)
            .unwrap_or(0);
        config.general.default_mode =
            PasswordMode::ALL[Select::new(&Translation::get("config.label.mode"), modes)
                .initial(initial)
                .run()?];

        let (min, max) = LENGTH_RANGE;
        config.general.default_length = Input::new(&Translation::get("config.label.length"))
            .default_value(&config.general.default_length.to_string())
            .number(min as i64, max as i64)? as usize;

        let (min, max) = COUNT_RANGE;
        config.general.default_count = Input::new(&Translation::get("config.label.count"))
            .default_value(&config.general.default_count.to_string())
            .number(min as i64, max as i64)? as usize;

        config.general.auto_save = Confirm::new(
            &Translation::get("config.label.auto_save"),
            config.general.auto_save,
        )
        .ask()?;

        let languages: Vec<Choice> = LANGUAGES.iter().map(|(_, name)| (*name).into()).collect();
        let initial = LANGUAGES
            .iter()
            .position(|(code, _)| *code == config.language.lang)
            .unwrap_or(0);
        let index = Select::new(&Translation::get("config.label.language"), languages)
            .initial(initial)
            .run()?;
        config.language.lang = LANGUAGES[index].0.to_string();
        Ok(())
    }

    fn yes_no(value: bool) -> String {
        Translation::get(if value { "config.yes" } else { "config.no" })
    }

    fn language_name(code: &str) -> &str {
        LANGUAGES
            .iter()
            .find(|(language, _)| *language == code)
            .map_or(code, |(_, name)| name)
    }
}
