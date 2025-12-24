use crate::{
    PasswordMode,
    config::Config,
    password::{Generator, HealthCheck, PasswordAnalysis},
};
use ratatui::crossterm::event::KeyCode;

pub enum CurrentScreen {
    Main,
    Generator,
    Settings,
    Help,
    Check,
    Exit,
}

pub struct App {
    pub current_screen: CurrentScreen,
    pub generator: GeneratorState,
    pub settings: Settings,
    pub help: Help,
    pub exit: Exit,
    pub generated_passwords: Vec<String>,
    pub password_input: String,
    pub selected_index: usize,
    pub show_detailed_check: bool,
    pub check_result: Option<PasswordAnalysis>,
    pub input_mode: InputMode,
    pub input_field: InputField,
    pub error_message: Option<String>,
}

pub struct GeneratorState {
    pub length: usize,
    pub count: usize,
    pub mode: PasswordMode,
    pub pattern: String,
    pub words: Option<usize>,
    pub seed_env: String,
    pub service: String,
    pub salt: String,
    pub selected_mode_index: usize,
    pub editing_field: Option<GeneratorField>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum GeneratorField {
    Length,
    Count,
    Pattern,
    Words,
    SeedEnv,
    Service,
    Salt,
}

#[derive(Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Clone, Copy, PartialEq)]
pub enum InputField {
    None,
    PasswordCheck,
    Generator(GeneratorField),
    Settings(SettingsField),
}

#[derive(Clone, Copy, PartialEq)]
pub enum SettingsField {
    Language,
    DefaultLength,
    DefaultCount,
    AutoSave,
}

pub struct Settings {
    pub language: String,
    pub auto_save: bool,
    pub default_length: usize,
    pub default_count: usize,
    pub selected_index: usize,
}

pub struct Help {
    pub scroll: usize,
}

pub struct Exit {
    pub exit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let config = Config::load_config().unwrap();
        let default_mode = match config.general.default_mode.to_lowercase().as_str() {
            "random" => PasswordMode::Random,
            "pattern" => PasswordMode::Pattern,
            "phrase" => PasswordMode::Phrase,
            "deterministic" => PasswordMode::Deterministic,
            _ => PasswordMode::Random,
        };

        Self {
            current_screen: CurrentScreen::Main,
            generator: GeneratorState {
                length: config.general.default_length,
                count: config.general.default_count,
                mode: default_mode,
                pattern: String::new(),
                words: None,
                seed_env: String::new(),
                service: String::new(),
                salt: String::new(),
                selected_mode_index: 0,
                editing_field: None,
            },
            settings: Settings {
                language: config.language.lang,
                auto_save: config.general.auto_save,
                default_length: config.general.default_length,
                default_count: config.general.default_count,
                selected_index: 0,
            },
            help: Help { scroll: 0 },
            exit: Exit { exit: false },
            generated_passwords: Vec::new(),
            password_input: String::new(),
            selected_index: 0,
            show_detailed_check: false,
            check_result: None,
            input_mode: InputMode::Normal,
            input_field: InputField::None,
            error_message: None,
        }
    }

    pub fn handle_input(&mut self, key: KeyCode) -> bool {
        if self.input_mode == InputMode::Editing {
            return self.handle_editing_input(key);
        }

        match self.current_screen {
            CurrentScreen::Main => self.handle_main_input(key),
            CurrentScreen::Generator => self.handle_generator_input(key),
            CurrentScreen::Settings => self.handle_settings_input(key),
            CurrentScreen::Help => self.handle_help_input(key),
            CurrentScreen::Check => self.handle_check_input(key),
            CurrentScreen::Exit => self.handle_exit_input(key),
        }
    }

