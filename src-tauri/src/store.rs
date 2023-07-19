use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub enum MimeType {
    #[serde(rename = "text/plain")]
    TextPlain,
    #[serde(rename = "image/png")]
    ImagePng,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Frame {
    pub id: scru128::Scru128Id,
    pub source: Option<String>,
    pub stack_hash: Option<ssri::Integrity>,
    pub mime_type: MimeType,
    pub hash: ssri::Integrity,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct DeleteFrame {
    pub id: scru128::Scru128Id,
    pub hash: ssri::Integrity,
    pub stack_hash: Option<ssri::Integrity>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub enum Packet {
    Frame(Frame),
    DeleteFrame(DeleteFrame),
}

impl Packet {
    pub fn id(&self) -> &scru128::Scru128Id {
        match self {
            Packet::Frame(frame) => &frame.id,
            Packet::DeleteFrame(delete_frame) => &delete_frame.id,
        }
    }
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

    pub fn get(&mut self, id: &scru128::Scru128Id) -> Option<Packet> {
        self.db
            .get(id.to_bytes())
            .ok()
            .and_then(|maybe_value| maybe_value)
            .and_then(|value| bincode::deserialize::<Packet>(&value).ok())
    }

    pub fn get_frame(&mut self, id: &scru128::Scru128Id) -> Option<Frame> {
        match self.get(id) {
            Some(Packet::Frame(frame)) => Some(frame),
            _ => None,
        }
    }

    pub fn insert_frame(&mut self, frame: &Frame) -> Packet {
        let packet = Packet::Frame(frame.clone());
        self.insert(&packet);
        packet
    }

    pub fn insert(&mut self, packet: &Packet) {
        let encoded: Vec<u8> = bincode::serialize(&packet).unwrap();
        self.db.insert(packet.id().to_bytes(), encoded).unwrap();
    }

    pub fn delete(&mut self, hash: &ssri::Integrity, stack_hash: &Option<ssri::Integrity>) -> Packet {
        let frame = DeleteFrame {
            id: scru128::new(),
            hash: hash.clone(),
            stack_hash: stack_hash.clone(),
        };

        let packet = Packet::DeleteFrame(frame);
        self.insert(&packet);
        packet
    }

    pub fn list(&self) -> impl Iterator<Item = Packet> {
        self.db.iter().filter_map(|item| {
            item.ok()
                .and_then(|(_, value)| bincode::deserialize::<Packet>(&value).ok())
        })
    }

    pub fn cat(&self, hash: &ssri::Integrity) -> Option<Vec<u8>> {
        cacache::read_hash_sync(&self.cache_path, hash).ok()
    }
}
