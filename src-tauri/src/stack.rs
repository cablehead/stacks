use std::collections::HashMap;

use crate::store::{Frame, MimeType, Store};

#[derive(Debug, Clone, serde::Serialize)]
pub struct Item {
    hash: ssri::Integrity,
    pub ids: Vec<scru128::Scru128Id>,
    mime_type: MimeType,
    pub content_type: String,
    pub terse: String,
    pub stack: HashMap<String, Item>,
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

    fn create_or_merge(&mut self, store: &Store, frame: &Frame) {
        match self.items.get_mut(&frame.hash) {
            Some(curr) => {
                assert_eq!(curr.mime_type, frame.mime_type, "Mime types don't match");
                curr.ids.push(frame.id);
            }
            None => {
                let (content_type, terse) = match frame.mime_type {
                    MimeType::TextPlain => {
                        let content = store.cat(&frame.hash).unwrap_or_else(Vec::new);
                        let terse = String::from_utf8_lossy(&content)
                            .chars()
                            .take(100)
                            .collect::<String>();
                        let content_type = if is_valid_https_url(&content) {
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
                self.items.insert(
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
