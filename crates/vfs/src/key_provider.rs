use std::{fs, path::PathBuf};

use fstools_formats::bhd::BhdKey;

// TODO: replace Option with Result
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

impl ArchiveKeyProvider for FileKeyProvider {
    fn get_key(&self, name: &str) -> Result<BhdKey, std::io::Error> {
        fs::read_to_string(self.key_dir.join(name).with_extension("pem"))
            .and_then(|pem| BhdKey::from_pem(&pem).map_err(std::io::Error::other))
    }
}
