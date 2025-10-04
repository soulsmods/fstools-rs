use std::{
    collections::HashSet,
    ffi::OsStr,
    io::{self, Read},
    path::{Path, PathBuf},
    sync::Arc,
};

use color_eyre::{eyre::WrapErr, Result};
use fstools::{formats::dcx::DcxHeader, prelude::*};
use fstools_elden_ring_support::{decrypt_regulation, dictionary};
use fstools_formats::dcx::DcxError;
use insta::assert_snapshot;
use libtest_mimic::{Arguments, Failed, Trial};

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Arguments::from_args();
    let er_path = PathBuf::from(std::env::var("ER_PATH").wrap_err("ER_PATH not set")?);
    let reg_path = er_path.join("regulation.bin");
    let keys_path = PathBuf::from(std::env::var("ER_KEYS_PATH").wrap_err("ER_KEYS_PATH not set")?);
    let vfs = Arc::new(fstools_elden_ring_support::load_dvd_bnd(
        er_path,
        FileKeyProvider::new(keys_path),
    )?);

    let lines = dictionary()
        .filter(|line| line.extension() == Some(OsStr::new("dcx")))
        .collect::<HashSet<_>>();

    let mut tests = vec![];

    assert_snapshot!(lines.len());

    for line in lines {
        let vfs = vfs.clone();
        let test = Trial::test(line.to_string_lossy().to_string(), move || {
            check_file(vfs.clone(), &line)
        })
        .with_kind("dcx");

        tests.push(test);
    }

    // Test against the regulation DCX (ZSTD encoded since 1.12)
    tests.push(Trial::test("regulation.bin", move || {
        check_regulation(&reg_path)
    }));

    libtest_mimic::run(&args, tests).exit();
}

pub fn check_regulation(path: &Path) -> Result<(), Failed> {
    let regulation_bytes = std::fs::read(path.join("regulation.bin"))?;
    let dcx_bytes = decrypt_regulation(&mut regulation_bytes.as_slice())?;
    check_dcx(io::Cursor::new(dcx_bytes))
}

pub fn check_file(vfs: Arc<DvdBnd>, file: &Path) -> Result<(), Failed> {
    let file = match vfs.open(file.to_string_lossy().as_ref()) {
        Ok(file) => file,
        Err(_) => {
            return Ok(());
        }
    };
    check_dcx(file)
}

pub fn check_dcx(reader: impl Read) -> Result<(), Failed> {
    let (_, mut decoder) = match DcxHeader::read(reader) {
        Ok(details) => details,
        Err(DcxError::UnknownAlgorithm(_)) => return Ok(()),
        Err(_) => return Err("failed to parse DCX header".into()),
    };

    std::io::copy(&mut decoder, &mut std::io::sink())?;

    Ok(())
}
