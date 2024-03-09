// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![recursion_limit = "512"]

use std::sync::Arc;

use tauri::CustomMenuItem;
use tauri::Manager;
use tauri::SystemTray;
use tauri::SystemTrayMenu;

use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod clipboard;
mod commands;
mod content_bus;
mod content_type;
mod http;
mod publish;
mod spotlight;
mod state;
mod store;
mod ui;
mod util;
mod view;

use crate::spotlight::Shortcut;

#[cfg(test)]
mod store_tests;

#[cfg(test)]
mod ui_tests;

#[cfg(test)]
mod view_tests;

use state::{SharedState, State};

#[tokio::main]
async fn main() {
    let context = tauri::generate_context!();

    let system_app_data_dir = tauri::api::path::data_dir()
        .unwrap()
        .join(&context.config().tauri.bundle.identifier);

    let db_path = match std::env::var("STACK_DB_PATH") {
        Ok(path) => path,
        Err(_) => {
            let data_dir = system_app_data_dir;
            data_dir.join("store-v3.0").to_str().unwrap().to_string()
        }
    };
    info!(db_path, "let's go");

    if command_name() == "stacks" {
        cli(&db_path).await;
    } else {
        serve(context, db_path).await;
    }
}

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the store
    #[clap(value_parser)]
    id: Option<String>,
}

async fn cli(db_path: &str) {
    // use std::os::unix::net::UnixStream;
    use std::path::Path;

    use hyper_util::rt::TokioIo;

    let args = Args::parse();

    let socket_path = Path::new(db_path).join("sock");
    let stream = tokio::net::UnixStream::connect(socket_path)
        .await
        .expect("Failed to connect to server");
    let io = TokioIo::new(stream);

    use bytes::Bytes;
    use http_body_util::Empty;
    use hyper::client::conn;
    use hyper::{Request, StatusCode};

    let (mut request_sender, connection) = conn::http1::handshake(io).await.unwrap();

    // spawn a task to poll the connection and drive the HTTP state
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Error in connection: {}", e);
        }
    });

    let request = Request::builder()
        // We need to manually add the host header because SendRequest does not
        .header("Host", "example.com")
        .method("GET")
        .body(Empty::<Bytes>::new())
        .unwrap();

    let response = request_sender.send_request(request).await.unwrap();
    assert!(response.status() == StatusCode::OK);

    eprintln!("{:?} {:?} {:?}", db_path, args, response);
}

async fn serve<A: tauri::Assets>(context: tauri::Context<A>, db_path: String) {
    init_tracing();

    let config = context.config();
    let version = &config.package.version.clone().unwrap();

    tauri::Builder::default()
        .on_window_event(|event| {
            let span = tracing::info_span!("on_window_event", "{:?}", event.event());
            span.in_scope(|| {
                if let tauri::WindowEvent::Focused(is_focused) = event.event() {
                    let state = event.window().state::<SharedState>();
                    state.with_lock(|state| {
                        state.ui.is_visible = *is_focused;
                    });
                }
            });
        })
        .system_tray(system_tray(version))
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
            commands::store_get_raw_content,
            commands::store_get_root,
            commands::store_nav_refresh,
            commands::store_nav_reset,
            commands::store_nav_set_filter,
            commands::store_nav_select,
            commands::store_nav_select_up,
            commands::store_nav_select_down,
            commands::store_nav_select_up_stack,
            commands::store_nav_select_down_stack,
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
            commands::store_pipe_stack_to_shell,
            commands::store_set_content_type,
            commands::store_add_to_stack,
            commands::store_add_to_new_stack,
            commands::store_new_stack,
            commands::store_mark_as_cross_stream,
            commands::spotlight_update_shortcut,
            commands::spotlight_get_shortcut,
            commands::spotlight_hide,
        ])
        .setup(move |app| {
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let window = app.get_window("main").unwrap();

            #[cfg(debug_assertions)]
            if std::env::var("STACK_DEVTOOLS").is_ok() {
                window.open_devtools();
                use tauri_plugin_positioner::{Position, WindowExt};
                let _ = window.move_window(Position::Center);
            }

            let (packet_sender, packet_receiver) = std::sync::mpsc::channel();

            let state = State::new(&db_path, packet_sender);
            let mutex = tracing_mutex_span::TracingMutexSpan::new("SharedState", state);
            let state: SharedState = Arc::new(mutex);
            app.manage(state.clone());

            publish::spawn(state.clone(), packet_receiver);
            content_bus::spawn_tiktokens(app.handle(), state.clone());

            http::start(app.handle().clone(), state.clone(), &db_path);
            clipboard::start(app.handle(), &state);

            let shortcut = state.with_lock(|state| {
                let settings = state.store.settings_get();
                settings
                    .and_then(|s| s.activation_shortcut)
                    .unwrap_or(Shortcut {
                        ctrl: true,
                        shift: false,
                        alt: false,
                        command: false,
                    })
            });
            spotlight::init(&window).unwrap();
            spotlight::register_shortcut(&window, &shortcut.to_macos_shortcut()).unwrap();

            Ok(())
        })
        .run(context)
        .expect("error while running tauri application");
}

fn init_tracing() {
    let (tx, mut rx) = tokio::sync::broadcast::channel(1000);

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
}

fn system_tray(version: &str) -> SystemTray {
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
    tauri::SystemTray::new().with_menu(menu)
}

fn command_name() -> String {
    std::env::args()
        .nth(0)
        .map(|arg| {
            std::path::Path::new(&arg)
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or("")
                .to_string()
        })
        .unwrap()
}
