use std::{collections::HashMap, fs, io, path::PathBuf};

use fstools_formats::bhd::BhdKey;

pub trait ArchiveKeyProvider {
    fn get_key(&self, name: &str) -> Result<BhdKey, std::io::Error>;
}

pub struct FileKeyProvider {
    key_dir: PathBuf,
}

impl FileKeyProvider {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            key_dir: path.into(),
        }
    }
}

impl ArchiveKeyProvider for HashMap<String, BhdKey> {
    fn get_key(&self, name: &str) -> Result<BhdKey, std::io::Error> {
        self.get(name)
            .cloned()
            .ok_or_else(|| io::Error::other(format!("missing key for archive {name}")))
    }
}

impl ArchiveKeyProvider for FileKeyProvider {
    #[tracing::instrument(skip(self))]
    fn get_key(&self, name: &str) -> Result<BhdKey, std::io::Error> {
        fs::read_to_string(self.key_dir.join(name).with_extension("pem"))
            .and_then(|pem| BhdKey::from_pem(&pem).map_err(std::io::Error::other))
    }
}
