use std::collections::HashMap;

use crate::store::{Frame, MimeType};

#[derive(Debug, Clone, serde::Serialize)]
pub struct Item {
    hash: ssri::Integrity,
    pub ids: Vec<scru128::Scru128Id>,
    mime_type: MimeType,
    pub content_type: String,
    pub terse: String,
    pub stack: HashMap<ssri::Integrity, Item>,
}

fn merge_into(stack: &mut HashMap<ssri::Integrity, Item>, frame: &Frame, content: &[u8]) {
    match stack.get_mut(&frame.hash) {
        Some(curr) => {
            assert_eq!(curr.mime_type, frame.mime_type, "Mime types don't match");
            curr.ids.push(frame.id);
        }
        None => {
            let (content_type, terse) = match frame.mime_type {
                MimeType::TextPlain => {
                    let terse = String::from_utf8_lossy(content)
                        .chars()
                        .take(100)
                        .collect::<String>();
                    let content_type = if is_valid_https_url(content) {
                        "Link".to_string()
                    } else {
                        "Text".to_string()
                    };
                    (content_type, terse)
                }
                MimeType::ImagePng => {
                    let terse = frame
                        .source
                        .clone()
                        .unwrap_or_else(|| "an image".to_string());
                    ("Image".to_string(), terse)
                }
            };
            stack.insert(
                frame.hash.clone(),
                Item {
                    hash: frame.hash.clone(),
                    ids: vec![frame.id],
                    mime_type: frame.mime_type.clone(),
                    terse,
                    stack: HashMap::new(),
                    content_type,
                },
            );
        }
    };
}

pub struct Stack {
    pub items: HashMap<ssri::Integrity, Item>,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    /*
     * TO PORT:
     *


        pub fn add_frame(&mut self, frame: &xs_lib::Frame) {
            match &frame.topic {
                Some(topic) if topic == "clipboard" => {
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
    */

    fn find_item_by_id(&mut self, id: &scru128::Scru128Id) -> Option<Item> {
        for item in self.items.values() {
            if item.ids.contains(id) {
                return Some(item.clone());
            }
        }
        None
    }

    pub fn merge(&mut self, frame: &Frame, content: &[u8]) {
        if let Some(source_id) = frame.source_id {
            if let Some(mut source) = self.find_item_by_id(&source_id) {
                source.ids.push(frame.id);
                source.content_type = "Stack".into();
                merge_into(&mut source.stack, frame, content);
                self.items.insert(source.hash.clone(), source);
            }
        } else {
            merge_into(&mut self.items, frame, content);
        }
    }
}

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
