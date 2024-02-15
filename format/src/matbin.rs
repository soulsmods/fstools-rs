use crate::read_utf16;
use std::io::{self, SeekFrom};
use byteorder::{ReadBytesExt, LE};

#[derive(Debug)]
pub enum MatbinError {
    IO(io::Error),
}

#[derive(Debug)]
pub struct Matbin {
    pub unk04: u32,
    pub shader_path: String,
    pub source_path: String,
    pub key: u32,
    pub param_count: u32,
    pub sampler_count: u32,
    pub params: Vec<MatbinParam>,
    pub samplers: Vec<MatbinSampler>,
}

impl Matbin {
    pub fn from_reader(r: &mut (impl io::Read + io::Seek)) -> Result<Self, io::Error> {
        let magic = r.read_u32::<LE>()?;
        //assert!(magic == 0x42414d, "Matbin was not of expected format");

        let unk04 = r.read_u32::<LE>()?;
        let shader_path_offset = r.read_u64::<LE>()?;
        let source_path_offset = r.read_u64::<LE>()?;
        let key = r.read_u32::<LE>()?;
        let param_count = r.read_u32::<LE>()?;
        let sampler_count = r.read_u32::<LE>()?;

        let current_pos = r.seek(SeekFrom::Current(0))?;
        r.seek(SeekFrom::Start(shader_path_offset as u64))?;
        let shader_path = read_utf16(r)?;
        r.seek(SeekFrom::Start(source_path_offset as u64))?;
        let source_path = read_utf16(r)?;
        r.seek(SeekFrom::Start(current_pos))?;

        assert!(r.read_u64::<LE>()? == 0x0);
        assert!(r.read_u64::<LE>()? == 0x0);
        assert!(r.read_u32::<LE>()? == 0x0);

        let mut params = vec![];
        for _ in 0..param_count {
            params.push(MatbinParam::from_reader(r)?);
        }

        let mut samplers = vec![];
        for _ in 0..sampler_count {
            samplers.push(MatbinSampler::from_reader(r)?);
        }

        Ok(Self {
            unk04,
            shader_path,
            source_path,
            key,
            param_count,
            sampler_count,
            params,
            samplers,
        })
    }
}

#[derive(Debug)]
pub struct MatbinParam {
    pub name: String,
    pub value: u32,
    pub key: u32,
    pub value_type: u32,
}

impl MatbinParam {
    pub fn from_reader(r: &mut (impl io::Read + io::Seek)) -> Result<Self, io::Error> {
        let name_offset = r.read_u64::<LE>()?;
        // TOOD: read value
        let value_offset = r.read_u64::<LE>()?;
        let key = r.read_u32::<LE>()?;
        let value_type = r.read_u32::<LE>()?;

        assert!(r.read_u64::<LE>()? == 0x0);
        assert!(r.read_u64::<LE>()? == 0x0);

        let current_pos = r.seek(SeekFrom::Current(0))?;
        r.seek(SeekFrom::Start(name_offset as u64))?;
        let name = read_utf16(r)?;
        r.seek(SeekFrom::Start(current_pos))?;

        Ok(Self {
            name,
            value: 0x0,
            key,
            value_type,
        })
    }
}

#[derive(Debug)]
pub struct MatbinSampler {
    pub sampler_type: String,
    pub path: String,
    pub key: u32,
    pub unkx: f32,
    pub unky: f32,
}

impl MatbinSampler {
    pub fn from_reader(r: &mut (impl io::Read + io::Seek)) -> Result<Self, io::Error> {
        let type_offset = r.read_u64::<LE>()?;
        let path_offset = r.read_u64::<LE>()?;
        let key = r.read_u32::<LE>()?;

        let unkx = r.read_f32::<LE>()?;
        let unky = r.read_f32::<LE>()?;

        assert!(r.read_u64::<LE>()? == 0x0);
        assert!(r.read_u64::<LE>()? == 0x0);
        assert!(r.read_u32::<LE>()? == 0x0);

        let current_pos = r.seek(SeekFrom::Current(0))?;
        r.seek(SeekFrom::Start(type_offset as u64))?;
        let sampler_type = read_utf16(r)?;
        r.seek(SeekFrom::Start(path_offset as u64))?;
        let path = read_utf16(r)?;
        r.seek(SeekFrom::Start(current_pos))?;

        Ok(Self {
            sampler_type,
            path,
            key,
            unkx,
            unky,
        })
    }
}
