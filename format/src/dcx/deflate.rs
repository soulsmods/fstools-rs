use std::io::{self, Read};
use flate2::{read::ZlibDecoder, Compression};

pub struct DcxDecoderDeflate<'a>(ZlibDecoder<&'a [u8]>);

impl<'a> DcxDecoderDeflate<'a> {
    pub fn from_buffer(buf: &'a [u8]) -> Self {
        Self(ZlibDecoder::new(buf))
    }
}

impl<'a> Read for DcxDecoderDeflate<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}
