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
    pub hash: Option<Integrity>,
    pub mime_type: MimeType,
    pub content_type: String,
    pub terse: String,
    pub tiktokens: usize,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub enum Packet {
    Add(AddPacket),
    Update(UpdatePacket),
    Fork(ForkPacket),
    Delete(DeletePacket),
}

impl Packet {
    pub fn id(&self) -> Scru128Id {
        match self {
            Packet::Add(packet) => packet.id,
            Packet::Update(packet) => packet.id,
            Packet::Fork(packet) => packet.id,
            Packet::Delete(packet) => packet.id,
        }
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct AddPacket {
    pub id: Scru128Id,
    pub hash: Integrity,
    pub stack_id: Option<Scru128Id>,
    pub source: Option<String>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct UpdatePacket {
    pub id: Scru128Id,
    pub source_id: Scru128Id,
    pub hash: Option<Integrity>,
    pub stack_id: Option<Scru128Id>,
    pub source: Option<String>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct ForkPacket {
    pub id: Scru128Id,
    pub source_id: Scru128Id,
    pub hash: Option<Integrity>,
    pub stack_id: Option<Scru128Id>,
    pub source: Option<String>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct DeletePacket {
    pub id: Scru128Id,
    pub source_id: Scru128Id,
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

    pub fn query(&self, query: &str) -> std::collections::HashSet<ssri::Integrity> {
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

pub struct Store {
    packets: sled::Tree,
    pub content_meta: sled::Tree,
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

        Store {
            packets,
            content_meta,
            meta,
            cache_path,
            index: Index::new(path.join("index")),
        }
    }

    pub fn scan_content_meta(&self) -> std::collections::HashMap<ssri::Integrity, ContentMeta> {
        let mut content_meta_cache = std::collections::HashMap::new();
        for item in self.content_meta.iter() {
            if let Ok((key, value)) = item {
                if let Ok(hash) = bincode::deserialize::<ssri::Integrity>(&key) {
                    if let Ok(meta) = bincode::deserialize::<ContentMeta>(&value) {
                        content_meta_cache.insert(hash, meta);
                    }
                }
            }
        }
        content_meta_cache
    }

    pub fn get_content_meta(&self, hash: &ssri::Integrity) -> Option<ContentMeta> {
        let content_meta_cache = self.scan_content_meta();
        content_meta_cache.get(&hash).cloned()
    }

    pub fn cas_write(&mut self, content: &[u8], mime_type: MimeType) -> Integrity {
        let hash = cacache::write_hash_sync(&self.cache_path, content).unwrap();

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
            hash: Some(hash.clone()),
            mime_type: mime_type.clone(),
            content_type,
            terse,
            tiktokens,
        };
        let encoded: Vec<u8> = bincode::serialize(&meta).unwrap();
        let bytes = bincode::serialize(&hash).unwrap();
        self.content_meta.insert(bytes, encoded).unwrap();

        match mime_type {
            MimeType::TextPlain => self.index.write(&hash, content),
            MimeType::ImagePng => (),
        }

        hash
    }

    pub fn cas_read(&self, hash: &Integrity) -> Option<Vec<u8>> {
        cacache::read_hash_sync(&self.cache_path, hash).ok()
    }

    pub fn insert_packet(&mut self, packet: &Packet) {
        let encoded: Vec<u8> = bincode::serialize(&packet).unwrap();
        self.packets
            .insert(packet.id().to_bytes(), encoded)
            .unwrap();
    }

    pub fn scan(&self) -> impl Iterator<Item = Packet> {
        self.packets.iter().filter_map(|item| {
            item.ok()
                .and_then(|(_, value)| bincode::deserialize::<Packet>(&value).ok())
        })
    }

    pub fn add(
        &mut self,
        content: &[u8],
        mime_type: MimeType,
        stack_id: Option<Scru128Id>,
        source: Option<String>,
    ) -> Packet {
        let hash = self.cas_write(content, mime_type);
        let packet = Packet::Add(AddPacket {
            id: scru128::new(),
            hash,
            stack_id,
            source,
        });
        self.insert_packet(&packet);
        packet
    }

    pub fn update(
        &mut self,
        source_id: Scru128Id,
        content: Option<&[u8]>,
        mime_type: MimeType,
        stack_id: Option<Scru128Id>,
        source: Option<String>,
    ) -> Packet {
        let hash = content.map(|c| self.cas_write(c, mime_type.clone()));
        let packet = Packet::Update(UpdatePacket {
            id: scru128::new(),
            source_id,
            hash,
            stack_id,
            source,
        });
        self.insert_packet(&packet);
        packet
    }

    pub fn fork(
        &mut self,
        source_id: Scru128Id,
        content: Option<&[u8]>,
        mime_type: MimeType,
        stack_id: Option<Scru128Id>,
        source: Option<String>,
    ) -> Packet {
        let hash = content.map(|c| self.cas_write(c, mime_type.clone()));
        let packet = Packet::Fork(ForkPacket {
            id: scru128::new(),
            source_id,
            hash,
            stack_id,
            source,
        });
        self.insert_packet(&packet);
        packet
    }

    pub fn delete(&mut self, source_id: Scru128Id) -> Packet {
        let packet = Packet::Delete(DeletePacket {
            id: scru128::new(),
            source_id,
        });
        self.insert_packet(&packet);
        packet
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
        let packet = store.add(content, MimeType::TextPlain, None, None);

        let stored_packet = store.scan().next().unwrap();
        assert_eq!(packet, stored_packet);

        match packet {
            Packet::Add(packet) => {
                let stored_content = store.cas_read(&packet.hash).unwrap();
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
        let packet = store.add(content, MimeType::TextPlain, None, None);

        let updated_content = b"Hello, updated world!";
        let update_packet = store.update(
            packet.id().clone(),
            Some(updated_content),
            MimeType::TextPlain,
            None,
            None,
        );

        let stored_update_packet = store.scan().last().unwrap();
        assert_eq!(update_packet, stored_update_packet);

        match update_packet {
            Packet::Update(packet) => {
                let stored_content = store.cas_read(&packet.hash.unwrap()).unwrap();
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
        let packet = store.add(content, MimeType::TextPlain, None, None);

        let forked_content = b"Hello, forked world!";
        let forked_packet = store.fork(
            packet.id().clone(),
            Some(forked_content),
            MimeType::TextPlain,
            None,
            None,
        );

        let stored_fork_packet = store.scan().last().unwrap();
        assert_eq!(forked_packet, stored_fork_packet);

        match forked_packet {
            Packet::Fork(packet) => {
                let stored_content = store.cas_read(&packet.hash.unwrap()).unwrap();
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
        let packet = store.add(content, MimeType::TextPlain, None, None);
        let delete_packet = store.delete(packet.id().clone());
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

        store.add(content1, MimeType::TextPlain, None, None);
        store.add(content2, MimeType::TextPlain, None, None);
        store.add(content3, MimeType::TextPlain, None, None);

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

pub fn count_tiktokens(content: &str) -> usize {
    let bpe = tiktoken_rs::cl100k_base().unwrap();
    let tokens = bpe.encode_with_special_tokens(content);
    tokens.len()
}
