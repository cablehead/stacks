use std::collections::HashMap;


#[derive(Debug, Clone, serde::Serialize)]
pub struct Item {
    hash: String,
    pub ids: Vec<scru128::Scru128Id>,
    mime_type: String,
    pub content_type: String,
    pub terse: String,
    pub stack: HashMap<String, Item>,
}

pub struct Stack {
    pub items: HashMap<String, Item>,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    /*
     * Todo:
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
                        stack: HashMap::new(),
                        content_type: content_type.to_string(),
                    },
                );
                // TODO: self.cas.insert(hash.clone(), content);
            }
        };

        hash
    }
    */

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
