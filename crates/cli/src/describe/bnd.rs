use serde::{Deserialize, Serialize};

#[derive(Describe, Serialize, Deserialize)]
pub struct BndFileDescription {
    #[serde(rename = "Name")]
    name: String,

    #[serde(rename = "Size (in bytes)")]
    size: usize,
}

#[derive(Describe, Serialize, Deserialize)]
pub struct BndDescription {
    #[serde(rename = "Files")]
    files: Vec<BndFileDescription>,
}
