/// Extensions for Rust standard library IO traits.
mod read;
mod seek;
mod widestring;
pub mod zerocopy;

pub use read::*;
pub use seek::*;
pub use widestring::*;
