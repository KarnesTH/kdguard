use clap::Parser;

mod generator;

#[derive(Parser)]
#[command(
    version,
    about = "A simple password generator with a password length between 8 and 64."
)]
pub struct Cli {
    #[clap(short, long, help = "Length of your password")]
    pub length: usize,
    #[clap(short, long, help = "Amount of passwords")]
    pub count: Option<usize>,
    #[clap(
        short,
        long,
        help = "If you want to save the password to a file",
        default_value_t = false
    )]
    pub save: bool,
    #[clap(short, long, help = "Output path to save a file")]
    pub output: Option<String>,
}
pub mod prelude {
    pub use crate::Cli;
    pub use crate::generator::Generator;
}