    fn handle_editing_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                self.input_field = InputField::None;
                if let InputField::Generator(GeneratorField::Pattern) = self.input_field {
                    self.generator.editing_field = None;
                }
                false
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.input_field = InputField::None;
                self.generator.editing_field = None;
                false
            }
            KeyCode::Char(c) => {
                match self.input_field {
                    InputField::PasswordCheck => {
                        self.password_input.push(c);
                    }
                    InputField::Generator(GeneratorField::Pattern) => {
                        self.generator.pattern.push(c);
                    }
                    InputField::Generator(GeneratorField::SeedEnv) => {
                        self.generator.seed_env.push(c);
                    }
                    InputField::Generator(GeneratorField::Service) => {
                        self.generator.service.push(c);
                    }
                    InputField::Generator(GeneratorField::Salt) => {
                        self.generator.salt.push(c);
                    }
                    InputField::Settings(SettingsField::Language) => {
                        self.settings.language.push(c);
                    }
                    _ => {}
                }
                false
            }
            KeyCode::Backspace => {
                match self.input_field {
                    InputField::PasswordCheck => {
                        self.password_input.pop();
                    }
                    InputField::Generator(GeneratorField::Pattern) => {
                        self.generator.pattern.pop();
                    }
                    InputField::Generator(GeneratorField::SeedEnv) => {
                        self.generator.seed_env.pop();
                    }
                    InputField::Generator(GeneratorField::Service) => {
                        self.generator.service.pop();
                    }
                    InputField::Generator(GeneratorField::Salt) => {
                        self.generator.salt.pop();
                    }
                    InputField::Settings(SettingsField::Language) => {
                        self.settings.language.pop();
                    }
                    _ => {}
                }
                false
            }
            _ => false,
        }
    }

    fn handle_main_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.current_screen = CurrentScreen::Exit;
                false
            }
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                false
            }
            KeyCode::Down => {
                if self.selected_index < 4 {
                    self.selected_index += 1;
                }
                false
            }
            KeyCode::Enter => {
                match self.selected_index {
                    0 => {
                        self.current_screen = CurrentScreen::Generator;
                        self.selected_index = 0;
                    }
                    1 => {
                        self.current_screen = CurrentScreen::Check;
                        self.password_input.clear();
                        self.check_result = None;
                        self.input_field = InputField::PasswordCheck;
                        self.input_mode = InputMode::Editing;
                    }
                    2 => {
                        self.current_screen = CurrentScreen::Settings;
                        self.selected_index = 0;
                    }
                    3 => {
                        self.current_screen = CurrentScreen::Help;
                        self.help.scroll = 0;
                    }
                    4 => {
                        self.current_screen = CurrentScreen::Exit;
                    }
                    _ => {}
                }
                false
            }
            _ => false,
        }
    }

    fn handle_generator_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Esc => {
                if self.input_mode == InputMode::Editing {
                    self.input_mode = InputMode::Normal;
                    self.input_field = InputField::None;
                    self.generator.editing_field = None;
                } else {
                    self.current_screen = CurrentScreen::Main;
                    self.selected_index = 0;
                    self.error_message = None;
                }
                false
            }
            KeyCode::Up => {
                if self.input_mode == InputMode::Normal && self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                false
            }
            KeyCode::Down => {
                if self.input_mode == InputMode::Normal {
                    let max_index = match self.generator.mode {
                        PasswordMode::Random => 1,
                        PasswordMode::Pattern => 1,
                        PasswordMode::Phrase => 1,
                        PasswordMode::Deterministic => 3,
                    };
                    if self.selected_index <= max_index {
                        self.selected_index += 1;
                        if self.selected_index > max_index {
                            self.selected_index = max_index;
                        }
                    }
                }
                false
            }
            KeyCode::Left => {
                if self.input_mode == InputMode::Normal {
                    if self.selected_index == 0 && self.generator.selected_mode_index > 0 {
                        self.generator.selected_mode_index -= 1;
                        self.generator.mode = match self.generator.selected_mode_index {
                            0 => PasswordMode::Random,
                            1 => PasswordMode::Pattern,
                            2 => PasswordMode::Phrase,
                            3 => PasswordMode::Deterministic,
                            _ => PasswordMode::Random,
                        };
                    } else {
                        match self.generator.mode {
                            PasswordMode::Random => {
                                if self.selected_index == 1 && self.generator.length > 8 {
                                    self.generator.length -= 1;
                                }
                            }
                            PasswordMode::Phrase => {
                                if self.selected_index == 1 {
                                    if let Some(ref mut words) = self.generator.words {
                                        if *words > 3 {
                                            *words -= 1;
                                        }
                                    } else {
                                        self.generator.words = Some(3);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                false
            }
            KeyCode::Right => {
                if self.input_mode == InputMode::Normal {
                    if self.selected_index == 0 && self.generator.selected_mode_index < 3 {
                        self.generator.selected_mode_index += 1;
                        self.generator.mode = match self.generator.selected_mode_index {
                            0 => PasswordMode::Random,
                            1 => PasswordMode::Pattern,
                            2 => PasswordMode::Phrase,
                            3 => PasswordMode::Deterministic,
                            _ => PasswordMode::Random,
                        };
                    } else {
                        match self.generator.mode {
                            PasswordMode::Random => {
                                if self.selected_index == 1 && self.generator.length < 64 {
                                    self.generator.length += 1;
                                }
                            }
                            PasswordMode::Phrase => {
                                if self.selected_index == 1 {
                                    if let Some(ref mut words) = self.generator.words {
                                        if *words < 20 {
                                            *words += 1;
                                        }
                                    } else {
                                        self.generator.words = Some(4);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                false
            }
            KeyCode::Enter => {
                if self.input_mode == InputMode::Editing {
                    self.input_mode = InputMode::Normal;
                    self.input_field = InputField::None;
                    self.generator.editing_field = None;
                    false
                } else if self.selected_index == 0 {
                    // Mode selection - do nothing
                    false
                } else {
                    self.generate_passwords();
                    false
                }
            }
            KeyCode::Char('e') => {
                if self.input_mode == InputMode::Normal {
                    match self.generator.mode {
                        PasswordMode::Pattern => {
                            if self.selected_index == 1 {
                                self.input_field = InputField::Generator(GeneratorField::Pattern);
                                self.generator.editing_field = Some(GeneratorField::Pattern);
                                self.input_mode = InputMode::Editing;
                            }
                        }
                        PasswordMode::Deterministic => match self.selected_index {
                            1 => {
                                self.input_field = InputField::Generator(GeneratorField::SeedEnv);
                                self.generator.editing_field = Some(GeneratorField::SeedEnv);
                                self.input_mode = InputMode::Editing;
                            }
                            2 => {
                                self.input_field = InputField::Generator(GeneratorField::Service);
                                self.generator.editing_field = Some(GeneratorField::Service);
                                self.input_mode = InputMode::Editing;
                            }
                            3 => {
                                self.input_field = InputField::Generator(GeneratorField::Salt);
                                self.generator.editing_field = Some(GeneratorField::Salt);
                                self.input_mode = InputMode::Editing;
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn handle_settings_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Esc => {
                self.current_screen = CurrentScreen::Main;
                self.selected_index = 0;
                false
            }
            KeyCode::Up => {
                if self.settings.selected_index > 0 {
                    self.settings.selected_index -= 1;
                }
                false
            }
            KeyCode::Down => {
                if self.settings.selected_index < 3 {
                    self.settings.selected_index += 1;
                }
                false
            }
            KeyCode::Enter => {
                self.save_settings();
                false
            }
            KeyCode::Left | KeyCode::Right => {
                match self.settings.selected_index {
                    1 => {
                        if self.settings.default_length > 8 {
                            self.settings.default_length -= 1;
                        }
                    }
                    2 => {
                        if self.settings.default_count > 1 {
                            self.settings.default_count -= 1;
                        }
                    }
                    3 => {
                        self.settings.auto_save = !self.settings.auto_save;
                    }
                    _ => {}
                }
                false
            }
            _ => false,
        }
    }

    fn handle_help_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.current_screen = CurrentScreen::Main;
                self.selected_index = 0;
                false
            }
            KeyCode::Up => {
                if self.help.scroll > 0 {
                    self.help.scroll -= 1;
                }
                false
            }
            KeyCode::Down => {
                self.help.scroll += 1;
                false
            }
            _ => false,
        }
    }

    fn handle_check_input(&mut self, key: KeyCode) -> bool {
        if self.input_mode == InputMode::Editing {
            match key {
                KeyCode::Enter => {
                    self.input_mode = InputMode::Normal;
                    self.input_field = InputField::None;
                    if !self.password_input.is_empty() {
                        self.check_password();
                    }
                    false
                }
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.input_field = InputField::None;
                    false
                }
                KeyCode::Char(c) => {
                    self.password_input.push(c);
                    false
                }
                KeyCode::Backspace => {
                    self.password_input.pop();
                    false
                }
                _ => false,
            }
        } else {
            match key {
                KeyCode::Esc => {
                    self.current_screen = CurrentScreen::Main;
                    self.selected_index = 0;
                    self.password_input.clear();
                    self.check_result = None;
                    self.input_field = InputField::None;
                    false
                }
                KeyCode::Enter => {
                    self.input_mode = InputMode::Editing;
                    self.input_field = InputField::PasswordCheck;
                    false
                }
                KeyCode::Char('d') => {
                    self.show_detailed_check = !self.show_detailed_check;
                    false
                }
                _ => false,
            }
        }
    }

    fn handle_exit_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char('y') | KeyCode::Enter => {
                self.exit.exit = true;
                true
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                self.current_screen = CurrentScreen::Main;
                self.selected_index = 0;
                false
            }
            _ => false,
        }
    }

    pub fn generate_passwords(&mut self) {
        self.error_message = None;
        self.generated_passwords.clear();

        for _ in 0..self.generator.count {
            let result = match self.generator.mode {
                PasswordMode::Random => Generator::generate_random_password(self.generator.length),
                PasswordMode::Pattern => {
                    if self.generator.pattern.is_empty() {
                        self.error_message = Some("Pattern is required".to_string());
                        return;
                    }
                    Generator::generate_pattern_password(&self.generator.pattern)
                }
                PasswordMode::Phrase => {
                    Generator::generate_phrase_password(self.generator.words.unwrap_or(4))
                }
                PasswordMode::Deterministic => {
                    if self.generator.seed_env.is_empty() {
                        self.error_message =
                            Some("Seed environment variable is required".to_string());
                        return;
                    }
                    let seed = std::env::var(&self.generator.seed_env).unwrap_or_default();
                    if seed.is_empty() {
                        self.error_message = Some(format!(
                            "Environment variable '{}' not found",
                            self.generator.seed_env
                        ));
                        return;
                    }
                    Generator::generate_deterministic_password(
                        &seed,
                        if self.generator.salt.is_empty() {
                            None
                        } else {
                            Some(&self.generator.salt)
                        },
                        if self.generator.service.is_empty() {
                            None
                        } else {
                            Some(&self.generator.service)
                        },
                    )
                }
            };

            match result {
                Ok(password) => self.generated_passwords.push(password),
                Err(e) => {
                    self.error_message = Some(format!("Error: {}", e));
                    return;
                }
            }
        }
    }

    pub fn check_password(&mut self) {
        self.check_result = Some(HealthCheck::analyze_password(&self.password_input));
    }

    pub fn save_settings(&mut self) {
        if let Err(e) = Config::update_config(
            Some(self.settings.language.clone()),
            Some(self.settings.default_length),
            Some(self.settings.default_count),
            Some(self.settings.auto_save),
        ) {
            self.error_message = Some(format!("Failed to save settings: {}", e));
        } else {
            self.error_message = None;
        }
    }
}
