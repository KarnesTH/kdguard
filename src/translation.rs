use lingua_i18n_rs::prelude::{Lingua, LinguaError};

use crate::kd_cli::texts::Texts;

const ENGLISH: &str = include_str!("../languages/en.json");
const GERMAN: &str = include_str!("../languages/de.json");

pub const LANGUAGES: [(&str, &str); 2] = [("de", "Deutsch"), ("en", "English")];

pub struct Translation;

impl Translation {
    /// Loads the embedded language files and activates `lang`, falling back to English for an
    /// unknown code. The built-in texts of the terminal components follow the same language.
    pub fn init(lang: &str) -> Result<(), LinguaError> {
        Lingua::load_translations_from_str("en", ENGLISH)?;
        Lingua::load_translations_from_str("de", GERMAN)?;
        Self::set_language(lang);
        Ok(())
    }

    /// Activates `lang` (English when unknown) for kdguard and the terminal components.
    pub fn set_language(lang: &str) {
        let active = if Self::is_supported(lang) { lang } else { "en" };
        let _ = Lingua::set_language(active);
        Texts::set_german(active == "de");
    }

    /// Returns the code of the active language.
    pub fn language() -> String {
        Lingua::get_language().unwrap_or_else(|_| "en".to_string())
    }

    /// Returns true for a language code that has a language file.
    pub fn is_supported(lang: &str) -> bool {
        LANGUAGES.iter().any(|(code, _)| *code == lang)
    }

    /// Returns the language code of the system: `de` for a German locale, otherwise `en`.
    pub fn system_language() -> &'static str {
        if Texts::get().german { "de" } else { "en" }
    }

    /// Returns the text for `key`, or the key itself when it is missing.
    pub fn get(key: &str) -> String {
        Lingua::t(key, &[]).unwrap_or_else(|_| key.to_string())
    }

    /// Returns the text for `key` with every `{{name}}` replaced, or the key itself when it is
    /// missing.
    pub fn with(key: &str, params: &[(&str, &str)]) -> String {
        Lingua::t(key, params).unwrap_or_else(|_| key.to_string())
    }
}
