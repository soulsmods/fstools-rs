use std::{borrow::Cow, io};

use bytemuck::PodCastError;
use byteorder::LE;
use thiserror::Error;
use widestring::{U16Str, U16String};
use zerocopy::{FromBytes, FromZeroes, Ref, F32, U32, U64};

use crate::io_ext::zerocopy::Padding;

#[derive(Debug, Error)]
pub enum MatbinError {
    #[error("Could not copy bytes {0}")]
    Io(#[from] io::Error),

    #[error("Could not read string")]
    String(#[from] ReadUtf16StringError),

    #[error("Got unknown parameter type {0}")]
    UnknownParameterType(u32),

    #[error("Could not create reference to value")]
    UnalignedValue,
}

// Defines a material for instancing in FLVERs and such.
// It does so by pointing at a shader and specifying the parameter/sampler
// setup.
#[allow(unused)]
pub struct Matbin<'buffer> {
    bytes: &'buffer [u8],

    header: &'buffer Header,
    
    parameters: &'buffer [Parameter],

    samplers: &'buffer [Sampler],
}

impl<'buffer> Matbin<'buffer> {
    pub fn parse(bytes: &'buffer [u8]) -> Option<Self> {
        let (header, next) = Ref::<_, Header>::new_from_prefix(bytes)?;
        let (parameters, next) = Parameter::slice_from_prefix(
            next,
            header.parameter_count.get() as usize,
        )?;

        let (samplers, _) = Sampler::slice_from_prefix(
            next,
            header.sampler_count.get() as usize,
        )?;

        Some(Self {
            bytes,
            header: header.into_ref(),
            parameters,
            samplers,
        })
    }

    pub fn shader_path(&self) -> Result<Cow<'_, U16Str>, MatbinError> {
        let offset = self.header.shader_path_offset.get() as usize;
        let bytes = &self.bytes[offset..];

        Ok(read_utf16_string(bytes)?)
    }

    pub fn source_path(&self) -> Result<Cow<'_, U16Str>, MatbinError> {
        let offset = self.header.source_path_offset.get() as usize;
        let bytes = &self.bytes[offset..];

        Ok(read_utf16_string(bytes)?)
    }

    pub fn samplers(
        &self,
    ) -> impl Iterator<Item=Result<SamplerIterElement, MatbinError>> {
        self.samplers.iter()
            .map(|e| {
                let sampler_type = {
                    let offset = e.type_offset.get() as usize;
                    let bytes = &self.bytes[offset..];
                    read_utf16_string(bytes)
                }?;

                let path = {
                    let offset = e.path_offset.get() as usize;
                    let bytes = &self.bytes[offset..];
                    read_utf16_string(bytes)
                }?;

                Ok(SamplerIterElement {
                    sampler_type,
                    path,
                })
            })
    }

    pub fn parameters(
        &self,
    ) -> impl Iterator<Item=Result<ParameterIterElement, MatbinError>> {
        self.parameters.iter()
            .map(|e| {
                let name = {
                    let offset = e.name_offset.get() as usize;
                    let bytes = &self.bytes[offset..];
                    read_utf16_string(bytes)
                }?;

                let value_slice = &self.bytes[e.value_offset.get() as usize..];
                let value = ParameterValue::from_type_and_slice(
                    e.value_type.get(),
                    value_slice,
                )?;

                Ok(ParameterIterElement {
                    name,
                    value,
                })
            })
    }
}

pub struct ParameterIterElement<'buffer> {
    pub name: Cow<'buffer, U16Str>,
    pub value: ParameterValue<'buffer>,
}

pub struct SamplerIterElement<'buffer> {
    pub sampler_type: Cow<'buffer, U16Str>,
    pub path: Cow<'buffer, U16Str>,
}

