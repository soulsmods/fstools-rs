use std::{
    fmt::{Debug, Formatter},
    io::{Error, Read},
    mem::size_of,
};

use byteorder::BE;
use thiserror::Error;
use zerocopy::{FromBytes, FromZeroes, U32};
use zstd::ZstdDecoder;

use self::{deflate::DeflateDecoder, oodle::OodleReader};

pub mod deflate;
pub mod oodle;
pub mod zstd;

const MAGIC_DCX: u32 = 0x44435800;
const MAGIC_ALGORITHM_KRAKEN: &[u8; 4] = b"KRAK";
const MAGIC_ALGORITHM_DEFLATE: &[u8; 4] = b"DFLT";
const MAGIC_ALGORITHM_ZSTD: &[u8; 4] = b"ZSTD";

#[derive(Debug, Error)]
pub enum DcxError {
    #[error("Could not copy bytes {0}")]
    Io(#[from] std::io::Error),

    #[error("Unrecognized DCX compression algorithm: {0:x?}")]
    UnknownAlgorithm([u8; 4]),

    #[error("Could not properly parse DCX file")]
    ParserError,

    #[error("Unable to create compression codec for DCX contents")]
    DecoderError,
}

#[derive(Debug, Error)]
pub enum DecompressionError {
    #[error("Oodle error code: {0}")]
    Oodle(u32),

    #[error("Zlib error")]
    Zlib,
}

#[derive(FromBytes, FromZeroes)]
#[repr(C, packed)]
pub struct DcxHeader {
    metadata: Metadata,
    sizes: Sizes,
    compression_parameters: CompressionParameters,
    _additional: Additional,
}

impl DcxHeader {
    pub fn read<R: Read>(mut reader: R) -> Result<(DcxHeader, DcxContentDecoder<R>), DcxError> {
        let mut header_data = [0u8; size_of::<DcxHeader>()];
        reader.read_exact(&mut header_data)?;

        let dcx = DcxHeader::read_from(&header_data).ok_or(Error::other("unaligned DCX header"))?;
        let decoder = dcx.create_decoder(reader)?;

        Ok((dcx, decoder))
    }

    pub fn create_decoder<R: Read>(&self, reader: R) -> Result<DcxContentDecoder<R>, DcxError> {
        let algorithm = &self.compression_parameters.algorithm;
        let decoder = match algorithm {
            MAGIC_ALGORITHM_KRAKEN => Decoder::Kraken(
                OodleReader::new(reader, self.sizes.uncompressed_size.get())
                    .ok_or(DcxError::DecoderError)?,
            ),
            MAGIC_ALGORITHM_DEFLATE => Decoder::Deflate(DeflateDecoder::new(reader)),
            MAGIC_ALGORITHM_ZSTD => {
                Decoder::Zstd(ZstdDecoder::new(reader).map_err(|_| DcxError::DecoderError)?)
            }
            _ => return Err(DcxError::UnknownAlgorithm(algorithm.to_owned())),
        };

        Ok(DcxContentDecoder {
            uncompressed_size: self.sizes.uncompressed_size,
            decoder,
        })
    }

    pub fn has_magic(buf: &[u8]) -> bool {
        match U32::<BE>::ref_from_prefix(buf) {
            Some(v) => v.get() == MAGIC_DCX,
            None => false,
        }
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn sizes(&self) -> &Sizes {
        &self.sizes
    }

    pub fn compression_parameters(&self) -> &CompressionParameters {
        &self.compression_parameters
    }
}

impl Debug for DcxHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DCX")
            .field("header", &self.metadata)
            .field("sizes", &self.sizes)
            .field("compression_parameters", &self.compression_parameters)
            .finish()
    }
}

pub enum Decoder<R: Read> {
    Kraken(OodleReader<R>),
    Deflate(DeflateDecoder<R>),
    Zstd(ZstdDecoder<R>),
}

pub struct DcxContentDecoder<R: Read> {
    /// Size of the contents once decompressed.
    uncompressed_size: U32<BE>,

    decoder: Decoder<R>,
}

impl<R: Read> DcxContentDecoder<R> {
    pub fn hint_size(&self) -> usize {
        self.uncompressed_size.get() as usize
    }
}

impl<R: Read> Read for DcxContentDecoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &mut self.decoder {
            Decoder::Kraken(d) => d.read(buf),
            Decoder::Deflate(d) => d.read(buf),
            Decoder::Zstd(d) => d.read(buf),
        }
    }
}

#[derive(FromZeroes, FromBytes)]
#[repr(C)]
#[allow(unused)]
/// The DCX chunk. Describes the layout of the container.
pub struct Metadata {
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

impl Debug for Metadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Metadata")
            .field("version", &self.version.get())
            .finish()
    }
}

#[derive(FromZeroes, FromBytes)]
#[repr(C)]
#[allow(unused)]
/// The DCS Chunk. Describes the sizes before and after compression.
pub struct Sizes {
    chunk_magic: [u8; 4],
    /// Size of the data when decompressed, can be used for reserving vector
    /// capacity.
    uncompressed_size: U32<BE>,

    /// Size of the data in its compressed form.
    compressed_size: U32<BE>,
}

impl Debug for Sizes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sizes")
            .field("uncompressed_size", &self.uncompressed_size.get())
            .field("compressed_size", &self.compressed_size.get())
            .finish()
    }
}

#[derive(FromZeroes, FromBytes)]
#[repr(C, packed)]
#[allow(unused)]
/// The DCP chunk. Describes parameters used for compression/decompression.
pub struct CompressionParameters {
    chunk_magic: [u8; 4],

    /// Either KRAK, DFLT, EDGE or ZSTD
    algorithm: [u8; 4],

    /// Seems to the size of the current DCP chunk including magic and algo
    chunk_size: U32<BE>,

    /// Arbitrary bytes describing the parameter chunk
    /// TODO make this of dynamic size using the chunksize
    settings: [u8; 20],
}

impl Debug for CompressionParameters {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let algorithm_name = String::from_utf8_lossy(&self.algorithm);

        f.debug_struct("CompressionParameters")
            .field("algorithm", &algorithm_name)
            .finish()
    }
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
