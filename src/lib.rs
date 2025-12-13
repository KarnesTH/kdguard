use clap::Parser;

mod generator;

#[derive(Parser)]
#[command(
    version,
    about = "A CLI tool to generate secure and random passwords",
    author = "KarnesTH <p_haehnel@hotmail.de>"
)]
pub struct Cli {
    #[clap(short, long, help = "Length of your password", default_value_t = 16)]
    pub length: usize,
    #[clap(short, long, help = "Amount of passwords", default_value_t = 1)]
    pub count: usize,
    #[clap(
        short,
        long,
        help = "If you want to save the password to a file",
        default_value_t = false
    )]
    pub save: bool,
    #[clap(short, long, help = "Output name to save a file (e.g. passgen.txt)")]
    pub output: Option<String>,
}
pub mod prelude {
    pub use crate::Cli;
    pub use crate::generator::Generator;
}
