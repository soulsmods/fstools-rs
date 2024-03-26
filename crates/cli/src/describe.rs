use std::{
    error::Error,
    io::{Cursor, Read},
};

use fstools_dvdbnd::DvdBnd;
use fstools_formats::{bnd4::BND4, dcx::DcxHeader, entryfilelist::EntryFileListContainer};

pub fn describe_bnd(dvd_bnd: &DvdBnd, name: &str) -> Result<(), Box<dyn Error>> {
    let (dcx, mut reader) = DcxHeader::read(dvd_bnd.open(name)?)?;

    let mut data = vec![];
    reader.read_to_end(&mut data)?;

    let bnd = BND4::from_reader(&mut Cursor::new(data))?;

    println!("Compression type: {:?}", dcx.compression_parameters());
    println!("Files: {}", bnd.files.len());

    for idx in 0..bnd.files.len() {
        println!("File[{idx}] {}", bnd.files[idx].path);
    }

    Ok(())
}

pub fn describe_entryfilelist(dvd_bnd: &DvdBnd, name: &str) -> Result<(), Box<dyn Error>> {
    let reader = dvd_bnd.open(name).expect("Could not open dvdbnd entry");
    let container = EntryFileListContainer::from_bytes(reader.data())?;

    let (_unk1s, container) = container.unk1s()?;
    let (_unk2s, container) = container.unk2s()?;
    let strings = container.strings();

    for string in strings.iter() {
        println!(" - Referenced asset: {:?}", string);
    }

    Ok(())
}

pub fn describe_matbin(_dvd_bnd: &DvdBnd, _name: &str) -> Result<(), Box<dyn Error>> {
    todo!()
}
