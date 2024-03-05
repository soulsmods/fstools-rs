use std::{io, io::Read};

use byteorder::BE;
use thiserror::Error;
use zerocopy::{AsBytes, FromBytes, FromZeroes, Ref, U32};

use self::{deflate::DcxDecoderDeflate, kraken::DcxDecoderKraken};

pub mod deflate;
pub mod kraken;

const MAGIC_DCX: u32 = 0x44435800;
const MAGIC_ALGORITHM_KRAKEN: &[u8; 4] = b"KRAK";
const MAGIC_ALGORITHM_DEFLATE: &[u8; 4] = b"DFLT";

#[derive(Debug, Error)]
pub enum DcxError {
    #[error("Could not copy bytes {0}")]
    Io(#[from] io::Error),

    #[error("Unrecognized DCX compression algorithm: {0:x?}")]
    UnknownAlgorithm([u8; 4]),

    #[error("Could not properly parse DCX file")]
    ParserError,
}

#[derive(Debug, Error)]
pub enum DecompressionError {
    #[error("Oodle error code: {0}")]
    Oodle(u32),

    #[error("Zlib error")]
    Zlib,
}

#[derive(FromZeroes, FromBytes)]
#[repr(C)]
#[allow(unused)]
pub struct Dcx<'a> {
    bytes: &'a [u8],

    header: &'a Header,
    sizes: &'a Sizes,
    compression_parameters: &'a CompressionParameters,
    compressed: &'a [u8],
}

impl<'a> Dcx<'a> {
    // TODO: add magic validation
    pub fn parse(bytes: &'a [u8]) -> Option<Self> {
        let (header, next) = Ref::<_, Header>::new_from_prefix(bytes)?;
        let (sizes, next) = Ref::<_, Sizes>::new_from_prefix(next)?;
        let (compression_parameters, next) =
            Ref::<_, CompressionParameters>::new_from_prefix(next)?;
        let (additional, rest) = Ref::<_, Additional>::new_from_prefix(next)?;

        Some(Self {
            bytes,
            header: header.into_ref(),
            sizes: sizes.into_ref(),
            compression_parameters: compression_parameters.into_ref(),
            compressed: rest,
        })
    }

    pub fn create_decoder(&self) -> Result<DcxContentDecoder, DcxError> {
        let algorithm = &self.compression_parameters.algorithm;
        let decoder = match algorithm {
            MAGIC_ALGORITHM_KRAKEN => Decoder::Kraken(DcxDecoderKraken::from_buffer(
                self.compressed,
                self.sizes.uncompressed_size,
            )),
            MAGIC_ALGORITHM_DEFLATE => {
                Decoder::Deflate(DcxDecoderDeflate::from_buffer(self.compressed))
            }
            _ => return Err(DcxError::UnknownAlgorithm(algorithm.to_owned())),
        };

        Ok(DcxContentDecoder {
            uncompressed_size: self.sizes.uncompressed_size,
            compressed: &self.compressed,
            decoder,
        })
    }

    pub fn has_magic(buf: &[u8]) -> bool {
        match U32::<BE>::ref_from_prefix(buf) {
            Some(v) => v.get() == MAGIC_DCX,
            None => false,
        }
    }
}

impl<'a> std::fmt::Debug for Dcx<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DCX")
            .field("header", self.header)
            .field("sizes", self.sizes)
            .field("compression_parameters", self.compression_parameters)
            .finish()
    }
}

pub enum Decoder<'a> {
    Kraken(DcxDecoderKraken<'a>),
    Deflate(DcxDecoderDeflate<'a>),
}

pub struct DcxContentDecoder<'a> {
    /// Reference to the compressed bytes.
    compressed: &'a [u8],

    /// Size of the contents once decompressed.
    uncompressed_size: U32<BE>,

    decoder: Decoder<'a>,
}

impl<'a> DcxContentDecoder<'a> {
    pub fn hint_size(&self) -> usize {
        self.uncompressed_size.get() as usize
    }
}

impl<'a> Read for DcxContentDecoder<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match &mut self.decoder {
            Decoder::Kraken(d) => d.read(buf),
            Decoder::Deflate(d) => d.read(buf),
        }
    }
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C)]
#[allow(unused)]
/// The DCX chunk. Describes the layout of the container.
struct Header {
    chunk_magic: [u8; 4],

    /// Overal Dcx file version
    version: U32<BE>,

    /// Offset to the DCS chunk
    sizes_offset: U32<BE>,

    /// Offset to the DCP chunk
    params_offset: U32<BE>,

    /// Offset to the DCA chunk
    data_info_offset: U32<BE>,

    /// Offset to the compressed data
    data_offset: U32<BE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C)]
#[allow(unused)]
/// The DCS Chunk. Describes the sizes before and after compression.
struct Sizes {
    chunk_magic: [u8; 4],
    /// Size of the data when decompressed, can be used for reserving vector
    /// capacity.
    uncompressed_size: U32<BE>,

    /// Size of the data in its compressed form.
    compressed_size: U32<BE>,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C)]
#[allow(unused)]
/// The DCP chunk. Describes parameters used for compression/decompression.
struct CompressionParameters {
    chunk_magic: [u8; 4],

    /// Either KRAK, DFLT or EDGE
    algorithm: [u8; 4],

    /// Seems to the size of the current DCP chunk including magic and algo
    chunk_size: U32<BE>,

    /// Arbitrary bytes describing the parameter chunk
    /// TODO make this of dynamic size using the chunksize
    settings: [u8; 20],
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C)]
#[allow(unused)]
/// The DCA chunk. Describes ???
struct Additional {
    chunk_magic: [u8; 4],

    /// Seems to the size of the current DCA chunk including magic
    chunk_size: U32<BE>,
}
