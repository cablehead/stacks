use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{PathBuf};
use std::sync::{Arc, Mutex};
use tauri::Manager;

use crate::xs_lib;


#[tauri::command]
pub fn store_get_content(hash: String, store: tauri::State<SharedStore>) -> Option<String> {
    let store = store.lock().unwrap();
    println!("CACHE MISS: {}", &hash);
    store
        .cas
        .get(&hash)
        .map(|content| String::from_utf8(content.clone()).unwrap())
}

#[tauri::command]
pub fn store_list_stacks(filter: String, store: tauri::State<SharedStore>) -> Vec<Item> {
    let store = store.lock().unwrap();

    let mut ret: Vec<Item> = store
        .items
        .values()
        .filter(|item| {
            if &item.content_type != "Stack" {
                return false;
            }

            return if filter == filter.to_lowercase() {
                item.terse.to_lowercase().contains(&filter)
            } else {
                item.terse.contains(&filter)
            };
        })
        .cloned()
        .collect();
    ret.sort_unstable_by(|a, b| b.ids.last().cmp(&a.ids.last()));
    ret.truncate(400);
    ret
}

#[tauri::command]
pub fn store_delete(app: tauri::AppHandle, hash: String, store: tauri::State<SharedStore>) {
    let mut store = store.lock().unwrap();
    println!("DEL: {}", &hash);
    if let Some(item) = store.items.remove(&hash) {
        println!("item: {:?}", item);
        let env = xs_lib::store_open(&store.db_path).unwrap();
        xs_lib::store_delete(&env, item.ids).unwrap();
    }
    store.cas.remove(&hash);
    app.emit_all("refresh-items", true).unwrap();
}

#[tauri::command]
pub fn store_add_to_stack(name: String, id: String, store: tauri::State<SharedStore>) {
    let store = store.lock().unwrap();
    let data = serde_json::json!({
        "name": name,
        "id": id
    })
    .to_string();
    println!("ADD TO STACK: {}", &data);
    let env = xs_lib::store_open(&store.db_path).unwrap();
    xs_lib::store_put(&env, Some("stack".into()), None, data).unwrap();
}

#[tauri::command]
pub fn store_delete_from_stack(name: String, id: String, store: tauri::State<SharedStore>) {
    let store = store.lock().unwrap();
    let data = serde_json::json!({
        "name": name,
        "id": id
    })
    .to_string();
    println!("DELETE FROM STACK: {}", &data);
    let env = xs_lib::store_open(&store.db_path).unwrap();
    xs_lib::store_put(&env, Some("stack".into()), Some("delete".into()), data).unwrap();
}

// Saves item to the cas
// If source_id is present creates a link to the source
// If stack_name is present, adds item to the stack
// if stack_name and source are present, removes source from stack
#[tauri::command]
pub fn store_capture(
    item: String,
    source_id: Option<String>,
    stack_name: Option<String>,
    store: tauri::State<SharedStore>,
) {
    println!("CAPTURE: {} {:?} {:?}", item, source_id, stack_name);
    let store = store.lock().unwrap();

    let env = xs_lib::store_open(&store.db_path).unwrap();

    let id = xs_lib::store_put(&env, Some("item".into()), None, item).unwrap();

    if let Some(source_id) = &source_id {
        let data = serde_json::json!({
            "source_id": source_id,
            "id": id
        })
        .to_string();
        xs_lib::store_put(&env, Some("link".into()), None, data).unwrap();
    }

    if let Some(stack_name) = stack_name {
        let data = serde_json::json!({
            "name": stack_name,
            "id": id
        })
        .to_string();
        xs_lib::store_put(&env, Some("stack".into()), None, data).unwrap();

        if let Some(source_id) = &source_id {
            let data = serde_json::json!({
                "name": stack_name,
                "id": source_id
            })
            .to_string();
            xs_lib::store_put(&env, Some("stack".into()), Some("delete".into()), data).unwrap();
        }
    }
}

#[tauri::command]
pub fn store_list_items(
    stack: Option<String>,
    filter: String,
    content_type: String,
    store: tauri::State<SharedStore>,
) -> Vec<Item> {
    let store = store.lock().unwrap();
    println!("FILTER : {:?} {} {}", &stack, &filter, &content_type);
    let filter = if filter.is_empty() {
        None
    } else {
        Some(filter)
    };
    let content_type = if content_type == "All" {
        None
    } else {
        let mut content_type = content_type;
        content_type.truncate(content_type.len() - 1);
        Some(content_type)
    };

    let base_items: Vec<Item> = if let Some(hash) = stack {
        let item = store.items.get(&hash).unwrap();
        item.stack.values().cloned().collect()
    } else {
        store.items.values().cloned().collect()
    };

    let mut recent_items: Vec<Item> = base_items
        .iter()
        .filter(|item| {
            if let Some(curr) = &filter {
                // match case insensitive, unless the filter has upper case, in which, match case
                // sensitive
                if curr == &curr.to_lowercase() {
                    item.terse.to_lowercase().contains(curr)
                } else {
                    item.terse.contains(curr)
                }
            } else {
                true
            }
        })
        .filter(|item| {
            if let Some(content_type) = &content_type {
                &item.content_type == content_type
            } else {
                true
            }
        })
        .cloned()
        .collect();
    recent_items.sort_unstable_by(|a, b| b.ids.last().cmp(&a.ids.last()));
    recent_items.truncate(400);
    recent_items
}

pub struct Store {
    items: HashMap<String, Item>,
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
    ids: Vec<scru128::Scru128Id>,
    mime_type: String,
    content_type: String,
    terse: String,
    link: Option<Link>,
    stack: HashMap<String, Item>,
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
