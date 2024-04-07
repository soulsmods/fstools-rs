use std::{
    error::Error,
    ffi::OsStr,
    ptr::{null_mut, NonNull},
    sync::{Arc, RwLock},
};

use decoder::OodleDecoder;
use ffi::{
    Function_OodleLZDecoder_Create, Function_OodleLZDecoder_DecodeSome,
    Function_OodleLZDecoder_Destroy,
};
pub use ffi::{
    OodleLZ_Compressor as Compressor, OodleLZ_DecodeSome_Out as DecodeSome_Out,
    OodleLZ_Decode_ThreadPhase as DecodeThreadPhase, OodleLZ_Verbosity as Verbosity,
};
use libloading::Library;

thread_local! {
    static CURRENT_OODLE: RwLock<Option<Oodle>> = const { RwLock::new(None) };
}

#[allow(warnings)]
pub(crate) mod ffi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use ffi::OODLELZ_BLOCK_LEN;

pub mod decoder;

#[derive(Clone)]
pub struct Oodle {
    /// Handle to the loaded Oodle library.
    #[allow(unused)]
    library: Arc<Library>,
    pub(crate) oodle_lz_decoder_create: Function_OodleLZDecoder_Create,
    pub(crate) oodle_lz_decoder_destroy: Function_OodleLZDecoder_Destroy,
    pub(crate) oodle_lz_decoder_decode_some: Function_OodleLZDecoder_DecodeSome,
}

impl Oodle {
    pub fn make_current(&self) {
        CURRENT_OODLE.with(|oodle| {
            let mut guard = match oodle.write() {
                Ok(guard) => guard,
                Err(e) => e.into_inner(),
            };

            *guard = Some(self.clone());
        });
    }

    pub fn current() -> Option<Self> {
        CURRENT_OODLE.with(|oodle| {
            let guard = match oodle.read() {
                Ok(guard) => guard,
                Err(e) => e.into_inner(),
            };

            guard.clone()
        })
    }

    /// Load an Oodle shared library from the given module name or path.
    ///
    /// # Safety
    ///
    /// It is up to the caller to ensure that a correct Oodle library is loaded,
    /// and that no platform specific globals will be modified by the called initialization
    /// routines.
    pub unsafe fn load<S: AsRef<OsStr>>(library: S) -> Result<Self, Box<dyn Error>> {
        let library = Arc::new(libloading::Library::new(library)?);
        let oodle_lz_decoder_create = *library.get(b"OodleLZDecoder_Create\0")?;
        let oodle_lz_decoder_destroy = *library.get(b"OodleLZDecoder_Destroy\0")?;
        let oodle_lz_decoder_decode_some = *library.get(b"OodleLZDecoder_DecodeSome\0")?;

        Ok(Oodle {
            library,
            oodle_lz_decoder_create,
            oodle_lz_decoder_destroy,
            oodle_lz_decoder_decode_some,
        })
    }

    pub fn create_decoder(
        &self,
        compressor: Compressor,
        uncompressed_size: usize,
    ) -> Option<OodleDecoder> {
        // Safety: all non-optional parameters passed are valid representations of their types
        let ptr = unsafe {
            (self.oodle_lz_decoder_create.expect("null fn ptr"))(
                compressor,
                uncompressed_size as i64,
                null_mut(),
                0,
            )
        };

        let ptr = NonNull::new(ptr)?;

        Some(OodleDecoder::new(self.clone(), ptr, uncompressed_size))
    }
}
