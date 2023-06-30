// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[allow(deprecated)]
use base64::decode;

use lazy_static::lazy_static;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use tauri::CustomMenuItem;
use tauri::Manager;
use tauri::SystemTrayMenu;
use tauri_plugin_log::LogTarget;

mod clipboard;
mod xs_lib;

#[derive(Clone, serde::Serialize)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

// POLL_INTERVAL is the number of milliseconds to wait between polls when watching for
// additions to the stream
// todo: investigate switching to: https://docs.rs/notify/latest/notify/
const POLL_INTERVAL: u64 = 10;

#[tauri::command]
async fn store_get_content(hash: String) -> Option<String> {
    println!("CACHE MISS: {}", &hash);
    let state = STORE.lock().unwrap();
    state
        .cas
        .get(&hash)
        .map(|content| String::from_utf8(content.clone()).unwrap())
}

#[tauri::command]
async fn store_list_stacks(filter: String) -> Vec<Item> {
    let store = &STORE.lock().unwrap();
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
async fn store_delete(app: tauri::AppHandle, hash: String) {
    println!("DEL: {}", &hash);
    let mut state = STORE.lock().unwrap();
    if let Some(item) = state.items.remove(&hash) {
        println!("item: {:?}", item);
        let data_dir = app.path_resolver().app_data_dir().unwrap();
        let data_dir = data_dir.join("stream");
        let env = xs_lib::store_open(&data_dir).unwrap();
        xs_lib::store_delete(&env, item.ids).unwrap();
    }
    state.cas.remove(&hash);
    drop(state);
    app.emit_all("refresh-items", true).unwrap();
}

#[tauri::command]
async fn store_list_items(
    stack: Option<String>,
    filter: String,
    content_type: String,
) -> Vec<Item> {
    println!("FILTER : {:?} {} {}", &stack, &filter, &content_type);
    let store = &STORE.lock().unwrap();

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

    let base_items = if let Some(hash) = stack {
        let item = store.items.get(&hash).unwrap();
        item.stack.clone()
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

#[tauri::command]
async fn open_docs(handle: tauri::AppHandle) {
    let _ = tauri::WindowBuilder::new(
        &handle,
        "external", /* the unique window label */
        tauri::WindowUrl::App("second.html".into()),
    )
    .build()
    .unwrap();
}

struct Store {
    items: HashMap<String, Item>,
    cas: HashMap<String, Vec<u8>>,
}

impl Store {
    fn new() -> Self {
        Self {
            items: HashMap::new(),
            cas: HashMap::new(),
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
                        stack: Vec::new(),
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

    fn add_frame(&mut self, frame: &xs_lib::Frame) {
        match &frame.topic {
            Some(topic) if topic == "clipboard" => {
                let clipped: Value = serde_json::from_str(&frame.data).unwrap();
                let types = clipped["types"].as_object().unwrap();

                if types.contains_key("public.utf8-plain-text") {
                    #[allow(deprecated)]
                    let content =
                        decode(types["public.utf8-plain-text"].as_str().unwrap()).unwrap();

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
                println!("topic: {} {:?}", topic, data);

                let id = data["id"].as_str();
                if let None = id {
                    return;
                }
                let id = id.unwrap();

                let target = self.find_item_by_id(id);
                if let None = target {
                    return;
                }
                let target = target.unwrap();

                let content = data["name"].as_str().unwrap();
                let hash = self.create_or_merge(
                    frame.id,
                    "text/plain",
                    "Stack",
                    content.chars().take(100).collect(),
                    content.as_bytes().to_vec(),
                );

                let item = self.items.get_mut(&hash).unwrap();
                item.content_type = "Stack".to_string();
                item.stack.push(target);
            }

            Some(_) => {
                let _ = self.create_or_merge(
                    frame.id,
                    "text/plain",
                    "Text",
                    frame.data.chars().take(100).collect(),
                    frame.data.as_bytes().to_vec(),
                );
            }
            None => (),
        };
    }
}

lazy_static! {
    static ref STORE: Mutex<Store> = Mutex::new(Store::new());
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
struct Item {
    hash: String,
    ids: Vec<scru128::Scru128Id>,
    mime_type: String,
    content_type: String,
    terse: String,
    link: Option<Link>,
    stack: Vec<Item>,
}

fn start_child_process(app: tauri::AppHandle, path: &Path) {
    let path = path.to_path_buf();
    std::thread::spawn(move || {
        let mut last_id = None;
        let mut counter = 0;
        loop {
            let pump = (|| -> Result<(), Box<dyn std::error::Error>> {
                let env = xs_lib::store_open(&path)?;
                let frames = xs_lib::store_cat(&env, last_id)?;
                if !frames.is_empty() {
                    for frame in frames {
                        last_id = Some(frame.id);
                        let mut state = STORE.lock()?;
                        state.add_frame(&frame);
                    }
                    app.emit_all("refresh-items", true)?;
                }
                Ok(())
            })();

            if let Err(e) = pump {
                log::error!("Error processing frames: {}", e);
            }

            if counter % 1000 == 0 {
                log::info!(
                    "start_child_process::last_id: {}",
                    last_id.map_or(String::from("None"), |id| id.to_string())
                );
            }
            counter += 1;
            std::thread::sleep(std::time::Duration::from_millis(POLL_INTERVAL));
        }
    });
}

fn main() {
    let context = tauri::generate_context!();
    let config = context.config();
    let version = &config.package.version.clone().unwrap();

    let menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("".to_string(), "Stacks").disabled())
        .add_item(CustomMenuItem::new("".to_string(), format!("Version {}", version)).disabled())
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new(
            "check-updates".to_string(),
            "Check for Updates...",
        ))
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit".to_string(), "Quit"));
    let system_tray = tauri::SystemTray::new().with_menu(menu);

    tauri::Builder::default()
        .on_window_event(|event| log::info!("EVENT: {:?}", event.event()))
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| {
            if let tauri::SystemTrayEvent::MenuItemClick { id, .. } = event {
                match id.as_str() {
                    "check-updates" => {
                        println!("update");
                        app.trigger_global("tauri://update", None);
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            store_list_items,
            store_delete,
            store_get_content,
            store_list_stacks,
            open_docs,
        ])
        .plugin(tauri_plugin_spotlight::init(Some(
            tauri_plugin_spotlight::PluginConfig {
                windows: Some(vec![tauri_plugin_spotlight::WindowConfig {
                    label: String::from("main"),
                    shortcut: String::from("Control+Space"),
                    macos_window_level: Some(20), // Default 24
                }]),
                global_close_shortcut: None,
            },
        )))
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets([LogTarget::LogDir, LogTarget::Stdout, LogTarget::Webview])
                .level_for("tao", log::LevelFilter::Debug)
                .build(),
        )
        .setup(|app| {
            #[allow(unused_variables)]
            let window = app.get_window("main").unwrap();
            window.open_devtools();

            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let data_dir = app.path_resolver().app_data_dir().unwrap();
            let data_dir = data_dir.join("stream");
            log::info!("PR: {:?}", data_dir);

            clipboard::start(&data_dir);
            start_child_process(app.handle(), &data_dir);

            Ok(())
        })
        .run(context)
        .expect("error while running tauri application");
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
