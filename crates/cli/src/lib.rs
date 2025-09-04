use std::{error::Error, path::PathBuf};

use clap::{Parser, Subcommand, ValueEnum};
use fstools_dvdbnd::{
    DvdBnd, FileKeyProvider,
    GameType::{EldenRing, Nightreign},
};

use crate::{
    describe::{
        describe_bnd, describe_entryfilelist, describe_flver, describe_matbin, describe_msb,
    },
    extract::extract,
};

mod describe;
mod extract;
mod repl;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[arg(long, env("GAME_PATH"))]
    pub game_path: PathBuf,

    #[arg(long, value_enum, env("GAME_TYPE"))]
    pub game_type: GameType,

    #[command(subcommand)]
    pub command: Action,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum GameType {
    ErPc,
    NrPc,
}

impl From<GameType> for fstools_dvdbnd::GameType {
    fn from(val: GameType) -> Self {
        match val {
            GameType::ErPc => EldenRing,
            GameType::NrPc => Nightreign,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum AssetType {
    Bnd,
    EntryFileList,
    Flver,
    Matbin,
    Msb,
}

#[derive(Debug, Subcommand)]
pub enum Action {
    /// Describe the asset with a given type and name.
    Describe {
        #[arg(
            short,
            long,
            required = false,
            value_delimiter = ',',
            help = "Chain of nested bnd names. Required to describe a file therein.\nExamples:\n    Describe a tae inside the anibnd of an objbnd:\n    -n obj\\o000100.objbnd.dcx, o000100.anibnd tae o000100.tae\n    Describe a flver inside a chrbnd:\n    -n chr\\c3000.chrbnd.dcx flver c3000.flver"
        )]
        nested_bnd_names: Vec<String>,

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
    pub fn run(self, dvd_bnd: &DvdBnd, game_type: &GameType) -> Result<(), Box<dyn Error>> {
        match self {
            Action::Describe {
                nested_bnd_names,
                ty: AssetType::Bnd,
                name,
            } => {
                describe_bnd(dvd_bnd, &name, &nested_bnd_names)?;
            }
            Action::Describe {
                nested_bnd_names: _nested_bnd_names,
                ty: AssetType::EntryFileList,
                name,
            } => {
                describe_entryfilelist(dvd_bnd, &name)?;
            }
            Action::Describe {
                nested_bnd_names,
                ty: AssetType::Flver,
                name,
            } => {
                describe_flver(dvd_bnd, &name, &nested_bnd_names)?;
            }
            Action::Describe {
                nested_bnd_names,
                ty: AssetType::Matbin,
                name,
            } => {
                describe_matbin(dvd_bnd, &name, &nested_bnd_names)?;
            }
            Action::Describe {
                nested_bnd_names,
                ty: AssetType::Msb,
                name,
            } => {
                describe_msb(dvd_bnd, &name, &nested_bnd_names, game_type)?;
            }
            Action::Extract {
                recursive,
                filter,
                output_path,
            } => {
                extract(dvd_bnd, recursive, filter, output_path, *game_type)?;
            }
            Action::Repl => {
                repl::begin(dvd_bnd, game_type)?;
            }
        }

        Ok(())
    }
}

pub fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    let Cli {
        game_path,
        game_type,
        command: action,
    } = cli;
    let game_key_dir = match game_type {
        GameType::ErPc => "/er_pc",
        GameType::NrPc => "/nr_pc",
    };
    let keys = FileKeyProvider::new(format!("keys{}", game_key_dir));

    let dvd_bnd = DvdBnd::create_from_game(game_type.into(), game_path, keys)?;
    action.run(&dvd_bnd, &game_type)?;

    Ok(())
}
