use core::ffi;
use std::{
    ffi::c_void,
    io,
    io::{ErrorKind, Read},
    mem,
    mem::size_of,
};

use byteorder::{ReadBytesExt, BE};
use flate2::{read::ZlibDecoder, Compression};
use oodle_safe::{CheckCRC, DecodeThreadPhase};
use oodle_sys::{
    OodleLZDecoder, OodleLZDecoder_Create, OodleLZDecoder_DecodeSome, OodleLZDecoder_Destroy,
    OodleLZ_CheckCRC_OodleLZ_CheckCRC_No, OodleLZ_CompressOptions_GetDefault,
    OodleLZ_CompressionLevel, OodleLZ_CompressionLevel_OodleLZ_CompressionLevel_Max,
    OodleLZ_Compressor, OodleLZ_Compressor_OodleLZ_Compressor_Kraken, OodleLZ_DecodeSome_Out,
    OodleLZ_Decode_ThreadPhase_OodleLZ_Decode_ThreadPhaseAll,
    OodleLZ_FuzzSafe_OodleLZ_FuzzSafe_Yes, OodleLZ_GetCompressedBufferSizeNeeded,
    OodleLZ_Verbosity_OodleLZ_Verbosity_None, OODLELZ_BLOCK_LEN,
};
use thiserror::Error;
use zerocopy::{AsBytes, FromBytes, FromZeroes, Ref, U32};

use crate::io_ext::ReadFormatsExt;

#[derive(Debug, Error)]
pub enum DCXError {
    #[error("Could not copy bytes {0}")]
    Io(#[from] io::Error),

    // #[error("Got error while decompressing: header = {0:?}, error = {:?}")]
    // Decompress(DCXDecompressionParams, DecompressionError),
    #[error("Unrecognized DCX compression algorithm: {0:x?}")]
    UnknownAlgorithm([u8; 4]),
}

#[derive(Debug, Error)]
pub enum DecompressionError {
    #[error("Got oodle error code: {0}")]
    Oodle(u32),

    #[error("Got zlib error.")]
    Zlib,
}

#[derive(FromZeroes, FromBytes)]
#[repr(C)]
#[allow(unused)]
pub struct DCX<'a> {
    bytes: &'a [u8],

    header: &'a Header,
    sizes: &'a Sizes,
    compression_parameters: &'a CompressionParameters,
    compressed: &'a [u8],
}

impl<'a> DCX<'a> {
    pub fn parse(bytes: &'a [u8]) -> Option<Self> {
        // TODO: add magic validation
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

    pub fn create_decoder(&self) -> Result<DCXContentDecoder, DCXError> {
        let algorithm = &self.compression_parameters.algorithm;
        let decoder = match algorithm {
            MAGIC_ALGORITHM_KRAKEN => Decoder::Kraken(DCXDecoderKraken::from_buffer(
                self.compressed,
                self.sizes.uncompressed_size,
            )),
            MAGIC_ALGORITHM_DEFLATE => {
                Decoder::Deflate(DCXDecoderDeflate::from_buffer(self.compressed))
            }
            _ => return Err(DCXError::UnknownAlgorithm(algorithm.to_owned())),
        };

        Ok(DCXContentDecoder {
            uncompressed_size: self.sizes.uncompressed_size,
            compressed: &self.compressed,
            decoder,
        })
    }
}

impl<'a> std::fmt::Debug for DCX<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DCX")
            .field("header", self.header)
            .field("sizes", self.sizes)
            .field("compression_parameters", self.compression_parameters)
            .finish()
    }
}

const MAGIC_ALGORITHM_KRAKEN: &[u8; 4] = b"KRAK";
const MAGIC_ALGORITHM_DEFLATE: &[u8; 4] = b"DFLT";

pub enum Decoder<'a> {
    Kraken(DCXDecoderKraken<'a>),
    Deflate(DCXDecoderDeflate<'a>),
}

pub struct DCXContentDecoder<'a> {
    /// Reference to the compressed bytes.
    compressed: &'a [u8],

    /// Size of the contents once decompressed.
    uncompressed_size: U32<BE>,

    decoder: Decoder<'a>,
}

impl<'a> DCXContentDecoder<'a> {
    pub fn hint_size(&self) -> usize {
        self.uncompressed_size.get() as usize
    }
}

impl<'a> Read for DCXContentDecoder<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match &mut self.decoder {
            Decoder::Kraken(d) => d.read(buf),
            Decoder::Deflate(d) => d.read(buf),
        }
    }
}

pub struct DCXDecoderDeflate<'a>(ZlibDecoder<&'a [u8]>);

impl<'a> DCXDecoderDeflate<'a> {
    fn from_buffer(buf: &'a [u8]) -> Self {
        Self(ZlibDecoder::new(buf))
    }
}

impl<'a> Read for DCXDecoderDeflate<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

const DECODE_WINDOW_SIZE: i32 = 3 * 1024 * 1024;
const DICTIONARY_SIZE: i32 = 2 * 1024 * 1024;
const BUFFER_SIZE: i32 = (256 + 63) * 1024;
const COMPRESSOR: OodleLZ_Compressor = OodleLZ_Compressor_OodleLZ_Compressor_Kraken;

pub struct DCXDecoderKraken<'a> {
    compressed: &'a [u8],
    uncompressed_size: U32<BE>,
    decoder: *mut OodleLZDecoder,
}

impl<'a> DCXDecoderKraken<'a> {
    fn from_buffer(buf: &'a [u8], uncompressed_size: U32<BE>) -> Self {
        let raw_size = buf.len() as i64;

        let decoder =
            unsafe { OodleLZDecoder_Create(COMPRESSOR, raw_size, 0 as *mut std::ffi::c_void, -1) };

        Self {
            compressed: buf,
            uncompressed_size,
            decoder,
        }
    }
}

impl<'a> Drop for DCXDecoderKraken<'a> {
    fn drop(&mut self) {
        unsafe { OodleLZDecoder_Destroy(self.decoder) }
    }
}

impl<'a> Read for DCXDecoderKraken<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut dec_window = vec![0u8; DECODE_WINDOW_SIZE as usize];

        unsafe {
            let mut output = vec![0u8; size_of::<OodleLZ_DecodeSome_Out>()];

            OodleLZDecoder_DecodeSome(
                self.decoder,
                output.as_mut_ptr() as *mut OodleLZ_DecodeSome_Out,
                dec_window.as_mut_ptr() as *mut c_void, // dec_window
                DICTIONARY_SIZE as isize,               // dec_window_pos
                self.uncompressed_size.get() as isize,  // in_size
                DECODE_WINDOW_SIZE as isize - DICTIONARY_SIZE as isize, // dec_avail
                self.compressed.as_ptr() as *const c_void,
                self.compressed.len() as isize,
                OodleLZ_FuzzSafe_OodleLZ_FuzzSafe_Yes,
                OodleLZ_CheckCRC_OodleLZ_CheckCRC_No,
                OodleLZ_Verbosity_OodleLZ_Verbosity_None,
                OodleLZ_Decode_ThreadPhase_OodleLZ_Decode_ThreadPhaseAll,
            );
            dbg!(&*(output.as_ptr() as *const OodleLZ_DecodeSome_Out));
        }

        Ok(0)
    }
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C)]
#[allow(unused)]
/// The DCX chunk. Describes the layout of the container.
struct Header {
    chunk_magic: [u8; 4],

    /// Overal DCX file version
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

    /// Size of the data as it's in the DCA.
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
