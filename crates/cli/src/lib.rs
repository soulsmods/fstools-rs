use std::{path::PathBuf, sync::Arc};

use clap::{Parser, Subcommand, ValueEnum};
use color_eyre::{
    eyre::{Context, OptionExt},
    Result,
};
use fstools_dvdbnd::DvdBnd;

use crate::{
    describe::{describe_bnd, describe_entryfilelist, describe_matbin},
    extract::extract,
};

mod describe;
mod extract;
mod keys;
mod mount;
mod repl;

#[derive(Debug, ValueEnum, Clone)]

pub enum Game {
    #[clap(alias = "er")]
    EldenRing,

    #[clap(alias = "nr")]
    EldenRingNightreign,
}

impl Game {
    pub fn app_id(&self) -> u32 {
        match self {
            Game::EldenRing => 1245620,
            Game::EldenRingNightreign => 2622380,
        }
    }
}

pub struct GameInstallation {
    root: PathBuf,
    exe: PathBuf,
    bhds: Vec<PathBuf>,
}

pub fn find_installation_dir(game: Game) -> Option<PathBuf> {
    let steam = steamlocate::SteamDir::locate().ok()?;
    let (app, library) = steam.find_app(game.app_id()).ok().flatten()?;
    let root = library.resolve_app_dir(&app);

    Some(root)
}

pub fn find_game_installation(root: PathBuf) -> Option<GameInstallation> {
    let exe = glob::glob(&format!("{}/Game/*.exe", root.display()))
        .ok()?
        .filter_map(std::result::Result::ok).find(|path| !path.ends_with("start_protected_game.exe"))?;
    let bhds = glob::glob(&format!("{}/Game/*.bhd", root.display()))
        .ok()?
        .filter_map(std::result::Result::ok)
        .collect();

    Some(GameInstallation { root, exe, bhds })
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[arg(long, short, env("ER_PATH"))]
    pub game: Game,

    #[arg(long, env("ER_PATH"))]
    pub game_path: Option<PathBuf>,

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
        game_path,
        command: action,
    } = cli;
    let game_install_dir = game_path
        .or_else(|| find_installation_dir(game))
        .ok_or_eyre("couldn't find game installation directory")?;
    let game_install =
        find_game_installation(game_install_dir).ok_or_eyre("couldn't find archives/exe")?;
    let key_provider = keys::ScannedArchiveKeyProvider::scan(&game_install)
        .with_context(|| format!("scanning keys in {:?}", game_install.root))?;

    let dvd_bnd = Arc::new(
        DvdBnd::create(&game_install.bhds, &key_provider).with_context(|| "loading BHD/BDTs")?,
    );
    action.run(&dvd_bnd)?;

    Ok(())
}
