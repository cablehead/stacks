use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub enum MimeType {
    #[serde(rename = "text/plain")]
    TextPlain,
    #[serde(rename = "image/png")]
    ImagePng,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Frame {
    pub id: scru128::Scru128Id,
    pub source: Option<String>,
    pub stack_hash: Option<ssri::Integrity>,
    pub mime_type: MimeType,
    pub hash: ssri::Integrity,
}

pub struct Store {
    db: sled::Db,
    cache_path: String,
}

impl Store {
    pub fn new(path: &str) -> Store {
        let db = sled::open(std::path::Path::new(path).join("index")).unwrap();
        let cache_path = std::path::Path::new(path)
            .join("cas")
            .into_os_string()
            .into_string()
            .unwrap();
        Store { db, cache_path }
    }

    pub fn cas_write(&self, content: &[u8]) -> ssri::Integrity {
        cacache::write_hash_sync(&self.cache_path, content).unwrap()
    }

    pub fn get(&mut self, id: &scru128::Scru128Id) -> Option<Frame> {
        self.db
            .get(id.to_bytes())
            .ok()
            .and_then(|maybe_value| maybe_value)
            .and_then(|value| bincode::deserialize::<Frame>(&value).ok())
    }

    pub fn insert(&mut self, frame: &Frame) {
        let encoded: Vec<u8> = bincode::serialize(&frame).unwrap();
        self.db.insert(frame.id.to_bytes(), encoded).unwrap();
    }

    pub fn list(&self) -> impl Iterator<Item = Frame> {
        self.db.iter().filter_map(|item| {
            item.ok()
                .and_then(|(_, value)| bincode::deserialize::<Frame>(&value).ok())
        })
    }

    pub fn cat(&self, hash: &ssri::Integrity) -> Option<Vec<u8>> {
        cacache::read_hash_sync(&self.cache_path, hash).ok()
    }
}
