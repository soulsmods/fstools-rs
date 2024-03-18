use std::{io, path::PathBuf};

use fstools::dvdbnd::{ArchiveKeyProvider, DvdBnd};

pub fn load_dvd_bnd(
    game_path: PathBuf,
    keys: impl ArchiveKeyProvider,
) -> Result<DvdBnd, io::Error> {
    let archives = [
        game_path.join("Data0"),
        game_path.join("Data1"),
        game_path.join("Data2"),
        game_path.join("Data3"),
        game_path.join("sd/sd"),
    ];

    DvdBnd::create(archives, &keys)
}

pub fn dictionary() -> impl Iterator<Item = PathBuf> {
    include_str!("data/EldenRingDictionary.txt")
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(std::path::PathBuf::from)
}
