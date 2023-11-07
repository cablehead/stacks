// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![recursion_limit = "512"]

use std::sync::{Arc, Mutex};

use tauri::CustomMenuItem;
use tauri::Manager;
use tauri::SystemTrayMenu;

use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod clipboard;
mod commands;
mod publish;
mod state;
mod store;
mod ui;
mod util;
mod view;

#[cfg(debug_assertions)]
mod http;

#[cfg(test)]
mod ui_tests;

#[cfg(test)]
mod view_tests;

use state::{SharedState, State};

#[tokio::main]
async fn main() {
    let (tx, mut rx) = tokio::sync::broadcast::channel(16);

    tokio::spawn(async move {
        let mut stdout = std::io::stdout();
        while let Ok(entry) = rx.recv().await {
            tracing_stacks::fmt::write_entry(&mut stdout, &entry).unwrap();
        }
    });

    tracing_subscriber::Registry::default()
        .with(tracing_subscriber::EnvFilter::new(
            "trace,sled=info,tao=debug,attohttpc=info,tantivy=warn,want=debug,reqwest=info,hyper=info",
        ))
        .with(tracing_stacks::RootSpanLayer::new(tx, None))
        .init();

    info!("let's go!");

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
        .on_window_event(|event| {
            info!("EVENT: {:?}", event.event());
            if let tauri::WindowEvent::Focused(is_focused) = event.event() {
                let state = event.window().state::<SharedState>();
                let mut state = state.lock().unwrap();
                state.ui.is_visible = *is_focused;
            }
        })
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| {
            if let tauri::SystemTrayEvent::MenuItemClick { id, .. } = event {
                match id.as_str() {
                    "check-updates" => {
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
            commands::store_win_move,
            commands::store_get_content,
            commands::store_get_root,
            commands::store_nav_refresh,
            commands::store_nav_reset,
            commands::store_nav_set_filter,
            commands::store_nav_select,
            commands::store_nav_select_up,
            commands::store_nav_select_down,
            commands::store_nav_select_left,
            commands::store_nav_select_right,
            commands::store_copy_to_clipboard,
            commands::store_delete,
            commands::store_undo,
            commands::store_new_note,
            commands::store_edit_note,
            commands::store_move_up,
            commands::store_touch,
            commands::store_move_down,
            commands::store_stack_lock,
            commands::store_stack_unlock,
            commands::store_stack_sort_auto,
            commands::store_stack_sort_manual,
            commands::store_settings_save,
            commands::store_settings_get,
            commands::store_set_theme_mode,
            commands::store_pipe_to_command,
            commands::store_set_content_type,
            commands::store_pipe_to_gpt,
            commands::store_add_to_stack,
            commands::store_add_to_new_stack,
            commands::store_new_stack,
            commands::store_mark_as_cross_stream,
        ])
        .plugin(tauri_plugin_spotlight::init(Some(
            tauri_plugin_spotlight::PluginConfig {
                windows: Some(vec![tauri_plugin_spotlight::WindowConfig {
                    label: String::from("main"),
                    shortcut: (if std::env::var("STACK_DEVTOOLS").is_ok() {
                        "Option+Space"
                    } else {
                        "Control+Space"
                    })
                    .to_string(),
                    macos_window_level: Some(20), // Default 24
                }]),
                global_close_shortcut: None,
            },
        )))
        .setup(|app| {
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            #[cfg(debug_assertions)]
            if std::env::var("STACK_DEVTOOLS").is_ok() {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
                use tauri_plugin_positioner::{Position, WindowExt};
                let _ = window.move_window(Position::Center);
            }

            let db_path = match std::env::var("STACK_DB_PATH") {
                Ok(path) => path,
                Err(_) => {
                    let data_dir = app.path_resolver().app_data_dir().unwrap();
                    data_dir.join("store-v3.0").to_str().unwrap().to_string()
                }
            };
            info!("PR: {:?}", db_path);

            let (packet_sender, packet_receiver) = std::sync::mpsc::channel();

            let state = State::new(&db_path, packet_sender);
            let state: SharedState = Arc::new(Mutex::new(state));
            app.manage(state.clone());

            publish::publish_previews(state.clone(), packet_receiver);

            // start HTTP api if in debug mode
            #[cfg(debug_assertions)]
            {
                http::start(app.handle().clone(), state.clone());
            }

            clipboard::start(app.handle(), &state);

            Ok(())
        })
        .run(context)
        .expect("error while running tauri application");
}
