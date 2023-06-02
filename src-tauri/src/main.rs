// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[allow(deprecated)]
use base64::decode;

use lazy_static::lazy_static;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use chrono::{TimeZone, Utc};
use std::cmp::min;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use tauri::CustomMenuItem;
use tauri::Manager;
use tauri::SystemTrayMenu;
use tauri_plugin_log::LogTarget;

mod clipboard;

#[derive(Clone, serde::Serialize)]
struct Payload {
    message: String,
}

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
fn get_item_content(hash: String) -> Option<String> {
    println!("CACHE MISS: {}", &hash);
    let items = ITEMS.lock().unwrap();
    items.get(&hash).map(|item| {
        let content = String::from_utf8(item.content.clone()).unwrap();
        content
    })
}

#[tauri::command]
fn init_window() -> String {
    recent_items()
}

lazy_static! {
    static ref ITEMS: Mutex<HashMap<String, Item>> = Mutex::new(HashMap::new());
}

struct Item {
    mime_type: String,
    terse: String,
    content: Vec<u8>,
    hash: String,
    ids: Vec<scru128::Scru128Id>,
}

impl Item {
    fn new(mime_type: &str, terse: &str, content: &[u8], id: scru128::Scru128Id) -> Self {
        let hash = format!("{:x}", Sha256::digest(content));
        Self {
            mime_type: mime_type.to_string(),
            terse: terse.to_string(),
            content: content.to_vec(),
            hash,
            ids: vec![id],
        }
    }

    fn from_frame(frame: &xs_lib::Frame) -> Option<Self> {
        match &frame.topic {
            Some(topic) if topic == "clipboard" => {
                let clipped: Value = serde_json::from_str(&frame.data).unwrap();
                let types = clipped["types"].as_object().unwrap();

                if types.contains_key("public.utf8-plain-text") {
                    #[allow(deprecated)]
                    let content =
                        decode(types["public.utf8-plain-text"].as_str().unwrap()).unwrap();
                    let terse: String = String::from_utf8(content.clone())
                        .unwrap()
                        .chars()
                        .take(100)
                        .collect();
                    Some(Item::new("text/plain", &terse, &content, frame.id))
                } else if types.contains_key("public.png") {
                    let content = types["public.png"].as_str().unwrap().as_bytes();
                    Some(Item::new(
                        "image/png",
                        clipped["source"].as_str().unwrap(),
                        &content,
                        frame.id,
                    ))
                } else {
                    println!("types: {:?}", types);
                    None
                }
            }
            Some(_) => Some(Item::new(
                "text/plain",
                &frame.data[..min(frame.data.len(), 100)],
                frame.data.as_bytes(),
                frame.id,
            )),
            None => None,
        }
    }
}

fn merge_item(item: Item) {
    let mut items = ITEMS.lock().unwrap();
    match items.get_mut(&item.hash) {
        Some(existing_item) => {
            assert_eq!(
                existing_item.mime_type, item.mime_type,
                "Mime types don't match"
            );
            existing_item.ids.extend(item.ids);
        }
        None => {
            items.insert(item.hash.clone(), item);
        }
    }
}

#[derive(serde::Serialize)]
struct ItemTerse {
    mime_type: String,
    hash: String,
    terse: String,
    meta: Vec<Value>,
}

fn recent_items() -> String {
    let items = ITEMS.lock().unwrap();
    let mut recent_items: Vec<&Item> = items.values().collect();
    recent_items.sort_unstable_by(|a, b| b.ids.last().cmp(&a.ids.last()));
    recent_items.truncate(400);

    let recent_items: Vec<ItemTerse> = recent_items
        .iter()
        .map(|item| {
            let created_at = format_scru128_date(item.ids[0]);
            let updated_at = format_scru128_date(*item.ids.last().unwrap());
            let meta = if item.ids.len() == 1 {
                vec![
                    json!({ "name": "ID", "value": item.ids[0].to_string() }),
                    json!({ "name": "Copied", "value": created_at }),
                ]
            } else {
                vec![
                    json!({ "name": "ID", "value": item.ids[0].to_string() }),
                    json!({ "name": "Times copied", "value": item.ids.len().to_string() }),
                    json!({ "name": "Last Copied", "value": updated_at }),
                    json!({ "name": "First Copied", "value": created_at }),
                ]
            };

            ItemTerse {
                mime_type: item.mime_type.clone(),
                hash: item.hash.clone(),
                terse: item.terse.clone(),
                meta,
            }
        })
        .collect();

    serde_json::to_string(&recent_items).unwrap()
}

fn format_scru128_date(id: scru128::Scru128Id) -> String {
    let timestamp = id.timestamp();
    let datetime = Utc.timestamp(timestamp as i64, 0);
    datetime.format("%a %Y-%b-%d %I:%M %p").to_string()
}

fn start_child_process(app: tauri::AppHandle, path: &Path) {
    let path = path.to_path_buf();
    std::thread::spawn(move || {
        let mut last_id = None;
        let mut counter = 0;
        loop {
            let env = xs_lib::store_open(&path).unwrap();
            let frames = xs_lib::store_cat(&env, last_id).unwrap();

            let mut updated = false;

            for frame in frames {
                last_id = Some(frame.id);
                let item = Item::from_frame(&frame);
                if let Some(item) = item {
                    updated = true;
                    merge_item(item);
                }
            }

            if updated {
                app.emit_all(
                    "recent-items",
                    Payload {
                        message: recent_items(),
                    },
                )
                .unwrap();
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
        .invoke_handler(tauri::generate_handler![init_window, get_item_content])
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
            let window = app.get_window("main").unwrap();
            window.open_devtools();
            window.close_devtools();

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
