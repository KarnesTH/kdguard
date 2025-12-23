use crate::{PasswordMode, config::Config};

pub enum CurrentScreen {
    Main,
    Generator,
    Settings,
    Help,
    Exit,
}

pub struct App {
    pub current_screen: CurrentScreen,
    pub generator: Generator,
    pub settings: Settings,
    pub help: Help,
    pub exit: Exit,
}

pub struct Generator {
    pub length: usize,
    pub count: usize,
    pub mode: PasswordMode,
    pub pattern: Option<String>,
    pub words: Option<usize>,
    pub seed_env: Option<String>,
    pub service: Option<String>,
    pub salt: Option<String>,
}

pub struct Settings {
    pub language: String,
    pub auto_save: bool,
    pub default_length: usize,
    pub default_count: usize,
}

pub struct Help {
    pub language: String,
    pub auto_save: bool,
    pub default_length: usize,
    pub default_count: usize,
}

pub struct Exit {
    pub exit: bool,
}

impl App {
    pub fn new() -> Self {
        let config = Config::load_config().unwrap();
        Self { 
            current_screen: CurrentScreen::Main,
            generator: Generator {
                length: config.general.default_length,
                count: config.general.default_count,
                mode: PasswordMode::Random,
                pattern: None,
                words: None,
                seed_env: None,
                service: None,
                salt: None,
            },
            settings: Settings {
                language: config.language.lang,
                auto_save: false,
                default_length: config.general.default_length,
                default_count: config.general.default_count,
            },
            help: Help {
                language: "en".to_string(),
                auto_save: config.general.auto_save,
                default_length: config.general.default_length,
                default_count: config.general.default_count,
            },
            exit: Exit {
                exit: false,
            },
         }
    }
}
