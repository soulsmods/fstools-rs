use std::{
    error::Error,
    io::{Cursor, Read},
};

use fstools_dvdbnd::DvdBnd;
use fstools_formats::{bnd4::BND4, dcx::DcxHeader};

pub fn describe_bnd(dvd_bnd: DvdBnd, name: &str) -> Result<(), Box<dyn Error>> {
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

pub fn describe_matbin(_dvd_bnd: DvdBnd, _name: &str) -> Result<(), Box<dyn Error>> {
    todo!()
}
