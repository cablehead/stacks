use std::collections::HashMap;

use crate::store::{DeleteFrame, Frame, MimeType};

#[derive(Debug, Clone, serde::Serialize)]
pub struct Item {
    pub hash: ssri::Integrity,
    pub ids: Vec<scru128::Scru128Id>,
    pub mime_type: MimeType,
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

    pub fn merge(&mut self, frame: &Frame, content: &[u8]) {
        if let Some(stack_hash) = &frame.stack_hash {
            if let Some(mut stack) = self.items.get_mut(&stack_hash) {
                stack.content_type = "Stack".into();
                merge_into(&mut stack.stack, frame, content);
            }
        } else {
            merge_into(&mut self.items, frame, content);
        }
    }

    pub fn merge_delete(&mut self, frame: &DeleteFrame) {
        match &frame.stack_hash {
            Some(stack_hash) => {
                if let Some(item) = self.items.get_mut(&stack_hash) {
                    item.stack.remove(&frame.hash);
                }
            }
            None => {
                self.items.remove(&frame.hash);
            }
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
