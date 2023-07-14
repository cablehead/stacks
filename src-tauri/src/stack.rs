use std::collections::HashMap;

use crate::store::{Frame, MimeType};

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

    fn create_or_merge(&mut self, frame: &Frame) {
        match self.items.get_mut(&frame.hash) {
            Some(curr) => {
                assert_eq!(curr.mime_type, frame.mime_type, "Mime types don't match");
                curr.ids.push(frame.id);
            }
            None => {
                let content_type = "Text".to_string();
                let terse = "terse".to_string();
                self.items.insert(
                    frame.hash.clone(),
                    Item {
                        hash: frame.hash.clone(),
                        ids: vec![frame.id],
                        mime_type: frame.mime_type.clone(),
                        terse,
                        stack: HashMap::new(),
                        content_type: content_type.to_string(),
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
