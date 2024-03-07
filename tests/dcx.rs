use std::{
    error::Error,
    io::{BufReader, Read},
    path::PathBuf,
};

use crc32fast::Hasher;
use fstools::{formats::dcx::DcxHeader, prelude::*};
use insta::assert_debug_snapshot;

#[test]
pub fn decode_kraken_dcx() -> Result<(), Box<dyn Error>> {
    let er_path = PathBuf::from(std::env::var("ER_PATH").expect("no elden ring path provided"));
    let keys = FileKeyProvider::new("keys");
    let archives = [
        er_path.join("Data0"),
        er_path.join("Data1"),
        er_path.join("Data2"),
        er_path.join("Data3"),
        er_path.join("sd/sd"),
    ];

    let vfs = Vfs::create(archives.clone(), &keys).expect("unable to create vfs");
    let file = vfs.open("/map/m60/m60_44_58_00/m60_44_58_00_445800.mapbnd.dcx")?;
    let (header, mut reader) = DcxHeader::read(file)?;
    let mut hasher = Hasher::new();

    let mut buffer = [0u8; 4096];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break, // End of file
            Ok(len) => hasher.update(&buffer[..len]),
            Err(e) => {
                // Handle the error more gracefully, e.g., return it or log it
                eprintln!("Error reading data: {}", e);
                break;
            }
        }
    }

    let hash = hasher.finalize();
    assert_debug_snapshot!((header, hash));

    Ok(())
}
