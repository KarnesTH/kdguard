pub mod check_command;
pub mod config_command;
pub mod generate_command;
pub mod help_command;

pub use check_command::CheckCommand;
pub use config_command::ConfigCommand;
pub use generate_command::GenerateCommand;
pub use help_command::HelpCommand;

pub type CommandResult = Result<(), Box<dyn std::error::Error>>;
