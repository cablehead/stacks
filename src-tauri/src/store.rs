use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::xs_lib;

pub struct Store {
    pub items: HashMap<String, Item>,
    cas: HashMap<String, Vec<u8>>,
    db_path: PathBuf,
}

impl Store {
    pub fn new(db_path: PathBuf) -> Self {
        Self {
            items: HashMap::new(),
            cas: HashMap::new(),
            db_path,
        }
    }

    pub fn cat(&self, hash: &str) -> Option<String> {
        self.cas
            .get(hash)
            .map(|content| String::from_utf8(content.clone()).unwrap())
    }

    fn create_or_merge(
        &mut self,
        id: scru128::Scru128Id,
        mime_type: &str,
        content_type: &str,
        terse: String,
        content: Vec<u8>,
    ) -> String {
        let hash = format!("{:x}", Sha256::digest(&content));

        match self.items.get_mut(&hash) {
            Some(curr) => {
                assert_eq!(curr.mime_type, mime_type, "Mime types don't match");
                curr.ids.push(id);
            }
            None => {
                self.items.insert(
                    hash.clone(),
                    Item {
                        hash: hash.clone(),
                        ids: vec![id],
                        mime_type: mime_type.to_string(),
                        terse,
                        link: None,
                        stack: HashMap::new(),
                        content_type: content_type.to_string(),
                    },
                );
                self.cas.insert(hash.clone(), content);
            }
        };

        hash
    }

    fn find_item_by_id(&mut self, id: &str) -> Option<Item> {
        let id = id.parse::<scru128::Scru128Id>().ok()?;
        for item in self.items.values() {
            if item.ids.contains(&id) {
                return Some(item.clone());
            }
        }
        None
    }

    pub fn add_frame(&mut self, frame: &xs_lib::Frame) {
        match &frame.topic {
            Some(topic) if topic == "clipboard" => {
                let clipped: Value = serde_json::from_str(&frame.data).unwrap();
                let types = clipped["types"].as_object().unwrap();

                if types.contains_key("public.utf8-plain-text") {
                    #[allow(deprecated)]
                    let content =
                        base64::decode(types["public.utf8-plain-text"].as_str().unwrap()).unwrap();

                    let content_type = if is_valid_https_url(&content) {
                        "Link"
                    } else {
                        "Text"
                    };

                    let _ = self.create_or_merge(
                        frame.id,
                        "text/plain",
                        content_type,
                        String::from_utf8(content.clone())
                            .unwrap()
                            .chars()
                            .take(100)
                            .collect(),
                        content,
                    );
                } else if types.contains_key("public.png") {
                    let content = types["public.png"].as_str().unwrap().as_bytes();
                    self.create_or_merge(
                        frame.id,
                        "image/png",
                        "Image",
                        clipped["source"].to_string(),
                        content.to_vec(),
                    );
                } else {
                    log::info!(
                        "add_frame TODO: topic: clipboard id: {}, types: {:?}, frame.data size: {}",
                        frame.id,
                        types.keys().collect::<Vec<_>>(),
                        frame.data.len()
                    );
                }
            }

            Some(topic) if topic == "stack" => {
                let data: Value = serde_json::from_str(&frame.data).unwrap();

                let id = data["id"].as_str();
                if let None = id {
                    return;
                }
                let id = id.unwrap();

                let target = self.find_item_by_id(id);
                if let None = target {
                    return;
                }
                let mut target = target.unwrap();

                let content = data["name"].as_str().unwrap();

                if let Some(attr) = &frame.attribute {
                    if attr == "delete" {
                        let hash = format!("{:x}", Sha256::digest(&content));
                        if let Some(stack) = self.items.get_mut(&hash) {
                            let id = id.parse::<scru128::Scru128Id>().ok().unwrap();
                            stack.stack.retain(|_, item| !item.ids.contains(&id));
                        }
                        return;
                    }
                }

                let hash = self.create_or_merge(
                    frame.id,
                    "text/plain",
                    "Stack",
                    content.chars().take(100).collect(),
                    content.as_bytes().to_vec(),
                );

                let item = self.items.get_mut(&hash).unwrap();
                item.content_type = "Stack".to_string();
                target.ids.push(frame.id);
                item.stack.insert(target.hash.to_string(), target);
            }

            Some(topic) => {
                log::info!("add_frame TODO: topic: {} id: {}", topic, frame.id,);
                if topic != "link" {
                    let _ = self.create_or_merge(
                        frame.id,
                        "text/plain",
                        "Text",
                        frame.data.chars().take(100).collect(),
                        frame.data.as_bytes().to_vec(),
                    );
                }
            }
            None => (),
        };
    }
}

#[derive(PartialEq, Debug, Clone, serde::Serialize)]
struct Link {
    provider: String,
    screenshot: String,
    title: String,
    description: String,
    url: String,
    icon: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Item {
    hash: String,
    pub ids: Vec<scru128::Scru128Id>,
    mime_type: String,
    pub content_type: String,
    pub terse: String,
    link: Option<Link>,
    pub stack: HashMap<String, Item>,
}

pub type SharedStore = Arc<Mutex<Store>>;

fn is_valid_https_url(url: &[u8]) -> bool {
    let re = regex::bytes::Regex::new(r"^https://[^\s/$.?#].[^\s]*$").unwrap();
    re.is_match(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_https_url() {
        assert!(is_valid_https_url(b"https://www.example.com"));
        assert!(!is_valid_https_url(b"Good afternoon"));
    }
}
