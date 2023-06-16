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
    app.emit_all("recent-items", recent_items()).unwrap();
}

#[tauri::command]
async fn store_set_filter(curr: String) -> Vec<Item> {
    println!("FILTER : {}", &curr);
    let mut state = STORE.lock().unwrap();
    state.filter = if curr.is_empty() { None } else { Some(curr) };
    drop(state);
    recent_items()
}

#[tauri::command]
fn init_window() -> Vec<Item> {
    recent_items()
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
    filter: Option<String>,
}

impl Store {
    fn new() -> Self {
        Self {
            items: HashMap::new(),
            cas: HashMap::new(),
            filter: None,
        }
    }

    fn add_frame(&mut self, frame: &xs_lib::Frame) {
        let result: Option<(&str, String, Vec<u8>)> = match &frame.topic {
            Some(topic) if topic == "clipboard" => {
                let clipped: Value = serde_json::from_str(&frame.data).unwrap();
                let types = clipped["types"].as_object().unwrap();

                if types.contains_key("public.utf8-plain-text") {
                    #[allow(deprecated)]
                    let content =
                        decode(types["public.utf8-plain-text"].as_str().unwrap()).unwrap();
                    Some((
                        "text/plain",
                        String::from_utf8(content.clone())
                            .unwrap()
                            .chars()
                            .take(100)
                            .collect(),
                        content,
                    ))
                } else if types.contains_key("public.png") {
                    let content = types["public.png"].as_str().unwrap().as_bytes();
                    Some(("image/png", clipped["source"].to_string(), content.to_vec()))
                } else {
                    println!(
                        "add_frame TODO: id: {}, types: {:?}, frame.data size: {}",
                        frame.id,
                        types.keys().collect::<Vec<_>>(),
                        frame.data.len()
                    );
                    None
                }
            }

            /*
            Some(topic) if topic == "microlink" => {
                let data: Value = serde_json::from_str(&frame.data).unwrap();
                if let Some(link) = process_microlink_frame(&data) {
                    let hash = format!("{:x}", Sha256::digest(&link.url));
                    let mut item = self.items.get_mut(&hash).unwrap();
                    item.link = Some(link);
                    item.ids.push(frame.id);
                    item.content_type = "Link".to_string();
                }
                None
            }
            */
            Some(topic) => {
                println!("topic: {}", topic);
                Some((
                    "text/plain",
                    frame.data.chars().take(100).collect(),
                    frame.data.as_bytes().to_vec(),
                ))
            }
            None => None,
        };

        if let Some((mime_type, terse, content)) = result {
            let hash = format!("{:x}", Sha256::digest(&content));

            match self.items.get_mut(&hash) {
                Some(curr) => {
                    assert_eq!(curr.mime_type, mime_type, "Mime types don't match");
                    curr.ids.push(frame.id);
                }
                None => {
                    self.items.insert(
                        hash.clone(),
                        Item {
                            hash: hash.clone(),
                            ids: vec![frame.id],
                            mime_type: mime_type.to_string(),
                            terse,
                            link: None,
                            content_type: if mime_type == "image/png" {
                                "Image"
                            } else {
                                if is_valid_https_url(&content) {
                                    "Link"
                                } else {
                                    "Text"
                                }
                            }
                            .to_string(),
                        },
                    );
                    self.cas.insert(hash, content);
                }
            }
        }
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
}

fn recent_items() -> Vec<Item> {
    let store = &STORE.lock().unwrap();

    let mut recent_items: Vec<Item> = store
        .items
        .values()
        .filter(|item| {
            if let Some(curr) = &store.filter {
                item.mime_type == "text/plain" && item.terse.contains(curr)
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
                    app.emit_all("recent-items", recent_items())?;
                }
                Ok(())
            })();

            if let Err(e) = pump {
                log::error!("Error processing frames: {}", e);
            }

            if counter % 1000 == 0 {
                log::info!("start_child_process::last_id: {:?}", last_id);
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
            store_set_filter,
            store_get_content,
            store_delete,
            init_window,
            microlink_screenshot,
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

/*
fn process_microlink_frame(data: &Value) -> Option<Link> {
    if !data["original_url"].is_string() {
        return None;
    }
    let title = data["title"].as_str().unwrap();
    let ex = regex::Regex::new(r"[^a-zA-Z0-9\s]").unwrap();
    let title = ex.split(title).next().unwrap().trim();
    Some(Link {
        provider: "microlink".to_string(),
        screenshot: data["screenshot"]["url"].as_str().unwrap().to_string(),
        title: title.to_string(),
        description: data["description"].as_str().unwrap().to_string(),
        url: data["original_url"].as_str().unwrap().to_string(),
        icon: data["logo"]["url"].as_str().unwrap().to_string(),
    })
}
*/

#[tauri::command]
async fn microlink_screenshot(app: tauri::AppHandle, url: String) -> Option<String> {
    println!("MICROLINK: {}", &url);
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.microlink.io/")
        .query(&[
            ("url", &url),
            ("screenshot", &"".to_string()),
            ("device", &"Macbook Pro 13".to_string()),
        ])
        .send()
        .await
        .unwrap();

    let mut res = response.json::<serde_json::Value>().await.unwrap();
    let data = &mut res["data"];
    data["original_url"] = serde_json::Value::String(url);
    let data = data.to_string();
    println!("RESPONSE: {}", data);
    let data_dir = app.path_resolver().app_data_dir().unwrap();
    let data_dir = data_dir.join("stream");
    let env = xs_lib::store_open(&data_dir).unwrap();
    log::info!(
        "{}",
        xs_lib::store_put(&env, Some("microlink".into()), None, data.clone())
            .map_err(|e| format!("{}", e))
            .unwrap()
    );
    None
}

fn is_valid_https_url(url: &[u8]) -> bool {
    let re = regex::bytes::Regex::new(r"^https://[^\s/$.?#].[^\s]*$").unwrap();
    re.is_match(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    /*
    fn get_test_data() -> Value {
        serde_json::json!({
            "title": "Turns websites into data — Microlink",
            "description": "Enter a URL, receive information...",
            "lang": "en",
            "author": "Microlink HQ",
            "publisher": "Microlink",
            "image": {
                "url": "https://cdn.microlink.io/logo/banner.jpeg",
                "type": "jpg",
                "size": 70184,
                "height": 1009,
                "width": 1686,
                "size_pretty": "70.2 kB"
            },
            "date": "2023-06-08T21:18:42.000Z",
            "url": "https://microlink.io/",
            "logo": {
                "url": "https://cdn.microlink.io/logo/trim.png",
                "type": "png",
                "size": 5050,
                "height": 500,
                "width": 500,
                "size_pretty": "5.05 kB"
            },
            "screenshot": {
                "size_pretty": "564 kB",
                "size": 563621,
                "type": "png",
                "url": "https://iad.microlink.io/ijQWQtfkPE4siur3Drxf38QMa_20sUIDLsVahjndfnErFrwcqygQK-8K6MKP-_E1sD5gqt9zOyMn1zrHDqSC4g.png",
                "width": 2560,
                "height": 1600
            }
        })
    }

    #[test]
    fn test_process_microlink_frame_without_original_url() {
        let data = get_test_data();
        let link = process_microlink_frame(&data);
        assert_eq!(link, None);
    }

    #[test]
    fn test_process_microlink_frame_with_original_url() {
        let mut data = get_test_data();
        data["original_url"] = Value::String("https://microlink.io".to_string());
        let link = process_microlink_frame(&data).unwrap();
        assert_eq!(link.provider, "microlink");
        assert_eq!(link.screenshot, "https://iad.microlink.io/ijQWQtfkPE4siur3Drxf38QMa_20sUIDLsVahjndfnErFrwcqygQK-8K6MKP-_E1sD5gqt9zOyMn1zrHDqSC4g.png");
        assert_eq!(link.title, "Turns websites into data");
        assert_eq!(link.description, "Enter a URL, receive information...");
        assert_eq!(link.url, "https://microlink.io");
        assert_eq!(link.icon, "https://cdn.microlink.io/logo/trim.png");
    }
    */

    #[test]
    fn test_is_valid_https_url() {
        assert!(is_valid_https_url(b"https://www.example.com"));
        assert!(!is_valid_https_url(b"Good afternoon"));
    }
}
