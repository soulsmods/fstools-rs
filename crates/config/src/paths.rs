use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
pub struct Paths {
    pub elden_ring: Option<PathBuf>,
    pub elden_ring_keys: Option<PathBuf>,
}
