// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::GlobalShortcutManager;
use tauri::Manager;
use tauri::Window;

#[tauri::command]
fn js_log(message: String) {
    println!("[JS]: {}", message);
}

#[tauri::command]
fn init_process(window: Window) {
    std::thread::spawn(move || loop {
        window
            .emit(
                "event-name",
                Payload {
                    message: "Tauri is awesome!".into(),
                },
            )
            .unwrap();

        std::thread::sleep(std::time::Duration::from_secs(1));
    });
}

#[derive(Clone, serde::Serialize)]
struct Payload {
    message: String,
}

// Next:
// https://betterprogramming.pub/front-end-back-end-communication-in-tauri-implementing-progress-bars-and-interrupt-buttons-2a4efd967059

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![js_log, init_process])
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::Focused(focused) => {
                if !focused {
                    event.window().hide().unwrap();
                }
            }
            _ => {}
        })
        .setup(|app| {
            app.emit_all(
                "event-name",
                Payload {
                    message: "Tauri is awesome!".into(),
                },
            )
            .unwrap();
            println!("setup");

            let win = app.get_window("main").unwrap();
            let mut shortcut = app.global_shortcut_manager();
            shortcut
                .register("Cmd+Shift+G", move || {
                    if win.is_visible().unwrap() {
                        win.hide().unwrap();
                    } else {
                        win.show().unwrap();
                        win.set_focus().unwrap();
                    }
                })
                .unwrap_or_else(|err| println!("{:?}", err));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
