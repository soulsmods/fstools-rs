use std::{error::Error, path::PathBuf};

use clap::{Parser, Subcommand, ValueEnum};
use fstools_dvdbnd::{DvdBnd, FileKeyProvider};

use crate::{
    describe::{describe_bnd, describe_entryfilelist, describe_matbin},
    extract::extract,
};

mod describe;
mod extract;
mod repl;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[arg(long, env("ER_PATH"))]
    pub game_path: PathBuf,

    #[command(subcommand)]
    pub command: Action,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum AssetType {
    Bnd,
    EntryFileList,
    Matbin,
}

#[derive(Debug, Subcommand)]
pub enum Action {
    /// Describe the asset with a given type and name.
    Describe {
        #[arg(value_enum)]
        ty: AssetType,

        name: String,
    },
    /// Extract the contents of the DVDBND.
    Extract {
        /// Extract the contents of BNDs inside the DVDBND?
        #[arg(short, long)]
        recursive: bool,

        /// A file name filter applied to files being extracted.
        filter: Option<String>,

        /// Path to a folder that files will be extracted to.
        #[arg(short, long, default_value("./extract"))]
        output_path: PathBuf,
    },

    Repl,
}

impl Action {
    pub fn run(self, dvd_bnd: &DvdBnd) -> Result<(), Box<dyn Error>> {
        match self {
            Action::Describe {
                ty: AssetType::Bnd,
                name,
            } => {
                describe_bnd(dvd_bnd, &name)?;
            }
            Action::Describe {
                ty: AssetType::EntryFileList,
                name,
            } => {
                describe_entryfilelist(dvd_bnd, &name)?;
            }
            Action::Describe {
                ty: AssetType::Matbin,
                name,
            } => {
                describe_matbin(dvd_bnd, &name)?;
            }
            Action::Extract {
                recursive,
                filter,
                output_path,
            } => {
                extract(dvd_bnd, recursive, filter, output_path)?;
            }
            Action::Repl => {
                repl::begin(dvd_bnd)?;
            }
        }

        Ok(())
    }
}

pub fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    let Cli {
        game_path,
        command: action,
    } = cli;
    let keys = FileKeyProvider::new("keys");
    let archives = [
        game_path.join("Data0"),
        game_path.join("Data1"),
        game_path.join("Data2"),
        game_path.join("Data3"),
        game_path.join("sd/sd"),
    ];

    let dvd_bnd = DvdBnd::create(archives, &keys)?;
    action.run(&dvd_bnd)?;

    Ok(())
}
