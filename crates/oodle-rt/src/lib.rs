use std::{
    error::Error,
    ffi::OsStr,
    ptr::{null_mut, NonNull},
    sync::{Arc, OnceLock, RwLock},
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

#[allow(warnings)]
pub(crate) mod ffi {
    include!("ffi.gen.rs");
}

pub use ffi::OODLELZ_BLOCK_LEN;
use steamlocate::SteamDir;
use walkdir::WalkDir;

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

const NR_APP_ID: u32 = 2622380;
const ER_APP_ID: u32 = 1245620;
const SDT_APP_ID: u32 = 249078;
const ACV_APP_ID: u32 = 1888160;

#[cfg(target_os = "macos")]
const SHARED_LIBRARY_EXTENSION: &str = "dylib";

#[cfg(windows)]
const SHARED_LIBRARY_EXTENSION: &str = "dll";

#[cfg(all(unix, not(target_os = "macos")))]
const SHARED_LIBRARY_EXTENSION: &str = "so";

fn oodle_lock() -> &'static RwLock<Option<Oodle>> {
    static CURRENT_OODLE: OnceLock<RwLock<Option<Oodle>>> = OnceLock::new();
    CURRENT_OODLE.get_or_init(|| RwLock::new(Oodle::find()))
}

impl Oodle {
    pub fn find() -> Option<Self> {
        let potential_apps = &[NR_APP_ID, ER_APP_ID, SDT_APP_ID, ACV_APP_ID];
        let steam_app_paths: Vec<_> = SteamDir::locate()
            .into_iter()
            .flat_map(|steam| {
                potential_apps.iter().filter_map(move |appid| {
                    let (app, library) = steam.find_app(*appid).ok().flatten()?;

                    Some(library.resolve_app_dir(&app))
                })
            })
            .collect();

        let oodle_dll_path = steam_app_paths
            .into_iter()
            .chain(std::env::current_dir().ok())
            .flat_map(|dir| {
                WalkDir::new(dir).into_iter().filter_map(|entry| {
                    let entry = entry.ok()?;
                    let file_name = entry.path().file_stem()?.to_str()?;
                    let file_ext = entry.path().extension()?.to_str()?;

                    // Might be a versioned .so, e.g. liboo2corelinux64.so.9
                    let is_shared_lib = file_name.ends_with(SHARED_LIBRARY_EXTENSION)
                        || file_ext == SHARED_LIBRARY_EXTENSION;
                    let is_oo2core = file_name.contains("oo2core");

                    if is_shared_lib && is_oo2core {
                        Some(entry.into_path())
                    } else {
                        None
                    }
                })
            })
            .next()?;

        // Safety: not at all
        unsafe { Self::load(oodle_dll_path).ok() }
    }

    pub fn current() -> Option<Self> {
        let guard = match oodle_lock().read() {
            Ok(guard) => guard,
            Err(e) => e.into_inner(),
        };

        guard.clone()
    }

    pub fn make_current(&self) {
        let mut guard = match oodle_lock().write() {
            Ok(guard) => guard,
            Err(e) => e.into_inner(),
        };

        *guard = Some(self.clone());
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
