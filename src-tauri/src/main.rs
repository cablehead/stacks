// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use tauri::CustomMenuItem;
use tauri::Manager;
use tauri::SystemTrayMenu;
use tauri_plugin_log::LogTarget;

mod clipboard;
mod commands;
mod stack;
mod state;
mod store;

use state::{SharedState, State};

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
            commands::store_list_items,
            // store::store_delete,
            // store::store_capture,
            // store::store_add_to_stack,
            // store::store_delete_from_stack,
            commands::store_get_content,
            commands::store_list_stacks,
            commands::store_copy_to_clipboard,
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
                .level_for("sled", log::LevelFilter::Debug)
                .build(),
        )
        .setup(|app| {
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            if let Ok(_) = std::env::var("STACK_DEVTOOLS") {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
            }

            let db_path = match std::env::var("STACK_DB_PATH") {
                Ok(path) => path,
                Err(_) => {
                    let data_dir = app.path_resolver().app_data_dir().unwrap();
                    data_dir.join("stream").to_str().unwrap().to_string()
                }
            };
            log::info!("PR: {:?}", db_path);

            let state = State::new(&db_path);
            let state: SharedState = Arc::new(Mutex::new(state));
            app.manage(state.clone());

            clipboard::start(app.handle(), &state);

            Ok(())
        })
        .run(context)
        .expect("error while running tauri application");
}