impl<'buffer> std::fmt::Debug for Matbin<'buffer> {
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

#[derive(Debug)]
pub enum ParameterValue<'buffer> {
    Bool(bool),
    Int(&'buffer U32<LE>),
    IntVec2(&'buffer [U32<LE>]),
    Float(&'buffer F32<LE>),
    FloatVec2(&'buffer [F32<LE>]),
    FloatVec3(&'buffer [F32<LE>]),
    FloatVec4(&'buffer [F32<LE>]),
    FloatVec5(&'buffer [F32<LE>]),
}

impl<'buffer> ParameterValue<'buffer> {
    pub fn from_type_and_slice(
        value_type: u32,
        value_slice: &'buffer [u8],
    ) -> Result<Self, MatbinError> {
        Ok(match value_type {
            0x0 => ParameterValue::Bool(
                value_slice[0] != 0x0
            ),
            0x4 => ParameterValue::Int(
                U32::<LE>::ref_from_prefix(value_slice)
                    .ok_or(MatbinError::UnalignedValue)?,
            ),
            0x5 => ParameterValue::IntVec2(
                U32::<LE>::slice_from_prefix(value_slice, 2)
                    .ok_or(MatbinError::UnalignedValue)?.0,
            ),
            0x8 => ParameterValue::Float(
                F32::<LE>::ref_from_prefix(value_slice)
                    .ok_or(MatbinError::UnalignedValue)?,
            ),
            0x9 => ParameterValue::FloatVec2(
                F32::<LE>::slice_from_prefix(value_slice, 2)
                    .ok_or(MatbinError::UnalignedValue)?.0,
            ),
            0xA => ParameterValue::FloatVec3(
                F32::<LE>::slice_from_prefix(value_slice, 3)
                    .ok_or(MatbinError::UnalignedValue)?.0,
            ),
            0xB => ParameterValue::FloatVec4(
                F32::<LE>::slice_from_prefix(value_slice, 4)
                    .ok_or(MatbinError::UnalignedValue)?.0,
            ),
            0xC => ParameterValue::FloatVec5(
                F32::<LE>::slice_from_prefix(value_slice, 5)
                    .ok_or(MatbinError::UnalignedValue)?.0,
            ),
            _ => {
                return Err(MatbinError::UnknownParameterType(value_type))
            }
        })
    }
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct Header {
    chunk_magic: [u8; 4],

    // Seems to be 2? Might be some version number. Couldn't easily find the
    /// parser with Ghidra so :shrug:.
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
#[repr(packed)]
#[allow(unused)]
pub struct Parameter {
    /// Offset to name of the parameter
    name_offset: U64<LE>,

    /// Offset to value of the parameter
    value_offset: U64<LE>,

    /// Adler32 hash of the name string without the string terminator
    name_hash: U32<LE>,

    /// Type of the value pointed at by value_offset
    value_type: U32<LE>,

    _padding18: Padding<16>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct Sampler {
    // Offset to the samplers type
    type_offset: U64<LE>,

    // Offset to the samplers path
    path_offset: U64<LE>,

    // Adler32 hash of the type string without the string terminator
    type_hash: U32<LE>,

    // ???
    unkxy: [F32<LE>; 2],

    _padding1c: Padding<20>,
}

#[derive(Debug, Error)]
pub enum ReadUtf16StringError {
    #[error("Could not find end of string")]
    NoEndFound,

    #[error("Bytemuck could not cast pod")]
    Bytemuck,
}

fn read_utf16_string(
    input: &[u8]
) -> Result<Cow<'_, U16Str>, ReadUtf16StringError> {
    // Find the end of the input string
    let length = input.chunks_exact(2)
        .position(|bytes| bytes[0] == 0x0 && bytes[1] == 0x0)
        .ok_or(ReadUtf16StringError::NoEndFound)?;

    // Create a view that has a proper end so we don't copy
    // the entire input slice if required and so we don't have to deal with
    // bytemuck freaking out over the end not being aligned
    let string_bytes = &input[..length * 2];

    Ok(match bytemuck::try_cast_slice::<u8, u16>(string_bytes) {
        Ok(s) => Cow::Borrowed(U16Str::from_slice(s)),
        Err(e) => {
            // We should probably return the error if it isn't strictly
            // about the alignment of the input.
            if e != PodCastError::TargetAlignmentGreaterAndInputNotAligned {
                return Err(ReadUtf16StringError::Bytemuck);
            }

            let aligned_copy = string_bytes.chunks(2)
                .map(|a| u16::from_le_bytes([a[0], a[1]]))
                .collect::<Vec<u16>>();

            Cow::Owned(U16String::from_vec(aligned_copy))
        }
    })
}
