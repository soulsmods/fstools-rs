use std::path::PathBuf;

use fstools_game_id::GameId;

#[derive(Clone)]
pub struct GameInstallation {
    pub root: PathBuf,
    pub exe: PathBuf,
    pub bhds: Vec<PathBuf>,
}

impl GameInstallation {
    pub fn find(id: GameId) -> Option<GameInstallation> {
        let root = find_installation_dir(id.steam_app_id)?;
        let exe = root.join(id.exe);
        let bhds = glob::glob(&format!("{}/Game/*.bhd", root.display()))
            .ok()?
            .filter_map(std::result::Result::ok)
            .collect();

        Some(GameInstallation { root, exe, bhds })
    }
}

pub fn find_installation_dir(game: u32) -> Option<PathBuf> {
    let steam = steamlocate::SteamDir::locate().ok()?;
    let (app, library) = steam.find_app(game).ok().flatten()?;
    let root = library.resolve_app_dir(&app);

    Some(root)
}
