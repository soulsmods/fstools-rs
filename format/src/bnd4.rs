use std::io::{self, Read, Seek, SeekFrom};
use byteorder::{ReadBytesExt, LE};

type BND4Reader = std::io::Cursor<Vec<u8>>;

#[derive(Debug)]
pub struct BND4 {
    pub unk04: u8,
    pub unk05: u8,
    pub unk0a: u8,
    pub file_count: u32,
    pub file_headers_offset: u64,
    pub version: u64,
    pub file_header_size: u64,
    pub file_headers_end: u64,
    pub unicode: bool,
    pub raw_format: u8,
    pub extended: u8,
    pub buckets_offset: u64,
    pub files: Vec<File>,
    pub data: Vec<u8>,
}

impl BND4 {
    pub fn from_reader(r: &mut BND4Reader) -> Result<Self, io::Error> {
        let magic = r.read_u32::<LE>()?;
        assert!(magic == 0x34444e42, "BND4 was not of expected format");

        let unk04 = r.read_u8()?;
        let unk05 = r.read_u8()?;
        assert!(r.read_u8()? == 0x0);
        assert!(r.read_u8()? == 0x0);
        assert!(r.read_u8()? == 0x0);

        assert!(r.read_u8()? == 0x0, "BND4 is not little endian");

        let unk0a = r.read_u8()?;
        assert!(r.read_u8()? == 0x0);
        let file_count = r.read_u32::<LE>()?;

        let file_headers_offset = r.read_u64::<LE>()?;
        let version = r.read_u64::<LE>()?;
        let file_header_size = r.read_u64::<LE>()?;
        let file_headers_end = r.read_u64::<LE>()?;
        let unicode = r.read_u8()? == 0x1;
        let raw_format = r.read_u8()?;
        let extended = r.read_u8()?;

        assert!(r.read_u32::<LE>()? == 0x0);
        assert!(r.read_u8()? == 0x0);

        let buckets_offset = r.read_u64::<LE>()?;

        let mut files = vec![];
        for _ in 0..file_count {
            files.push(File::from_reader(r)?);
        }

        let mut data = vec![];
        r.seek(SeekFrom::Start(0))?;
        r.read_to_end(&mut data)?;

        Ok(Self {
            unk04,
            unk05,
            unk0a,
            file_count,
            file_headers_offset,
            version,
            file_header_size,
            file_headers_end,
            unicode,
            raw_format,
            extended,
            buckets_offset,
            files,
            data,
        })
    }

    pub fn file_descriptor_by_stem(&self, path: &str) -> Option<&File> {
        let lookup = std::path::PathBuf::from(Self::normalize_path(path));

        self.files.iter()
            .find(|f| {
                let path = std::path::PathBuf::from(
                    Self::normalize_path(&f.path)
                );

                path.file_stem() == lookup.file_stem()
            })
    }

    fn normalize_path(path: &str) -> String {
        path.replace("N:\\", "")
            .to_lowercase()
            .replace('\\', "/")
    }
}

#[derive(Debug, PartialEq)]
pub struct File {
    pub flags: u8,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub data_offset: u32,
    pub id: u32,
    pub path: String,
}

impl File {
    pub fn from_reader(r: &mut BND4Reader) -> Result<Self, io::Error> {
        let flags = r.read_u8()?;
        assert!(r.read_u8()? == 0x0);
        assert!(r.read_u8()? == 0x0);
        assert!(r.read_u8()? == 0x0);
        assert!(r.read_u8()? == 0xFF);
        assert!(r.read_u8()? == 0xFF);
        assert!(r.read_u8()? == 0xFF);
        assert!(r.read_u8()? == 0xFF);

        let compressed_size = r.read_u64::<LE>()?;
        let uncompressed_size = r.read_u64::<LE>()?;
        let data_offset = r.read_u32::<LE>()?;
        let id = r.read_u32::<LE>()?;
        let name_offset = r.read_u32::<LE>()?;

        let current = r.stream_position()?;
        r.seek(SeekFrom::Start(name_offset as u64))?;
        let path = read_utf16(r)?;
        r.seek(SeekFrom::Start(current))?;

        assert!(
            compressed_size == uncompressed_size,
            "BND4 entry compression detected"
        );

        Ok(Self {
            flags,
            compressed_size,
            uncompressed_size,
            data_offset,
            id,
            path,
        })
    }

    pub fn bytes(&self, r: &mut BND4Reader) -> Result<Vec<u8>, io::Error> {
        let mut buffer = vec![0x0u8; self.compressed_size as usize];
        r.seek(SeekFrom::Start(self.data_offset as u64))?;
        r.read_exact(&mut buffer)?;

        Ok(buffer)
    }
}

pub trait FromBnd4File {
    fn from_bnd4(bytes: &[u8]) -> Self;
}

fn read_utf16(r: &mut BND4Reader) -> Result<String, io::Error> {
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
