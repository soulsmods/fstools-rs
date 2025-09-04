use std::{
    io::{self, Read},
    path::Path,
};

use aes::cipher::{BlockDecryptMut, KeyIvInit};
use fstools::formats::{bnd4::BND4, dcx::DcxHeader};

pub fn decrypt_regulation(reader: &mut impl Read) -> io::Result<Vec<u8>> {
    const REGULATION_KEY: &[u8; 32] = &[
        0x99, 0xBF, 0xFC, 0x36, 0x6A, 0x6B, 0xC8, 0xC6, 0xF5, 0x82, 0x7D, 0x09, 0x36, 0x02, 0xD6,
        0x76, 0xC4, 0x28, 0x92, 0xA0, 0x1C, 0x20, 0x7F, 0xB0, 0x24, 0xD3, 0xAF, 0x4E, 0x49, 0x3F,
        0xEF, 0x99,
    ];

    let mut iv = [0u8; 16];
    reader.read_exact(&mut iv)?;

    let mut out_buf = Vec::new();
    reader.read_to_end(&mut out_buf)?;

    type Aes256Cbc = cbc::Decryptor<aes::Aes256>;
    let mut cipher = Aes256Cbc::new_from_slices(REGULATION_KEY, &iv).unwrap();

    // SAFETY: GenericArray<u8, _> is safe to transmute from an equiv. slice of u8s
    unsafe {
        cipher.decrypt_blocks_mut(out_buf.align_to_mut().1);
    }

    Ok(out_buf)
}

pub fn load_regulation(game_path: impl AsRef<Path>) -> io::Result<BND4> {
    let regulation_bytes = std::fs::read(game_path.as_ref().join("regulation.bin"))?;
    let dcx_bytes = decrypt_regulation(&mut regulation_bytes.as_slice())?;

    let (_, mut dcx_decoder) = DcxHeader::read(io::Cursor::new(dcx_bytes))
        .map_err(|_| io::Error::other("DCX header reading failed"))?;

    let mut bnd4_bytes = Vec::new();
    dcx_decoder.read_to_end(&mut bnd4_bytes)?;

    BND4::from_reader(io::Cursor::new(bnd4_bytes))
        .map_err(|_| io::Error::other("Failed to read regulation BND4"))
}
