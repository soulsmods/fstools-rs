use clap::Parser;
use color_eyre::Result;
use tracing_subscriber::{prelude::*, EnvFilter};

pub fn main() -> Result<()> {
    color_eyre::install()?;

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_writer(std::io::stderr);
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(tracing_error::ErrorLayer::default())
        .init();

    fstools_cli::run(fstools_cli::Cli::parse())?;
    Ok(())
}
