use std::io::{self, SeekFrom};

use byteorder::{ReadBytesExt, LE};

use crate::io_ext::ReadFormatsExt;

#[derive(Debug)]
pub enum TPFError {
    IO(io::Error),
}

#[derive(Debug)]
pub struct TPF {
    pub textures: Vec<Texture>,
}

impl TPF {
    pub fn from_reader(r: &mut (impl io::Read + io::Seek)) -> Result<Self, io::Error> {
        r.read_magic(b"TPF\0")?;

        let _data_size = r.read_u32::<LE>()?;
        let texture_count = r.read_u32::<LE>()?;
        let _platform = r.read_u8()?;
        let _unk0d = r.read_u8()?;
        assert!(r.read_u8()? == 0x1, "Encoding isn't 0x1");
        r.read_padding(1)?;

        let mut textures = vec![];
        for _ in 0..texture_count {
            textures.push(Texture::from_reader(r)?);
        }

        Ok(Self { textures })
    }
}

#[derive(Debug)]
pub struct Texture {
    pub data_offset: u32,
    pub data_size: u32,
    pub format: u8,
    pub cubemap: u8,
    pub mipmaps: u8,
    pub name: String,
}

impl Texture {
    pub fn from_reader(r: &mut (impl io::Read + io::Seek)) -> Result<Self, io::Error> {
        let data_offset = r.read_u32::<LE>()?;
        let data_size = r.read_u32::<LE>()?;
        let format = r.read_u8()?;
        let cubemap = r.read_u8()?;
        let mipmaps = r.read_u8()?;
        let _unk0b = r.read_u8()?;
        let name_offset = r.read_u32::<LE>()?;
        let _unk10 = r.read_u32::<LE>()?;

        let current = r.stream_position()?;
        r.seek(SeekFrom::Start(name_offset as u64))?;
        let name = r.read_utf16::<LE>()?;
        r.seek(SeekFrom::Start(current))?;

        Ok(Self {
            data_offset,
            data_size,
            format,
            cubemap,
            mipmaps,
            name,
        })
    }

    pub fn bytes(&self, r: &mut (impl io::Read + io::Seek)) -> Result<Vec<u8>, io::Error> {
        let mut buffer = vec![0x0u8; self.data_size as usize];
        r.seek(SeekFrom::Start(self.data_offset as u64))?;
        r.read_exact(&mut buffer)?;
        Ok(buffer)
    }
}
