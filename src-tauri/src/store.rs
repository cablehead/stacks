use std::collections::{HashMap, HashSet};

use scru128::Scru128Id;
use serde::{Deserialize, Serialize};
use ssri::Integrity;

use crate::spotlight;

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
    #[tracing::instrument(name = "InProgressStream::new")]
    pub fn new(stack_id: Scru128Id, mime_type: MimeType, content_type: String) -> Self {
        let hash = ssri::Integrity::from("");

        let content_meta = ContentMeta {
            hash: hash.clone(),
            mime_type,
            content_type: content_type.clone(),
            terse: "".to_string(),
            tiktokens: 0,
        };

        InProgressStream {
            content_meta,
            content: Vec::new(),
            packet: Packet {
                id: scru128::new(),
                packet_type: PacketType::Add,
                source_id: None,
                hash: Some(hash.clone()),
                stack_id: Some(stack_id),
                ephemeral: true,
                content_type: Some(content_type),
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

        let text_content = String::from_utf8_lossy(&self.content).into_owned();

        // Update terse
        self.content_meta.terse = if text_content.len() > 100 {
            text_content.chars().take(100).collect()
        } else {
            text_content
        };

        self.packet.hash = Some(self.content_meta.hash.clone());
    }

    pub fn end_stream(&mut self, store: &mut Store) -> Packet {
        let hash = store.cas_write(
            &self.content,
            self.content_meta.mime_type.clone(),
            self.content_meta.content_type.clone(),
        );
        self.packet.hash = Some(hash);
        self.packet.ephemeral = false;
        self.packet.clone()
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
    bincode::deserialize::<Packet>(value)
        .or_else(|_| {
            bincode::deserialize::<PacketV3>(value).map(|v3_packet| Packet {
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
        let writer = index.writer_with_num_threads(1, 15_000_000).unwrap();
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
        let mut doc = tantivy::TantivyDocument::new();
        doc.add_text(self.content_field, &content);
        let bytes = bincode::serialize(&hash).unwrap();
        doc.add_bytes(self.hash_field, bytes);
        self.writer.add_document(doc).unwrap();
        self.writer.commit().unwrap();
        self.reader.reload().unwrap();
    }

    #[cfg(test)]
    pub fn query(&self, query: &str) -> HashSet<ssri::Integrity> {
        use tantivy::schema::Value;
        let term = tantivy::schema::Term::from_field_text(self.content_field, query);
        let query = tantivy::query::FuzzyTermQuery::new_prefix(term, 1, true);

        let searcher = self.reader.searcher();
        let top_docs = searcher
            .search(&query, &tantivy::collector::TopDocs::with_limit(10000))
            .unwrap();

        top_docs
            .into_iter()
            .map(|(_, doc_address)| {
                let doc: tantivy::TantivyDocument = searcher.doc(doc_address).unwrap();
                let bytes = doc.get_first(self.hash_field).unwrap().as_bytes().unwrap();
                let hash: ssri::Integrity = bincode::deserialize(bytes).unwrap();
                hash
            })
            .collect()
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Settings {
    pub openai_access_token: String,
    pub openai_selected_model: String,
    pub cross_stream_access_token: Option<String>,
    pub activation_shortcut: Option<spotlight::Shortcut>,
}

pub struct Store {
    packets: sled::Tree,
    content_meta: sled::Tree,
    content_meta_cache: HashMap<ssri::Integrity, ContentMeta>,
    syntaxes: HashSet<String>,
    pub content_bus_tx: tokio::sync::broadcast::Sender<ContentMeta>,
    pub meta: sled::Tree,
    pub cache_path: String,
    pub index: Index,
}

impl Store {
    pub fn new(path: &str) -> Store {
        let path = std::path::Path::new(path);
        let db = sled::open(path.join("sled")).unwrap();
        let packets = db.open_tree("packets").unwrap();
        let content_meta = db.open_tree("content_meta").unwrap();
        let meta = db.open_tree("meta").unwrap();
        let cache_path = path.join("cas").into_os_string().into_string().unwrap();

        let (content_bus_tx, _rx) = tokio::sync::broadcast::channel(20);

        let mut store = Store {
            packets,
            content_meta,
            content_meta_cache: HashMap::new(),
            // TODO: oh my
            syntaxes: syntect::parsing::SyntaxSet::load_defaults_nonewlines()
                .syntaxes()
                .iter()
                .map(|syntax| syntax.name.to_lowercase())
                .filter(|name| name != "markdown")
                .collect(),

            content_bus_tx,
            meta,
            cache_path,
            index: Index::new(path.join("index")),
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

                // TODO: oh my
                if (filter.is_empty() || terse.contains(&filter))
                    && (content_type.is_empty()
                        || content_type == "all"
                        || content_type_meta == content_type
                        || (content_type == "source code"
                            && self.syntaxes.contains(&content_type_meta)))
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
            let hash = bincode::deserialize::<ssri::Integrity>(&key);
            let meta = bincode::deserialize::<ContentMeta>(&value);

            match (hash, meta) {
                (Ok(hash), Ok(meta)) => {
                    // Skip content metadata if CAS content no longer exists
                    if self.cas_read(&hash).is_none() {
                        tracing::warn!("Skipping content metadata for missing CAS entry: {}", hash);
                        continue;
                    }

                    if meta.mime_type == MimeType::TextPlain
                        && meta.tiktokens == 0
                        && !meta.terse.is_empty()
                    {
                        tracing::warn!("TODO: backfill tiktokens for content that falls through the cracks: {:?}", &meta);
                    }
                    content_meta_cache.insert(hash, meta);
                }
                (Err(e), _) | (_, Err(e)) => {
                    panic!("Could not deserialize content: {e:?}");
                }
            }
        }

        self.scan()
            .filter(|p| p.packet_type == PacketType::Update || p.packet_type == PacketType::Add)
            .for_each(|p| {
                let meta = p.hash.and_then(|hash| content_meta_cache.get_mut(&hash));
                if let (Some(meta), Some(content_type)) = (meta, p.content_type) {
                    meta.content_type = content_type;
                }
            });

        content_meta_cache
    }

    pub fn get_content_meta(&self, hash: &ssri::Integrity) -> Option<ContentMeta> {
        self.content_meta_cache.get(hash).cloned()
    }

    #[tracing::instrument(skip_all)]
    pub fn get_content(&self, hash: &ssri::Integrity) -> Option<Vec<u8>> {
        self.cas_read(hash)
    }

    #[tracing::instrument(skip_all)]
    pub fn cas_write(
        &mut self,
        content: &[u8],
        mime_type: MimeType,
        content_type: String,
    ) -> Integrity {
        let hash = cacache::write_hash_sync(&self.cache_path, content).unwrap();
        if let Some(meta) = self.content_meta_cache.get_mut(&hash) {
            meta.content_type = content_type;
            return hash;
        }

        let terse = match mime_type {
            MimeType::TextPlain => {
                let text_content = String::from_utf8_lossy(content).into_owned();
                if text_content.len() > 100 {
                    text_content.chars().take(100).collect()
                } else {
                    text_content
                }
            }
            MimeType::ImagePng => "Image".to_string(),
        };

        let meta = ContentMeta {
            hash: hash.clone(),
            mime_type: mime_type.clone(),
            content_type,
            terse,
            tiktokens: 0,
        };
        let encoded: Vec<u8> = bincode::serialize(&meta).unwrap();
        let bytes = bincode::serialize(&hash).unwrap();
        self.content_meta.insert(bytes, encoded).unwrap();

        self.content_meta_cache.insert(hash.clone(), meta.clone());

        match mime_type {
            MimeType::TextPlain => self.index.write(&hash, content),
            MimeType::ImagePng => (),
        }

        let _ = self.content_bus_tx.send(meta);

        hash
    }

    pub fn cas_read(&self, hash: &Integrity) -> Option<Vec<u8>> {
        cacache::read_hash_sync(&self.cache_path, hash).ok()
    }

    #[tracing::instrument(skip_all)]
    pub fn purge(&mut self, hash: &Integrity) -> Result<(), Box<dyn std::error::Error>> {
        // Remove from CAS storage
        cacache::remove_hash_sync(&self.cache_path, hash)?;

        // Remove from content metadata
        let hash_bytes = bincode::serialize(hash)?;
        self.content_meta.remove(hash_bytes)?;

        // Remove from in-memory cache
        self.content_meta_cache.remove(hash);

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub fn enumerate_cas(&self) -> Vec<ssri::Integrity> {
        // Since we use cacache::write_hash_sync (no key), list_sync won't find entries.
        // Instead, enumerate from our content metadata cache, which tracks all CAS hashes.
        self.content_meta_cache.keys().cloned().collect()
    }

    pub fn update_tiktokens(&mut self, hash: ssri::Integrity, tiktokens: usize) {
        if let Some(meta) = self.content_meta_cache.get(&hash) {
            let mut meta = meta.clone();
            meta.tiktokens = tiktokens;

            let encoded: Vec<u8> = bincode::serialize(&meta).unwrap();
            let hash_bytes = bincode::serialize(&hash).unwrap();
            self.content_meta
                .insert(hash_bytes, encoded.clone())
                .unwrap();
            self.content_meta_cache.insert(hash, meta.clone());
        }
    }

    pub fn insert_packet(&self, packet: &Packet) {
        let encoded: Vec<u8> = bincode::serialize(&packet).unwrap();
        self.packets.insert(packet.id.to_bytes(), encoded).unwrap();
    }

    pub fn scan(&self) -> impl Iterator<Item = Packet> + use<'_> {
        self.packets
            .iter()
            .filter_map(|item| item.ok().and_then(|(_, value)| deserialize_packet(&value)))
            .filter(|packet| {
                // Skip packets with dangling CAS hashes
                if let Some(hash) = &packet.hash {
                    if self.cas_read(hash).is_none() {
                        tracing::warn!("Skipping packet with missing CAS content: {}", hash);
                        return false;
                    }
                }
                true
            })
    }

    pub fn add(&mut self, content: &[u8], mime_type: MimeType, stack_id: Scru128Id) -> Packet {
        let (mime_type, content_type) = infer_mime_type(content, mime_type);
        let hash = self.cas_write(content, mime_type, content_type.clone());
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
        let hash = self.cas_write(name, MimeType::TextPlain, "Text".to_string());
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
        let hash = content.map(|c| {
            let (mime_type, content_type) = infer_mime_type(c, mime_type);
            self.cas_write(c, mime_type, content_type)
        });
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

    pub fn update_touch(&self, source_id: Scru128Id) -> Packet {
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

    pub fn update_move(&self, source_id: Scru128Id, movement: Movement) -> Packet {
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

    pub fn mark_as_cross_stream(&self, stack_id: Scru128Id) -> Packet {
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
        &self,
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
        &self,
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
        let hash = content.map(|c| {
            let (mime_type, content_type) = infer_mime_type(c, mime_type);
            self.cas_write(c, mime_type, content_type)
        });
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

    pub fn delete(&self, source_id: Scru128Id) -> Packet {
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

    pub fn get_packet(&self, id: &Scru128Id) -> Option<Packet> {
        let value = self.packets.get(id.to_bytes()).unwrap();
        value.and_then(|value| deserialize_packet(&value))
    }

    pub fn remove_packet(&self, id: &Scru128Id) -> Option<Packet> {
        let removed = self.packets.remove(id.to_bytes()).unwrap();
        removed.and_then(|value| deserialize_packet(&value))
    }

    pub fn settings_save(&self, settings: Settings) {
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
}

pub fn is_valid_https_url(url: &[u8]) -> bool {
    let re = regex::bytes::Regex::new(r"^https://[^\s/$.?#].[^\s]*$").unwrap();
    re.is_match(url)
}

#[tracing::instrument(skip_all)]
pub fn count_tiktokens(content: &str) -> usize {
    let bpe = tiktoken_rs::cl100k_base().unwrap();
    let tokens = bpe.encode_with_special_tokens(content);
    tokens.len()
}

#[tracing::instrument(skip_all)]
pub fn infer_mime_type(content: &[u8], mime_type: MimeType) -> (MimeType, String) {
    let content_type = match mime_type {
        MimeType::TextPlain => {
            if is_valid_https_url(content) {
                "Link".to_string()
            } else {
                "Text".to_string()
            }
        }
        MimeType::ImagePng => "Image".to_string(),
    };

    (mime_type, content_type)
}
