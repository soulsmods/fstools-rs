use std::fs;
use std::path::{Path, PathBuf};

use format::bhd2::BhdKey;

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
        fs::read(self.key_dir.join(name).with_extension("pem")).and_then(|pem| {
            BhdKey::public_key_from_pem_pkcs1(&pem[..]).map_err(|err| std::io::Error::other(err))
        })
    }
}
