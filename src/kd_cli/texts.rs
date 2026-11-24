use std::sync::atomic::{AtomicU8, Ordering};

const UNSET: u8 = 0;
const GERMAN_TEXTS: u8 = 1;
const ENGLISH_TEXTS: u8 = 2;

static LANGUAGE: AtomicU8 = AtomicU8::new(UNSET);

pub struct Texts {
    pub german: bool,
    pub yes_key: char,
    pub yes: &'static str,
    pub no: &'static str,
    pub aborted: &'static str,
    pub not_interactive: &'static str,
    pub required: &'static str,
    pub minimum: &'static str,
    pub maximum: &'static str,
    pub not_a_number: &'static str,
    pub range_to: &'static str,
    pub standard: &'static str,
    pub confirm_password: &'static str,
    pub password_mismatch: &'static str,
    pub more: &'static str,
    pub of: &'static str,
    pub no_matches: &'static str,
    pub selected: &'static str,
    pub none: &'static str,
    pub remaining: &'static str,
    pub total: &'static str,
    pub failed: &'static str,
    pub select_hint: &'static str,
    pub multi_select_hint: &'static str,
    pub filter_hint: &'static str,
    pub number_hint: &'static str,
    pub path_hint: &'static str,
    pub usage: &'static str,
    pub commands: &'static str,
    pub options: &'static str,
}

const GERMAN: Texts = Texts {
    german: true,
    yes_key: 'j',
    yes: "Ja",
    no: "Nein",
    aborted: "Abgebrochen",
    not_interactive: "Keine Eingabe möglich: kein interaktives Terminal",
    required: "Eingabe erforderlich",
    minimum: "Mindestens",
    maximum: "Höchstens",
    not_a_number: "Keine gültige Zahl",
    range_to: "bis",
    standard: "Standard",
    confirm_password: "Passwort bestätigen",
    password_mismatch: "Passwörter stimmen nicht überein",
    more: "weitere",
    of: "von",
    no_matches: "Keine Treffer",
    selected: "gewählt",
    none: "keine",
    remaining: "noch",
    total: "Gesamt",
    failed: "Abbruch",
    select_hint: "↑↓ wählen · Enter bestätigen · Esc abbrechen",
    multi_select_hint: "Leertaste umschalten · a alle · Enter bestätigen",
    filter_hint: "Tippen filtert · ↑↓ wählen · Enter bestätigen",
    number_hint: "↑↓ erhöhen / verringern · Enter bestätigen",
    path_hint: "Tab vervollständigen",
    usage: "BENUTZUNG",
    commands: "BEFEHLE",
    options: "OPTIONEN",
};

const ENGLISH: Texts = Texts {
    german: false,
    yes_key: 'y',
    yes: "Yes",
    no: "No",
    aborted: "Aborted",
    not_interactive: "No input possible: not an interactive terminal",
    required: "Input required",
    minimum: "At least",
    maximum: "At most",
    not_a_number: "Not a valid number",
    range_to: "to",
    standard: "default",
    confirm_password: "Confirm password",
    password_mismatch: "Passwords do not match",
    more: "more",
    of: "of",
    no_matches: "No matches",
    selected: "selected",
    none: "none",
    remaining: "left",
    total: "Total",
    failed: "Failed",
    select_hint: "↑↓ select · Enter confirm · Esc cancel",
    multi_select_hint: "Space toggle · a all · Enter confirm",
    filter_hint: "Type to filter · ↑↓ select · Enter confirm",
    number_hint: "↑↓ increase / decrease · Enter confirm",
    path_hint: "Tab completes",
    usage: "USAGE",
    commands: "COMMANDS",
    options: "OPTIONS",
};

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetUserDefaultUILanguage() -> u16;
}

impl Texts {
    /// Returns the texts chosen with `set_german`, otherwise the German texts when LC_ALL,
    /// LC_MESSAGES or LANG starts with `de` (on Windows: the user interface language is German),
    /// otherwise the English texts.
    pub fn get() -> &'static Texts {
        let mut language = LANGUAGE.load(Ordering::Relaxed);
        if language == UNSET {
            language = if Self::german_locale() { GERMAN_TEXTS } else { ENGLISH_TEXTS };
            LANGUAGE.store(language, Ordering::Relaxed);
        }
        if language == GERMAN_TEXTS { &GERMAN } else { &ENGLISH }
    }

    /// Uses the German or English texts for every following get() instead of the detected locale.
    pub fn set_german(german: bool) {
        LANGUAGE.store(if german { GERMAN_TEXTS } else { ENGLISH_TEXTS }, Ordering::Relaxed);
    }

    fn german_locale() -> bool {
        #[cfg(windows)]
        {
            unsafe { GetUserDefaultUILanguage() & 0x3ff == 0x07 }
        }
        #[cfg(not(windows))]
        {
            ["LC_ALL", "LC_MESSAGES", "LANG"]
                .iter()
                .filter_map(|name| std::env::var(name).ok())
                .find(|value| !value.is_empty())
                .is_some_and(|value| value.starts_with("de"))
        }
    }
}