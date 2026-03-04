use std::{
    collections::HashMap,
    fs,
    io::Read as _,
    path::{Path, PathBuf},
};

use fstools_formats::bhd::BhdKey;
use regex::bytes::Regex;
use thiserror::Error;
use tracing::warn;

#[derive(Debug, Error)]
pub enum KeyScanError {
    #[error("No PEM data found in executable")]
    NoKeysFound,

    #[error("Unable to match RSA key to archives: {0:?}")]
    NoMatchFound(Vec<String>),

    #[error("I/O error")]
    Io(#[from] std::io::Error),
}

pub fn recover_keys(
    executable_path: &Path,
    archives: &[PathBuf],
) -> Result<HashMap<String, BhdKey>, KeyScanError> {
    let bytes = std::fs::read(executable_path)?;

    let candidates = find_pem_candidates(&bytes)?;
    if candidates.is_empty() {
        return Err(KeyScanError::NoKeysFound);
    }

    match_keys_to_archives(candidates, archives)
}

fn find_pem_candidates(bytes: &[u8]) -> Result<Vec<BhdKey>, KeyScanError> {
    let mut candidates = Vec::new();

    let regex = Regex::new(
        r"-----BEGIN RSA PUBLIC KEY-----\s+([A-Za-z0-9+/=\r\n]+?)-----END RSA PUBLIC KEY-----",
    )
    .expect("PEM regex is invalid");

    for caps in regex.captures_iter(bytes) {
        let full = caps.get(0).expect("must be present");
        let offset = full.start();
        let Ok(pem) = String::from_utf8(full.as_bytes().to_vec()) else {
            continue;
        };

        match BhdKey::from_pem(&pem) {
            Ok(key) => candidates.push(key),
            Err(err) => {
                warn!("skipping candidate at offset 0x{offset:x}: {err}");
            }
        }
    }

    Ok(candidates)
}

fn match_keys_to_archives(
    candidates: Vec<BhdKey>,
    archives: &[PathBuf],
) -> Result<HashMap<String, BhdKey>, KeyScanError> {
    let mut remaining: Vec<PathBuf> = archives.to_vec();
    let mut matched = HashMap::new();

    for candidate in candidates {
        if remaining.is_empty() {
            break;
        }

        let mut matched_index = None;

        for (index, path) in remaining.iter().enumerate() {
            if try_key_on_bhd(&candidate, path)? {
                matched.insert(
                    path.file_stem()
                        .expect("no file name?")
                        .to_string_lossy()
                        .into_owned(),
                    candidate.clone(),
                );
                matched_index = Some(index);
                break;
            }
        }

        if let Some(index) = matched_index {
            remaining.remove(index);
        }
    }

    if !remaining.is_empty() {
        let remaining_paths = remaining
            .into_iter()
            .map(|path| format!("{}", path.display()))
            .collect::<Vec<_>>();

        return Err(KeyScanError::NoMatchFound(remaining_paths));
    }

    Ok(matched)
}

fn try_key_on_bhd(key: &BhdKey, bhd_path: &Path) -> Result<bool, KeyScanError> {
    let mut file = fs::File::open(bhd_path)?;

    let mut first_block = vec![0u8; key.input_size()];
    let mut read = 0;

    while read < first_block.len() {
        let bytes = file.read(&mut first_block[read..])?;
        if bytes == 0 {
            break;
        }
        read += bytes;
    }

    let decrypted = key.decrypt_block(&first_block);
    Ok(decrypted.starts_with(b"BHD"))
}
