use std::error::Error;

use clap::Parser;

pub fn main() -> Result<(), Box<dyn Error>> {
    fstools_cli::run(fstools_cli::Cli::parse())
}
