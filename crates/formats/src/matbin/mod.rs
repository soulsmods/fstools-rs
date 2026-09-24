use std::io;

use byteorder::LE;
use thiserror::Error;
use utf16string::WStr;
use zerocopy::{FromBytes, FromZeroes, Ref, F32, U32, U64};

use crate::io_ext::{read_wide_cstring, zerocopy::Padding, ReadWidestringError};

#[derive(Debug, Error)]
pub enum MatbinError {
    #[error("Could not copy bytes {0}")]
    Io(#[from] io::Error),

    #[error("Could not read string")]
    String(#[from] ReadWidestringError),

    #[error("Got unknown parameter type {0}")]
    UnknownParameterType(u32),

    #[error("Could not create reference to value")]
    UnalignedValue,
}

// Defines a material for instancing in FLVERs and such.
// It does so by pointing at a shader and specifying the parameter/sampler
// setup.
#[allow(unused)]
pub struct Matbin<'a> {
    bytes: &'a [u8],

    header: &'a Header,

    parameters: &'a [Parameter],

    samplers: &'a [Sampler],
}

impl<'a> Matbin<'a> {
    pub fn parse(bytes: &'a [u8]) -> Option<Self> {
        let (header, next) = Ref::<_, Header>::new_from_prefix(bytes)?;
        let (parameters, next) =
            Parameter::slice_from_prefix(next, header.parameter_count.get() as usize)?;

        let (samplers, _) = Sampler::slice_from_prefix(next, header.sampler_count.get() as usize)?;

        Some(Self {
            bytes,
            header: header.into_ref(),
            parameters,
            samplers,
        })
    }

    pub fn shader_path(&self) -> Result<&'_ WStr<LE>, MatbinError> {
        let offset = self.header.shader_path_offset.get() as usize;
        let bytes = &self.bytes[offset..];

        Ok(read_wide_cstring(bytes)?)
    }

    pub fn source_path(&self) -> Result<&'_ WStr<LE>, MatbinError> {
        let offset = self.header.source_path_offset.get() as usize;
        let bytes = &self.bytes[offset..];

        Ok(read_wide_cstring(bytes)?)
    }

    pub fn samplers(&self) -> impl Iterator<Item = Result<SamplerIterElement<'_>, MatbinError>> {
        self.samplers.iter().map(|e| {
            let name = {
                let offset = e.name_offset.get() as usize;
                let bytes = &self.bytes[offset..];
                read_wide_cstring(bytes)
            }?;

            let path = {
                let offset = e.path_offset.get() as usize;
                let bytes = &self.bytes[offset..];
                read_wide_cstring(bytes)
            }?;

            Ok(SamplerIterElement { name, path })
        })
    }

    pub fn parameters(
        &self,
    ) -> impl Iterator<Item = Result<ParameterIterElement<'_>, MatbinError>> {
        self.parameters.iter().map(|e| {
            let name = {
                let offset = e.name_offset.get() as usize;
                let bytes = &self.bytes[offset..];
                read_wide_cstring(bytes)
            }?;

            let value_slice = &self.bytes[e.value_offset.get() as usize..];
            let value = ParameterValue::from_type_and_slice(e.value_type.get(), value_slice)?;

            Ok(ParameterIterElement { name, value })
        })
    }
}

impl<'a> std::fmt::Debug for Matbin<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Matbin")
            .field("shader_path", &self.shader_path())
            .field("source_path", &self.source_path())
            .field("header", self.header)
            .field("parameters", &self.parameters)
            .field("samplers", &self.samplers)
            .finish()
    }
}

pub struct ParameterIterElement<'a> {
    pub name: &'a WStr<LE>,
    pub value: ParameterValue<'a>,
}

pub struct SamplerIterElement<'a> {
    pub name: &'a WStr<LE>,
    pub path: &'a WStr<LE>,
}

