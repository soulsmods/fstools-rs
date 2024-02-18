#![feature(adt_const_params)]
#![feature(generic_const_exprs)]

use std::io::{Error, Read};

use byteorder::{ReadBytesExt, LE};

pub mod bhd;
pub mod bhd2;
pub mod bnd4;
pub mod dcx;
pub mod flver;
pub mod io_ext;
pub mod matbin;
pub mod tpf;

pub fn read_utf16(r: &mut impl Read) -> Result<String, Error> {
    let mut buffer = Vec::new();

    loop {
        let current = r.read_u16::<LE>()?;
        if current != 0x0 {
            buffer.push(current);
        } else {
            break;
        }
    }

    Ok(String::from_utf16(buffer.as_slice()).unwrap())
}
