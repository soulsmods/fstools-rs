use std::{collections::HashSet, error::Error, path::PathBuf, sync::Arc};

use fstools::{formats::dcx::DcxHeader, prelude::*};
use fstools_formats::dcx::DcxError;
use insta::assert_snapshot;
use libtest_mimic::{Arguments, Failed, Trial};

fn main() -> Result<(), Box<dyn Error>> {
    let args = Arguments::from_args();
    let er_path = PathBuf::from(std::env::var("ER_PATH").expect("er_path"));
    let keys_path = PathBuf::from(std::env::var("ER_KEYS_PATH").expect("er_keys_path"));
    let keys = FileKeyProvider::new(keys_path);
    let archives = [
        er_path.join("Data0"),
        er_path.join("Data1"),
        er_path.join("Data2"),
        er_path.join("Data3"),
        er_path.join("sd/sd"),
    ];

    let vfs = Arc::new(DvdBnd::create(archives.clone(), &keys).expect("unable to create dvdbnd"));

    let dictionary = include_str!("data/EldenRingDictionary.txt");
    let lines = dictionary
        .lines()
        .filter(|line| line.ends_with(".dcx"))
        .collect::<HashSet<_>>();

    let mut tests = vec![];

    assert_snapshot!(lines.len());

    for line in lines {
        let vfs = vfs.clone();
        let test = Trial::test(line, move || check_file(vfs.clone(), line)).with_kind("dcx");

        tests.push(test);
    }

    libtest_mimic::run(&args, tests).exit();
}

pub fn check_file(vfs: Arc<DvdBnd>, file: &str) -> Result<(), Failed> {
    let file = match vfs.open(file) {
        Ok(file) => file,
        Err(_) => {
            return Ok(());
        }
    };

    let (_, mut reader) = match DcxHeader::read(file) {
        Ok(details) => details,
        Err(DcxError::UnknownAlgorithm(_)) => return Ok(()),
        Err(_) => return Err("failed to parse DCX header".into()),
    };

    std::io::copy(&mut reader, &mut std::io::sink())?;

    Ok(())
}
