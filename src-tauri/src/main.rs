// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;

use tauri::CustomMenuItem;
use tauri::Manager;
use tauri::SystemTrayMenu;
use tauri::Window;
use tauri_plugin_log::LogTarget;

use lazy_static::lazy_static;

mod clipboard;
mod producer;

lazy_static! {
    static ref PRODUCER: producer::Producer = producer::Producer::new();
    static ref PROCESS_MAP: Mutex<HashMap<String, Arc<AtomicBool>>> = Mutex::new(HashMap::new());
    static ref DATADIR: Mutex<PathBuf> = Mutex::new(PathBuf::new());
}

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

#[tauri::command]
fn init_process(window: Window) -> Result<Vec<String>, String> {
    let label = window.label().to_string();
    log::info!("WINDOW: {:?}", label);

    // If there's an existing process for this window, stop it
    let mut process_map = PROCESS_MAP.lock().unwrap();

    if let Some(should_continue) = process_map.get(&label) {
        should_continue.store(false, Ordering::SeqCst);
    } else {
        // only setup an event listener the first time we see this window
        window.on_window_event(move |event| log::info!("EVENT: {:?}", event));
    }

    let should_continue = Arc::new(AtomicBool::new(true));
    process_map.insert(label, should_continue.clone());
    drop(process_map); // Explicitly drop the lock

    let (initial_data, consumer) = PRODUCER.add_consumer();

    std::thread::spawn(move || {
        for line in consumer.iter() {
            if !should_continue.load(Ordering::SeqCst) {
                log::info!("Window closed, ending thread.");
                break;
            }

            window.emit("item", Payload { message: line }).unwrap();
        }
    });

    Ok(initial_data)
}

// POLL_INTERVAL is the number of milliseconds to wait between polls when watching for
// additions to the stream
// todo: investigate switching to: https://docs.rs/notify/latest/notify/
const POLL_INTERVAL: u64 = 5;

fn start_child_process(path: &Path) {
    let path = path.to_path_buf();
    std::thread::spawn(move || {
        let mut last_id = None;
        let mut counter = 0;
        loop {
            let env = xs_lib::store_open(&path).unwrap();
            let frames = xs_lib::store_cat(&env, last_id).unwrap();
            for frame in frames {
                last_id = Some(frame.id);
                let data = serde_json::to_string(&frame).unwrap();
                PRODUCER.send_data(data);
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
        .invoke_handler(tauri::generate_handler![init_process])
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
            let _window = app.get_window("main").unwrap();
            // window.open_devtools();
            // window.close_devtools();

            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let data_dir = app.path_resolver().app_data_dir().unwrap();
            let data_dir = data_dir.join("stream");
            log::info!("PR: {:?}", data_dir);
            let mut shared = DATADIR.lock().unwrap();
            *shared = data_dir;

            clipboard::start(&shared);
            start_child_process(&shared);

            Ok(())
        })
        .run(context)
        .expect("error while running tauri application");
}
