use std::{
    env,
    error::Error,
    fs,
    fs::File,
    io,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, RwLock},
};

use directories::ProjectDirs;
use serde_derive::{Deserialize, Serialize};
use toml::{Table, Value};

use crate::paths::Paths;

mod paths;

#[derive(Default, Deserialize, Serialize)]
struct Options {
    paths: Paths,
}

thread_local! {
    static CURRENT_CONFIG: RwLock<Arc<Options>> = RwLock::new(Default::default());
}

fn env_override_str<T: FromStr>(value: &mut Option<T>, env_name: &str) {
    if let Some(env_var) = env::var(env_name)
        .ok()
        .and_then(|env_var| T::from_str(&env_var).ok())
    {
        *value = Some(env_var);
    }
}

impl Options {
    pub fn path() -> Option<PathBuf> {
        const SETTINGS_FILENAME: &str = "settings.toml";

        let dirs = ProjectDirs::from("io.github", "soulsmods", "fstools")?;
        let config_dir = dirs.config_dir();

        Some(config_dir.join("settings.toml"))
    }

    pub fn save(&self) -> Result<(), io::Error> {
        let config_path =
            Self::path().ok_or(io::Error::other("Couldn't determine config directory"))?;

        let output = toml::to_string_pretty(self).map_err(io::Error::other)?;
        fs::write(config_path, output)?;

        Ok(())
    }

    pub fn load() -> Result<Self, io::Error> {
        let config_path =
            Self::path().ok_or(io::Error::other("Couldn't determine config directory"))?;

        let mut options = fs::read_to_string(config_path)
            .and_then(|contents| toml::from_str::<Options>(&contents).map_err(io::Error::other))
            .unwrap_or_default();

        env_override_str(&mut options.paths.elden_ring, "ER_PATH");
        env_override_str(&mut options.paths.elden_ring_keys, "ER_KEYS_PATH");

        Ok(options)
    }

    pub fn current() -> Arc<Options> {
        CURRENT_CONFIG.with(|c| c.read().expect("config_r_lock").clone())
    }

    pub fn make_current(self) {
        CURRENT_CONFIG.with(|c| *c.write().expect("config_w_lock") = Arc::new(self));
    }
}
