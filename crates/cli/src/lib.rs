use std::{path::PathBuf, sync::Arc};

use clap::{builder::PossibleValue, Parser, Subcommand, ValueEnum};
use color_eyre::{
    eyre::{Context, OptionExt},
    Result,
};
use fstools_dvdbnd::{recover_keys, DvdBnd};
use fstools_game_id::GameId;
use fstools_game_installation::GameInstallation;

use crate::{
    describe::{describe_bnd, describe_entryfilelist, describe_matbin},
    extract::extract,
};

mod describe;
mod extract;
mod mount;
mod repl;

#[derive(Clone, Debug)]
pub struct Game(GameId);

impl Game {
    pub fn id(&self) -> GameId {
        self.0.clone()
    }
}

impl ValueEnum for Game {
    fn value_variants<'a>() -> &'a [Self] {
        &[Game(GameId::ELDEN_RING), Game(GameId::NIGHTREIGN)]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self.0 {
            GameId::ELDEN_RING => {
                Some(PossibleValue::new("elden-ring").aliases(["eldenring", "er"]))
            }
            GameId::NIGHTREIGN => {
                Some(PossibleValue::new("nightreign").aliases(["nightrein", "nr"]))
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[arg(long, short)]
    pub game: Game,

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

    /// Mount the DVDBND as a virtual filesystem
    Mount {
        /// Path to the mount point directory.
        mount_point: PathBuf,
    },

    Repl,
}

impl Action {
    pub fn run(self, dvd_bnd: &Arc<DvdBnd>) -> Result<()> {
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
            Action::Mount { mount_point } => {
                mount::mount_filesystem(Arc::clone(dvd_bnd), &mount_point)?;
            }
            Action::Repl => {
                repl::begin(dvd_bnd)?;
            }
        }

        Ok(())
    }
}

#[tracing::instrument(skip_all)]
pub fn run(cli: Cli) -> Result<()> {
    let Cli {
        game,
        command: action,
        ..
    } = cli;

    let installation = GameInstallation::find(game.id())
        .ok_or_eyre("couldn't find game installation directory")?;
    let keys = recover_keys(&installation.exe, &installation.bhds).context("scanning keys")?;

    let dvd_bnd =
        Arc::new(DvdBnd::create(&installation.bhds, &keys).with_context(|| "loading BHD/BDTs")?);
    action.run(&dvd_bnd)?;

    Ok(())
}
