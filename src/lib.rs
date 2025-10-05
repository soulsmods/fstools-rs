use std::{
    io::{Cursor, Read, Seek},
    path::{Path, PathBuf},
};

use fstools_dvdbnd::{recover_keys, ArchiveKeyProvider, DvdBnd, DvdBndEntryError, KeyScanError};
use fstools_formats::{
    bnd4::{BND4Entry, Bnd4Header},
    dcx::DcxHeader,
    tpf::{Texture, TPF},
};
use fstools_game_id::GameId;
use fstools_game_installation::GameInstallation;
use rayon::prelude::{
    IntoParallelIterator, IntoParallelRefIterator as _, ParallelBridge, ParallelIterator as _,
};

pub mod formats {
    pub use fstools_formats::*;
}

pub mod dvdbnd {
    pub use fstools_dvdbnd::*;
}

pub mod game {
    pub use fstools_game_id::*;
    pub use fstools_game_installation::*;
}

pub mod prelude {
    pub use super::{dvdbnd::*, formats::*, game::*};
}

pub struct Assets {
    dvd_bnd: DvdBnd,
    dictionary: Vec<PathBuf>,
    installation: GameInstallation,
}

#[derive(thiserror::Error, Debug)]
pub enum GameAssetsOpenError {
    #[error("an installation of the game could not be found")]
    GameNotFound,

    #[error("failed to read on-disk game data")]
    Io(#[from] std::io::Error),

    #[error("an error occurred during DVDBND key recovery")]
    NoKeysRecovered(#[from] KeyScanError),
}

#[derive(thiserror::Error, Debug)]
pub enum AssetsIndexError {
    #[error("failed to read BND metadata")]
    BndFailure,

    #[error("failed to read data of DVDBND entry")]
    InvalidDvdBndEntry(#[from] DvdBndEntryError),

    #[error("failed to read on-disk game data")]
    Io(#[from] std::io::Error),

    #[error("unrecognised file type")]
    UnknownFileType,
}

#[derive(Debug)]
pub struct AssetIndexEntry {
    owner: PathBuf,
    kind: AssetIndexEntryKind,
}

#[derive(Debug)]
pub enum AssetIndexEntryKind {
    File,
    BndEntry(BND4Entry),
    TpfEntry(Texture),
}

trait ReadAndSeek: Read + Seek {}
impl<R: Read + Seek> ReadAndSeek for R {}

impl Assets {
    pub fn open(id: GameId) -> Result<Self, GameAssetsOpenError> {
        let install = GameInstallation::find(id).ok_or(GameAssetsOpenError::GameNotFound)?;
        let keys = recover_keys(&install.exe, &install.bhds)?;

        Self::open_with(install, &keys)
    }

    pub fn open_with(
        installation: GameInstallation,
        keys: &impl ArchiveKeyProvider,
    ) -> Result<Self, GameAssetsOpenError> {
        let dvd_bnd = DvdBnd::create(&installation.bhds, keys)?;

        Ok(Self {
            dvd_bnd,
            dictionary: vec![],
            installation,
        })
    }

    pub fn with_dictionary(self, dictionary: impl IntoIterator<Item = PathBuf>) -> Self {
        Self {
            dictionary: dictionary.into_iter().collect(),
            ..self
        }
    }

    pub fn index(&self) -> (Vec<AssetIndexEntry>, Vec<AssetsIndexError>) {
        let (results, errors): (Vec<_>, Vec<AssetsIndexError>) = self
            .dictionary
            .par_iter()
            .map(|path| {
                let mut file = self.dvd_bnd.open(path)?;
                let is_compressed = path.extension().is_some_and(|ext| ext == "dcx");
                let decompressed_path = if is_compressed {
                    path.file_stem().map(Path::new)
                } else {
                    Some(path.as_path())
                };

                let decompressed_extension = decompressed_path
                    .and_then(|path| path.extension()?.to_str())
                    .ok_or(AssetsIndexError::UnknownFileType)?;

                let create_reader = || -> Result<Box<dyn ReadAndSeek>, AssetsIndexError> {
                    if is_compressed {
                        let (dcx_header, mut dcx_reader) = DcxHeader::read(&mut file).unwrap();
                        let mut contents =
                            Vec::with_capacity(dcx_header.sizes().decompressed() as usize);

                        dcx_reader.read_to_end(&mut contents)?;
                        Ok(Box::new(Cursor::new(contents)))
                    } else {
                        Ok(Box::new(file))
                    }
                };

                let entries = match decompressed_extension {
                    "tpf" => {
                        let mut reader = create_reader()?;
                        let header = TPF::from_reader(&mut reader)?;
                        header
                            .textures
                            .into_iter()
                            .map(AssetIndexEntryKind::TpfEntry)
                            .collect()
                    }
                    _ if decompressed_extension.ends_with("bnd") => {
                        let mut reader = create_reader()?;
                        let header = Bnd4Header::from_reader(&mut reader)?;

                        header
                            .files
                            .into_iter()
                            .map(AssetIndexEntryKind::BndEntry)
                            .collect()
                    }
                    _ => vec![AssetIndexEntryKind::File],
                };

                Ok::<_, AssetsIndexError>(entries.into_iter().par_bridge().map(|entry| {
                    AssetIndexEntry {
                        kind: entry,
                        owner: path.clone(),
                    }
                }))
            })
            .partition_map(|result| match result {
                Ok(v) => rayon::iter::Either::Left(v),
                Err(e) => rayon::iter::Either::Right(e),
            });

        (results.into_par_iter().flatten().collect(), errors)
    }
}
