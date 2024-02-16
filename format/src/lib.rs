#![feature(adt_const_params)]
#![feature(generic_const_exprs)]

use std::io;

use byteorder::{ReadBytesExt, LE};

pub mod bhd;
pub mod dcx;
pub mod bnd4;
pub mod tpf;
pub mod flver;
pub mod matbin;

pub fn read_utf16(r: &mut impl io::Read) -> Result<String, io::Error> {
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