pub enum ParameterValue<'a> {
    Bool(bool),
    Int(&'a U32<LE>),
    IntVec2(&'a [U32<LE>]),
    Float(&'a F32<LE>),
    FloatVec2(&'a [F32<LE>]),
    FloatVec3(&'a [F32<LE>]),
    FloatVec4(&'a [F32<LE>]),
    FloatVec5(&'a [F32<LE>]),
}

impl<'a> ParameterValue<'a> {
    pub fn from_type_and_slice(
        value_type: u32,
        value_slice: &'a [u8],
    ) -> Result<Self, MatbinError> {
        Ok(match value_type {
            0x0 => ParameterValue::Bool(value_slice[0] != 0x0),
            0x4 => ParameterValue::Int(
                U32::<LE>::ref_from_prefix(value_slice).ok_or(MatbinError::UnalignedValue)?,
            ),
            0x5 => ParameterValue::IntVec2(
                U32::<LE>::slice_from_prefix(value_slice, 2)
                    .ok_or(MatbinError::UnalignedValue)?
                    .0,
            ),
            0x8 => ParameterValue::Float(
                F32::<LE>::ref_from_prefix(value_slice).ok_or(MatbinError::UnalignedValue)?,
            ),
            0x9 => ParameterValue::FloatVec2(
                F32::<LE>::slice_from_prefix(value_slice, 2)
                    .ok_or(MatbinError::UnalignedValue)?
                    .0,
            ),
            0xA => ParameterValue::FloatVec3(
                F32::<LE>::slice_from_prefix(value_slice, 3)
                    .ok_or(MatbinError::UnalignedValue)?
                    .0,
            ),
            0xB => ParameterValue::FloatVec4(
                F32::<LE>::slice_from_prefix(value_slice, 4)
                    .ok_or(MatbinError::UnalignedValue)?
                    .0,
            ),
            0xC => ParameterValue::FloatVec5(
                F32::<LE>::slice_from_prefix(value_slice, 5)
                    .ok_or(MatbinError::UnalignedValue)?
                    .0,
            ),
            _ => return Err(MatbinError::UnknownParameterType(value_type)),
        })
    }
}

impl<'a> std::fmt::Debug for ParameterValue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            ParameterValue::Bool(v) => format!("Bool({})", v),
            ParameterValue::Int(v) => format!("Int({})", v.get()),
            ParameterValue::IntVec2(v) => format!("IntVec2([{}, {}])", v[0].get(), v[1].get(),),
            ParameterValue::Float(v) => format!("Float({})", v.get()),
            ParameterValue::FloatVec2(v) => format!("FloatVec2([{}, {}])", v[0].get(), v[1].get(),),
            ParameterValue::FloatVec3(v) => format!(
                "FloatVec3([{}, {}, {}])",
                v[0].get(),
                v[1].get(),
                v[2].get(),
            ),
            ParameterValue::FloatVec4(v) => format!(
                "FloatVec4([{}, {}, {}, {}])",
                v[0].get(),
                v[1].get(),
                v[2].get(),
                v[3].get(),
            ),
            ParameterValue::FloatVec5(v) => format!(
                "FloatVec5([{}, {}, {}, {}, {}])",
                v[0].get(),
                v[1].get(),
                v[2].get(),
                v[3].get(),
                v[4].get(),
            ),
        })
    }
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct Header {
    chunk_magic: [u8; 4],

    // Seems to be 2? Might be some version number. Couldn't easily find the
    // parser with Ghidra so :shrug:.
    unk04: U32<LE>,

    /// Offset to the shader path
    shader_path_offset: U64<LE>,

    /// Offset to the source path as a wstring. Seems to reference the source
    /// for the current matbin file.
    source_path_offset: U64<LE>,

    /// Adler32 hash of the source path string without the string terminator
    source_path_hash: U32<LE>,

    /// Amount of parameters for this material
    parameter_count: U32<LE>,

    /// Amount of samples for this material
    sampler_count: U32<LE>,

    _padding24: Padding<20>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct Parameter {
    /// Offset to name of the parameter
    name_offset: U64<LE>,

    /// Offset to value of the parameter
    value_offset: U64<LE>,

    /// Adler32 hash of the name string without the string terminator
    name_hash: U32<LE>,

    /// Type of the value pointed at by `value_offset`
    value_type: U32<LE>,

    _padding18: Padding<16>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C, packed)]
#[allow(unused)]
pub struct Sampler {
    /// Offset to the samplers name
    name_offset: U64<LE>,

    /// Offset to the samplers path
    path_offset: U64<LE>,

    /// Adler32 hash of the name string without the string terminator
    name_hash: U32<LE>,

    /// ???
    unkxy: [F32<LE>; 2],

    _padding1c: Padding<20>,
}
