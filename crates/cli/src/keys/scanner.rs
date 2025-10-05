use std::{
    collections::{HashMap, HashSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use crate::GameInstallation;
use color_eyre::eyre::{ensure, eyre, Context, Result};
use fstools_formats::bhd::BhdKey;
use regex::bytes::Regex;

struct Candidate {
    key: BhdKey,
}

pub fn scan_keys(installation: &GameInstallation) -> Result<HashMap<String, BhdKey>> {
    let executable_path = &installation.exe;
    let bytes = std::fs::read(&installation.exe)?;

    let candidates = find_pem_candidates(&bytes)?;
    ensure!(
        !candidates.is_empty(),
        "no RSA keys detected in {}",
        executable_path.display()
    );

    match_keys_to_archives(candidates, installation)
}

fn find_pem_candidates(bytes: &[u8]) -> Result<Vec<Candidate>> {
    let mut seen = HashSet::new();
    let mut candidates = Vec::new();
    let regex = Regex::new(
        r"-----BEGIN RSA PUBLIC KEY-----\s+([A-Za-z0-9+/=\r\n]+?)-----END RSA PUBLIC KEY-----",
    )?;

    for caps in regex.captures_iter(bytes) {
        let full = caps.get(0).ok_or_else(|| eyre!("regex capture missing"))?;
        let offset = full.start();
        let pem = String::from_utf8(full.as_bytes().to_vec())
            .wrap_err_with(|| format!("invalid UTF-8 in PEM block at offset 0x{offset:x}"))?;
      
        if !seen.insert(pem.clone()) {
            continue;
        }

        match BhdKey::from_pem(&pem) {
            Ok(key) => candidates.push(Candidate { key }),
            Err(err) => {
                tracing::warn!("skipping candidate at offset 0x{offset:x}: {err}");
            }
        }
    }

    Ok(candidates)
}

fn match_keys_to_archives(
    candidates: Vec<Candidate>,
    install: &GameInstallation,
) -> Result<HashMap<String, BhdKey>> {
    let mut remaining: Vec<PathBuf> = install.bhds.clone();
    let mut matched = HashMap::new();

    for candidate in candidates {
        if remaining.is_empty() {
            break;
        }

        let mut matched_index = None;

        for (index, path) in remaining.iter().enumerate() {
            if try_key_on_bhd(&candidate.key, path)? {
                matched.insert(
                    path.file_stem().unwrap().to_string_lossy().into_owned(),
                    candidate.key.clone(),
                );
                matched_index = Some(index);
                break;
            }
        }

        if let Some(index) = matched_index {
            remaining.remove(index);
        }
    }

    ensure!(
        remaining.is_empty(),
        "failed to match keys for archives: {}",
        remaining
            .into_iter()
            .map(|path| format!("{}", path.display()))
            .collect::<Vec<_>>()
            .join(", ")
    );

    Ok(matched)
}

fn try_key_on_bhd(key: &BhdKey, bhd_path: &Path) -> Result<bool> {
    let mut file = fs::File::open(bhd_path)
        .with_context(|| format!("failed to open BHD file {}", bhd_path.display()))?;

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
