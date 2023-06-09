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
async fn store_delete(app: tauri::AppHandle, hash: String) -> Vec<Item> {
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
    recent_items()
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
                    println!("add_frame TODO: types: {:?}", types);
                    None
                }
            }

            Some(topic) if topic == "microlink" => {
                let data: Value = serde_json::from_str(&frame.data).unwrap();
                let url = data["url"].as_str().unwrap();
                let hash = format!("{:x}", Sha256::digest(&"https://microlink.io"));
                let mut item = self.items.get_mut(&hash).unwrap();

                item.link = Some(Link {
                    provider: "microlink".to_string(),
                    screenshot: data["screenshot"]["url"].as_str().unwrap().to_string(),
                    title: "".to_string(),
                    description: "".to_string(),
                });

                println!("topic: {} {:?} {:?}", topic, hash, item);
                None
            }

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

#[derive(Debug, Clone, serde::Serialize)]
struct Link {
    provider: String,
    screenshot: String,
    title: String,
    description: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct Item {
    hash: String,
    ids: Vec<scru128::Scru128Id>,
    mime_type: String,
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
