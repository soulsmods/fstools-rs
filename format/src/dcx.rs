use std::io;

use byteorder::{ReadBytesExt, BE};

#[derive(Debug)]
pub enum DCXError {
    IO(io::Error),
}

#[derive(Debug)]
pub struct DCX {
    pub unk04: u32,
    pub dcs_offset: u32,
    pub dcp_offset: u32,
    pub unk10: u32,
    pub unk14: u32,
    pub dcs: u32,
    pub uncompressed_size: u32,
    pub compressed_size: u32,
    pub dcp: u32,
    pub format: u32,
    pub unk2c: u32,
    pub compression_level: u8,
    pub unk31: u8,
    pub unk32: u8,
    pub unk33: u8,
    pub unk34: u32,
    pub unk38: u32,
    pub unk3c: u32,
    pub unk40: u32,
    pub dca: u32,
    pub dca_size: u32,
    pub decompressed: Vec<u8>,
}

impl DCX {
    pub fn from_reader(r: &mut impl io::Read) -> Result<Self, io::Error> {
        let magic = r.read_u32::<BE>()?;
        assert!(magic == 0x44435800, "DCX was not of expected format");

        let unk04 = r.read_u32::<BE>()?;
        let dcs_offset = r.read_u32::<BE>()?;
        let dcp_offset = r.read_u32::<BE>()?;
        let unk10 = r.read_u32::<BE>()?;
        let unk14 = r.read_u32::<BE>()?;
        let dcs = r.read_u32::<BE>()?;
        let uncompressed_size = r.read_u32::<BE>()?;
        let compressed_size = r.read_u32::<BE>()?;
        let dcp = r.read_u32::<BE>()?;
        let format = r.read_u32::<BE>()?;
        assert!(format == 0x4b52414b, "Format was not KRAKEN");

        let unk2c = r.read_u32::<BE>()?;
        let compression_level = r.read_u8()?;
        let unk31 = r.read_u8()?;
        let unk32 = r.read_u8()?;
        let unk33 = r.read_u8()?;
        let unk34 = r.read_u32::<BE>()?;
        let unk38 = r.read_u32::<BE>()?;
        let unk3c = r.read_u32::<BE>()?;
        let unk40 = r.read_u32::<BE>()?;
        let dca = r.read_u32::<BE>()?;
        let dca_size = r.read_u32::<BE>()?;

        let mut compressed = vec![0x0u8; compressed_size as usize];
        r.read_exact(&mut compressed)?;

        let mut decompressed = vec![0x0u8; uncompressed_size as usize];
        let result = oodle_safe::decompress(
            &compressed,
            &mut decompressed,
            None,
            None,
            None,
            None,
        );

        if result.is_err() {
            panic!("Oodle decompress failed");
        }

        // std::fs::write("./test.dcx.decompress", decompressed)?;

        Ok(Self {
            unk04,
            dcs_offset,
            dcp_offset,
            unk10,
            unk14,
            dcs,
            uncompressed_size,
            compressed_size,
            dcp,
            format,
            unk2c,
            compression_level,
            unk31,
            unk32,
            unk33,
            unk34,
            unk38,
            unk3c,
            unk40,
            dca,
            dca_size,
            decompressed,
        })
    }
}
