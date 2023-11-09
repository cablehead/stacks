use std::collections::{HashMap, HashSet};

use scru128::Scru128Id;
use serde::{Deserialize, Serialize};
use ssri::Integrity;

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub enum MimeType {
    #[serde(rename = "text/plain")]
    TextPlain,
    #[serde(rename = "image/png")]
    ImagePng,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct ContentMeta {
    pub hash: Integrity,
    pub mime_type: MimeType,
    pub content_type: String,
    pub terse: String,
    pub tiktokens: usize,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct InProgressStream {
    pub content_meta: ContentMeta,
    pub content: Vec<u8>,
    pub packet: Packet,
}

impl InProgressStream {
    pub fn new(stack_id: Option<Scru128Id>, content: &[u8]) -> Self {
        let hash = ssri::Integrity::from(&content);
        let text_content = String::from_utf8_lossy(content).into_owned();
        let tiktokens = count_tiktokens(&text_content);
        let terse = if text_content.len() > 100 {
            text_content.chars().take(100).collect()
        } else {
            text_content
        };

        let content_type = if is_valid_https_url(content) {
            "Link".to_string()
        } else {
            "Text".to_string()
        };

        let content_meta = ContentMeta {
            hash: hash.clone(),
            mime_type: MimeType::TextPlain,
            content_type,
            terse,
            tiktokens,
        };

        InProgressStream {
            content_meta,
            content: content.to_vec(),
            packet: Packet {
                id: scru128::new(),
                packet_type: PacketType::Add,
                source_id: None,
                hash: Some(hash.clone()),
                stack_id,
                ephemeral: true,
                content_type: None,
                movement: None,
                lock_status: None,
                sort_order: None,
                cross_stream: false,
            },
        }
    }

    pub fn append(&mut self, content: &[u8]) {
        // Append additional content
        self.content.extend_from_slice(content);

        // Update hash
        self.content_meta.hash = ssri::Integrity::from(&self.content);

        // Update tiktokens
        let text_content = String::from_utf8_lossy(&self.content).into_owned();
        // self.content_meta.tiktokens = count_tiktokens(&text_content);

        // Update terse
        self.content_meta.terse = if text_content.len() > 100 {
            text_content.chars().take(100).collect()
        } else {
            text_content
        };

        self.packet.hash = Some(self.content_meta.hash.clone());
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub enum PacketType {
    Add,
    Update,
    Fork,
    Delete,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct PacketV3 {
    pub id: Scru128Id,
    pub packet_type: PacketType,
    pub source_id: Option<Scru128Id>,
    pub hash: Option<Integrity>,
    pub stack_id: Option<Scru128Id>,
    pub ephemeral: bool,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Movement {
    Up,
    Down,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum StackLockStatus {
    Unlocked,
    Locked,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum StackSortOrder {
    Auto,
    Manual,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Packet {
    pub id: Scru128Id,
    pub packet_type: PacketType,
    pub source_id: Option<Scru128Id>,
    pub hash: Option<Integrity>,
    pub stack_id: Option<Scru128Id>,
    pub ephemeral: bool,
    pub content_type: Option<String>,
    pub movement: Option<Movement>,
    pub lock_status: Option<StackLockStatus>,
    pub sort_order: Option<StackSortOrder>,
    pub cross_stream: bool,
}

fn deserialize_packet(value: &[u8]) -> Option<Packet> {
    bincode::deserialize::<Packet>(&value)
        .or_else(|_| {
            bincode::deserialize::<PacketV3>(&value).map(|v3_packet| Packet {
                id: v3_packet.id,
                packet_type: v3_packet.packet_type,
                source_id: v3_packet.source_id,
                hash: v3_packet.hash,
                stack_id: v3_packet.stack_id,
                ephemeral: v3_packet.ephemeral,
                content_type: None,
                movement: None,
                lock_status: None,
                sort_order: None,
                cross_stream: false,
            })
        })
        .ok()
}

pub struct Index {
    content_field: tantivy::schema::Field,
    hash_field: tantivy::schema::Field,
    writer: tantivy::IndexWriter,
    reader: tantivy::IndexReader,
}

impl Index {
    fn new(path: std::path::PathBuf) -> Index {
        let mut schema_builder = tantivy::schema::Schema::builder();
        let content_field = schema_builder.add_text_field("content", tantivy::schema::TEXT);
        let hash_field = schema_builder.add_bytes_field("hash", tantivy::schema::STORED);
        let schema = schema_builder.build();

        std::fs::create_dir_all(&path).unwrap();
        let dir = tantivy::directory::MmapDirectory::open(&path).unwrap();
        let index = tantivy::Index::open_or_create(dir, schema).unwrap();
        let writer = index.writer_with_num_threads(1, 3_000_000).unwrap();
        let reader = index.reader().unwrap();

        Index {
            content_field,
            hash_field,
            writer,
            reader,
        }
    }

    #[tracing::instrument(skip_all)]
    fn write(&mut self, hash: &ssri::Integrity, content: &[u8]) {
        let content = String::from_utf8_lossy(content);
        let mut doc = tantivy::Document::new();
        doc.add_text(self.content_field, &content);
        let bytes = bincode::serialize(&hash).unwrap();
        doc.add_bytes(self.hash_field, bytes);
        self.writer.add_document(doc).unwrap();
        self.writer.commit().unwrap();
        self.reader.reload().unwrap();
    }

    #[cfg(test)]
    pub fn query(&self, query: &str) -> HashSet<ssri::Integrity> {
        let term = tantivy::schema::Term::from_field_text(self.content_field, query);
        let query = tantivy::query::FuzzyTermQuery::new_prefix(term, 1, true);

        let searcher = self.reader.searcher();
        let top_docs = searcher
            .search(&query, &tantivy::collector::TopDocs::with_limit(10000))
            .unwrap();

        top_docs
            .into_iter()
            .map(|(_, doc_address)| {
                let doc = searcher.doc(doc_address).unwrap();
                let bytes = doc.get_first(self.hash_field).unwrap().as_bytes().unwrap();
                let hash: ssri::Integrity = bincode::deserialize(bytes).unwrap();
                hash
            })
            .collect()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub openai_access_token: String,
    pub openai_selected_model: String,
    pub cross_stream_access_token: Option<String>,
}

pub struct Store {
    packets: sled::Tree,
    content_meta: sled::Tree,
    content_meta_cache: HashMap<ssri::Integrity, ContentMeta>,
    pub meta: sled::Tree,
    pub cache_path: String,
    pub index: Index,
    in_progress_streams: HashMap<Scru128Id, InProgressStream>,
}

impl Store {
    pub fn new(path: &str) -> Store {
        let path = std::path::Path::new(path);
        let db = sled::open(path.join("sled")).unwrap();
        let packets = db.open_tree("packets").unwrap();
        let content_meta = db.open_tree("content_meta").unwrap();
        let meta = db.open_tree("meta").unwrap();
        let cache_path = path.join("cas").into_os_string().into_string().unwrap();

        let mut store = Store {
            packets,
            content_meta,
            content_meta_cache: HashMap::new(),
            meta,
            cache_path,
            index: Index::new(path.join("index")),
            in_progress_streams: HashMap::new(),
        };
        store.content_meta_cache = store.scan_content_meta();
        store
    }

    pub fn query(&self, filter: &str, content_type: &str) -> HashSet<ssri::Integrity> {
        let filter = filter.to_lowercase();
        let content_type = content_type.to_lowercase();

        self.content_meta_cache
            .iter()
            .filter_map(|(hash, meta)| {
                let terse = meta.terse.to_lowercase();
                let content_type_meta = meta.content_type.to_lowercase();

                if (filter.is_empty() || terse.contains(&filter))
                    && (content_type.is_empty()
                        || content_type == "all"
                        || content_type_meta == content_type)
                {
                    Some(hash.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn scan_content_meta(&self) -> HashMap<ssri::Integrity, ContentMeta> {
        let mut content_meta_cache = HashMap::new();
        for (key, value) in self.content_meta.iter().flatten() {
            if let Ok(hash) = bincode::deserialize::<ssri::Integrity>(&key) {
                if let Ok(meta) = bincode::deserialize::<ContentMeta>(&value) {
                    content_meta_cache.insert(hash, meta);
                }
            }
        }

        self.scan().for_each(|p| {
            if p.packet_type == PacketType::Update {
                if let Some(hash) = p.hash.clone() {
                    if let Some(content_type) = p.content_type.clone() {
                        if let Some(meta) = content_meta_cache.get_mut(&hash) {
                            meta.content_type = content_type;
                        }
                    }
                }
            }
        });

        content_meta_cache
    }

    pub fn get_content_meta(&self, hash: &ssri::Integrity) -> Option<ContentMeta> {
        match self.content_meta_cache.get(hash) {
            Some(meta) => Some(meta.clone()),
            None => {
                for stream in self.in_progress_streams.values() {
                    if let Some(stream_hash) = &stream.packet.hash {
                        if stream_hash == hash {
                            return Some(stream.content_meta.clone());
                        }
                    }
                }
                None
            }
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn get_content(&self, hash: &ssri::Integrity) -> Option<Vec<u8>> {
        match self.cas_read(hash) {
            Some(content) => Some(content),
            None => {
                for stream in self.in_progress_streams.values() {
                    if let Some(stream_hash) = &stream.packet.hash {
                        if stream_hash == hash {
                            return Some(stream.content.clone());
                        }
                    }
                }
                None
            }
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn cas_write(&mut self, content: &[u8], mime_type: MimeType) -> Integrity {
        let hash = cacache::write_hash_sync(&self.cache_path, content).unwrap();
        if let Some(_) = self.content_meta_cache.get(&hash) {
            return hash;
        }

        let (content_type, terse, tiktokens) = match mime_type {
            MimeType::TextPlain => {
                let text_content = String::from_utf8_lossy(content).into_owned();
                let tiktokens = count_tiktokens(&text_content);
                let terse = if text_content.len() > 100 {
                    text_content.chars().take(100).collect()
                } else {
                    text_content
                };

                let content_type = if is_valid_https_url(content) {
                    "Link".to_string()
                } else {
                    "Text".to_string()
                };
                (content_type, terse, tiktokens)
            }
            MimeType::ImagePng => ("Image".to_string(), "Image".to_string(), 0),
        };

        let meta = ContentMeta {
            hash: hash.clone(),
            mime_type: mime_type.clone(),
            content_type,
            terse,
            tiktokens,
        };
        let encoded: Vec<u8> = bincode::serialize(&meta).unwrap();
        let bytes = bincode::serialize(&hash).unwrap();
        self.content_meta.insert(bytes, encoded).unwrap();

        self.content_meta_cache.insert(hash.clone(), meta);

        match mime_type {
            MimeType::TextPlain => self.index.write(&hash, content),
            MimeType::ImagePng => (),
        }

        hash
    }

    fn cas_read(&self, hash: &Integrity) -> Option<Vec<u8>> {
        cacache::read_hash_sync(&self.cache_path, hash).ok()
    }

    pub fn insert_packet(&mut self, packet: &Packet) {
        let encoded: Vec<u8> = bincode::serialize(&packet).unwrap();
        self.packets.insert(packet.id.to_bytes(), encoded).unwrap();
    }

    pub fn scan(&self) -> impl Iterator<Item = Packet> {
        self.packets
            .iter()
            .filter_map(|item| item.ok().and_then(|(_, value)| deserialize_packet(&value)))
    }

    pub fn add(&mut self, content: &[u8], mime_type: MimeType, stack_id: Scru128Id) -> Packet {
        let hash = self.cas_write(content, mime_type);
        let packet = Packet {
            id: scru128::new(),
            packet_type: PacketType::Add,
            source_id: None,
            hash: Some(hash),
            stack_id: Some(stack_id),
            ephemeral: false,
            content_type: None,
            movement: None,
            lock_status: None,
            sort_order: None,
            cross_stream: false,
        };
        self.insert_packet(&packet);
        packet
    }

    pub fn add_stack(&mut self, name: &[u8], lock_status: StackLockStatus) -> Packet {
        let hash = self.cas_write(name, MimeType::TextPlain);
        let packet = Packet {
            id: scru128::new(),
            packet_type: PacketType::Add,
            source_id: None,
            hash: Some(hash),
            stack_id: None,
            ephemeral: false,
            content_type: None,
            movement: None,
            lock_status: Some(lock_status),
            sort_order: None,
            cross_stream: false,
        };
        self.insert_packet(&packet);
        packet
    }

    pub fn update(
        &mut self,
        source_id: Scru128Id,
        content: Option<&[u8]>,
        mime_type: MimeType,
        stack_id: Option<Scru128Id>,
    ) -> Packet {
        let hash = content.map(|c| self.cas_write(c, mime_type.clone()));
        let packet = Packet {
            id: scru128::new(),
            packet_type: PacketType::Update,
            source_id: Some(source_id),
            hash,
            stack_id,
            ephemeral: false,
            content_type: None,
            movement: None,
            lock_status: None,
            sort_order: None,
            cross_stream: false,
        };
        self.insert_packet(&packet);
        packet
    }

    pub fn update_touch(&mut self, source_id: Scru128Id) -> Packet {
        let packet = Packet {
            id: scru128::new(),
            packet_type: PacketType::Update,
            source_id: Some(source_id),
            hash: None,
            stack_id: None,
            ephemeral: false,
            content_type: None,
            movement: None,
            lock_status: None,
            sort_order: None,
            cross_stream: false,
        };
        self.insert_packet(&packet);
        packet
    }

    pub fn update_content_type(&mut self, hash: ssri::Integrity, content_type: String) -> Packet {
        let mut meta = self.content_meta_cache.get(&hash).unwrap().clone();
        let packet = Packet {
            id: scru128::new(),
            packet_type: PacketType::Update,
            source_id: None,
            hash: Some(hash.clone()),
            stack_id: None,
            ephemeral: false,
            content_type: Some(content_type.clone()),
            movement: None,
            lock_status: None,
            sort_order: None,
            cross_stream: false,
        };
        self.insert_packet(&packet);
        meta.content_type = content_type;
        self.content_meta_cache.insert(hash, meta);
        packet
    }

    pub fn update_move(&mut self, source_id: Scru128Id, movement: Movement) -> Packet {
        let packet = Packet {
            id: scru128::new(),
            packet_type: PacketType::Update,
            source_id: Some(source_id),
            hash: None,
            stack_id: None,
            ephemeral: false,
            content_type: None,
            movement: Some(movement),
            lock_status: None,
            sort_order: None,
            cross_stream: false,
        };
        self.insert_packet(&packet);
        packet
    }

    pub fn mark_as_cross_stream(&mut self, stack_id: Scru128Id) -> Packet {
        let packet = Packet {
            id: scru128::new(),
            packet_type: PacketType::Update,
            source_id: None,
            hash: None,
            stack_id: Some(stack_id),
            ephemeral: false,
            content_type: None,
            movement: None,
            lock_status: None,
            sort_order: None,
            cross_stream: true,
        };
        self.insert_packet(&packet);
        packet
    }

    pub fn update_stack_lock_status(
        &mut self,
        source_id: Scru128Id,
        lock_status: StackLockStatus,
    ) -> Packet {
        let packet = Packet {
            id: scru128::new(),
            packet_type: PacketType::Update,
            source_id: Some(source_id),
            hash: None,
            stack_id: None,
            ephemeral: false,
            content_type: None,
            movement: None,
            lock_status: Some(lock_status),
            sort_order: None,
            cross_stream: false,
        };
        self.insert_packet(&packet);
        packet
    }

    pub fn update_stack_sort_order(
        &mut self,
        source_id: Scru128Id,
        sort_order: StackSortOrder,
    ) -> Packet {
        let packet = Packet {
            id: scru128::new(),
            packet_type: PacketType::Update,
            source_id: Some(source_id),
            hash: None,
            stack_id: None,
            ephemeral: false,
            content_type: None,
            movement: None,
            lock_status: None,
            sort_order: Some(sort_order),
            cross_stream: false,
        };
        self.insert_packet(&packet);
        packet
    }

    pub fn fork(
        &mut self,
        source_id: Scru128Id,
        content: Option<&[u8]>,
        mime_type: MimeType,
        stack_id: Option<Scru128Id>,
    ) -> Packet {
        let hash = content.map(|c| self.cas_write(c, mime_type.clone()));
        let packet = Packet {
            id: scru128::new(),
            packet_type: PacketType::Fork,
            source_id: Some(source_id),
            hash,
            stack_id,
            ephemeral: false,
            content_type: None,
            movement: None,
            lock_status: None,
            sort_order: None,
            cross_stream: false,
        };
        self.insert_packet(&packet);
        packet
    }

    pub fn delete(&mut self, source_id: Scru128Id) -> Packet {
        let packet = Packet {
            id: scru128::new(),
            packet_type: PacketType::Delete,
            source_id: Some(source_id),
            hash: None,
            stack_id: None,
            ephemeral: false,
            content_type: None,
            movement: None,
            lock_status: None,
            sort_order: None,
            cross_stream: false,
        };
        self.insert_packet(&packet);
        packet
    }

    pub fn remove_packet(&mut self, id: &Scru128Id) -> Option<Packet> {
        let removed = self.packets.remove(id.to_bytes()).unwrap();
        removed.and_then(|value| deserialize_packet(&value))
    }

    pub fn settings_save(&mut self, settings: Settings) {
        let settings_str = serde_json::to_string(&settings).unwrap();
        self.meta
            .insert("settings", settings_str.as_bytes())
            .unwrap();
    }

    pub fn settings_get(&self) -> Option<Settings> {
        let res = self.meta.get("settings").unwrap();
        res.map(|bytes| {
            let str = std::str::from_utf8(bytes.as_ref()).unwrap();
            serde_json::from_str(str).unwrap()
        })
    }

    pub fn start_stream(&mut self, stack_id: Option<Scru128Id>, content: &[u8]) -> Packet {
        let stream = InProgressStream::new(stack_id, content);
        let packet = stream.packet.clone();
        self.in_progress_streams.insert(stream.packet.id, stream);
        packet
    }

    pub fn update_stream(&mut self, id: Scru128Id, content: &[u8]) -> Packet {
        let stream = self.in_progress_streams.get_mut(&id).unwrap();
        stream.append(content);
        stream.packet.clone()
    }

    pub fn end_stream(&mut self, id: Scru128Id) -> Packet {
        let mut stream = self.in_progress_streams.remove(&id).unwrap();
        let hash = self.cas_write(&stream.content, stream.content_meta.mime_type);
        stream.packet.hash = Some(hash);
        stream.packet.ephemeral = false;
        self.insert_packet(&stream.packet);
        stream.packet
    }
}

fn is_valid_https_url(url: &[u8]) -> bool {
    let re = regex::bytes::Regex::new(r"^https://[^\s/$.?#].[^\s]*$").unwrap();
    re.is_match(url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_add() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();

        let mut store = Store::new(path);

        let content = b"Hello, world!";
        let packet = store.add_stack(content, StackLockStatus::Unlocked);

        let stored_packet = store.scan().next().unwrap();
        assert_eq!(packet, stored_packet);

        match packet.packet_type {
            PacketType::Add => {
                let stored_content = store.cas_read(&packet.hash.unwrap()).unwrap();
                assert_eq!(content.to_vec(), stored_content);
            }
            _ => panic!("Expected AddPacket"),
        }
    }

    #[test]
    fn test_update() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();

        let mut store = Store::new(path);

        let content = b"Hello, world!";
        let packet = store.add_stack(content, StackLockStatus::Unlocked);

        let updated_content = b"Hello, updated world!";
        let update_packet = store.update(
            packet.id.clone(),
            Some(updated_content),
            MimeType::TextPlain,
            None,
        );

        let stored_update_packet = store.scan().last().unwrap();
        assert_eq!(update_packet, stored_update_packet);

        match stored_update_packet {
            Packet {
                packet_type: PacketType::Update,
                hash: Some(hash),
                ..
            } => {
                let stored_content = store.cas_read(&hash).unwrap();
                assert_eq!(updated_content.to_vec(), stored_content);
            }
            _ => panic!("Expected UpdatePacket"),
        }
    }

    #[test]
    fn test_fork() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();

        let mut store = Store::new(path);

        let content = b"Hello, world!";
        let packet = store.add_stack(content, StackLockStatus::Unlocked);

        let forked_content = b"Hello, forked world!";
        let forked_packet = store.fork(
            packet.id.clone(),
            Some(forked_content),
            MimeType::TextPlain,
            None,
        );

        let stored_fork_packet = store.scan().last().unwrap();
        assert_eq!(forked_packet, stored_fork_packet);

        match forked_packet {
            Packet {
                packet_type: PacketType::Fork,
                hash,
                ..
            } => {
                let stored_content = store.cas_read(&hash.unwrap()).unwrap();
                assert_eq!(forked_content.to_vec(), stored_content);
            }
            _ => panic!("Expected ForkPacket"),
        }
    }

    #[test]
    fn test_delete() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let mut store = Store::new(path);
        let content = b"Hello, world!";
        let packet = store.add_stack(content, StackLockStatus::Unlocked);
        let delete_packet = store.delete(packet.id.clone());
        let stored_delete_packet = store.scan().last().unwrap();
        assert_eq!(delete_packet, stored_delete_packet);
    }

    #[test]
    fn test_query() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();

        let mut store = Store::new(path);

        let content1 = b"Hello, world!";
        let content2 = b"Hello, fuzzy world!";
        let content3 = b"Hello, there!";

        store.add_stack(content1, StackLockStatus::Unlocked);
        store.add_stack(content2, StackLockStatus::Unlocked);
        store.add_stack(content3, StackLockStatus::Unlocked);

        let results = store.index.query("fzzy");
        let results: Vec<_> = results
            .into_iter()
            .map(|hash| store.cas_read(&hash).unwrap())
            .collect();
        assert_eq!(results, vec![b"Hello, fuzzy world!".to_vec()]);
    }

    #[test]
    fn test_is_valid_https_url() {
        assert!(is_valid_https_url(b"https://www.example.com"));
        assert!(!is_valid_https_url(b"Good afternoon"));
    }
}

#[tracing::instrument(skip_all)]
pub fn count_tiktokens(content: &str) -> usize {
    let bpe = tiktoken_rs::cl100k_base().unwrap();
    let tokens = bpe.encode_with_special_tokens(content);
    tokens.len()
}
