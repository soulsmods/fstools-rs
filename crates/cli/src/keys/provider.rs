use std::{collections::HashMap, io};

use crate::GameInstallation;
use color_eyre::eyre::Result;
use fstools_dvdbnd::ArchiveKeyProvider;
use fstools_formats::bhd::BhdKey;
use tracing::debug;

use super::scanner::scan_keys;

pub struct ScannedArchiveKeyProvider {
    keys: HashMap<String, BhdKey>,
}

impl ScannedArchiveKeyProvider {
    pub fn scan(installation: &GameInstallation) -> Result<Self> {
        let keys = scan_keys(installation)?;
        debug!(keys = ?keys, "finished key scan");
        Ok(Self { keys })
    }
}

impl ArchiveKeyProvider for ScannedArchiveKeyProvider {
    fn get_key(&self, name: &str) -> Result<BhdKey, io::Error> {
        self.keys
            .get(name)
            .cloned()
            .ok_or_else(|| io::Error::other(format!("missing key for archive {name}")))
    }
}
