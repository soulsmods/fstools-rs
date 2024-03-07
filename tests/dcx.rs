use std::{error::Error, path::PathBuf};

use format::dcx::DcxHeader;
pub use fstools::prelude::*;

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
    let (_, mut reader) = DcxHeader::read(file)?;

    std::io::copy(&mut reader, &mut std::io::sink())?;

    Ok(())
}
