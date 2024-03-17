mod describe;
mod extract;

use std::{error::Error, path::PathBuf};

use clap::{Parser, Subcommand, ValueEnum};
use describe::describe_matbin;
use fstools_dvdbnd::{DvdBnd, FileKeyProvider};

use crate::{describe::describe_bnd, extract::extract};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    #[arg(long)]
    game_path: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum AssetType {
    Bnd,
    Matbin,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Describe the asset with a given type and name.
    Describe {
        #[arg(value_enum)]
        ty: AssetType,

        name: String,
    },
    /// Extract the contents of the DVDBND.
    Extract {
        /// Extract the contents of BNDs inside the DVDBND?
        recursive: Option<bool>,

        /// A file name filter applied to files being extracted.
        filter: Option<String>,

        output_path: PathBuf,

        dictionary_path: PathBuf,
    },
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let Args { game_path, command } = Args::parse();

    let keys = FileKeyProvider::new("keys");
    let archives = [
        game_path.join("Data0"),
        game_path.join("Data1"),
        game_path.join("Data2"),
        game_path.join("Data3"),
        game_path.join("sd/sd"),
    ];

    let dvd_bnd = DvdBnd::create(archives, &keys)?;

    match command {
        Command::Describe {
            ty: AssetType::Bnd,
            name,
        } => {
            describe_bnd(dvd_bnd, &name)?;
        }
        Command::Describe {
            ty: AssetType::Matbin,
            name,
        } => {
            describe_matbin(dvd_bnd, &name)?;
        }
        Command::Extract {
            recursive,
            filter,
            output_path,
            dictionary_path,
        } => {
            extract(
                dvd_bnd,
                recursive.unwrap_or(false),
                filter,
                output_path,
                dictionary_path,
            )?;
        }
    }

    Ok(())
}
