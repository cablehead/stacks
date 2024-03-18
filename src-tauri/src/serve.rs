use std::sync::Arc;

use tauri::CustomMenuItem;
use tauri::Manager;
use tauri::SystemTray;
use tauri::SystemTrayMenu;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::clipboard;
use crate::commands;
use crate::content_bus;
use crate::http;
use crate::publish;
use crate::spotlight;
use crate::state::{SharedState, State};

pub async fn serve<A: tauri::Assets>(context: tauri::Context<A>, db_path: String) {
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
                    .unwrap_or(spotlight::Shortcut {
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
