use std::collections::HashMap;
use format::bhd2::BhdKey;

pub trait ArchiveKeyProvider {
    fn get_key(&self, name: &str) -> Option<BhdKey>;
}

impl ArchiveKeyProvider for HashMap<&str, &[u8; 429]> {
    fn get_key(&self, name: &str) -> Option<BhdKey> {
        self.get(name).and_then(|pem| BhdKey::public_key_from_pem_pkcs1(&pem[..]).ok())
    }
}
