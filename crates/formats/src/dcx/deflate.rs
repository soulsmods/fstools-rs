use std::io::{self, Read};

use flate2::read::ZlibDecoder;

pub struct DcxDecoderDeflate<R: Read>(ZlibDecoder<R>);

impl<R: Read> DcxDecoderDeflate<R> {
    pub fn new(reader: R) -> Self {
        Self(ZlibDecoder::new(reader))
    }
}

impl<R: Read> Read for DcxDecoderDeflate<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}
