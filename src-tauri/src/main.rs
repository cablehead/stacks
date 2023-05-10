// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::GlobalShortcutManager;
use tauri::Manager;
use tauri::Window;

use std::io::{self, BufRead};

mod producer;

#[tauri::command]
fn js_log(message: String) {
    println!("[JS]: {}", message);
}

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref PROCESS_MAP: std::sync::Mutex<HashMap<String, Arc<AtomicBool>>> =
        std::sync::Mutex::new(HashMap::new());
}

lazy_static! {
    static ref PRODUCER: producer::Producer = producer::Producer::new();
}

#[tauri::command]
fn init_process(window: Window) {
    let label = window.label().to_string();
    println!("WINDOW: {:?}", label);

    // If there's an existing process for this window, stop it
    let mut process_map = PROCESS_MAP.lock().unwrap();

    if let Some(should_continue) = process_map.get(&label) {
        should_continue.store(false, Ordering::SeqCst);
    }

    let should_continue = Arc::new(AtomicBool::new(true));
    process_map.insert(label.clone(), should_continue.clone());
    drop(process_map); // Explicitly drop the lock

    window.on_window_event(move |event| println!("EVENT: {:?}", event));

    let (_initial_data, _consumer) = PRODUCER.add_consumer();

    std::thread::spawn(move || {
        // Read lines from stdin and emit events
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            if !should_continue.load(Ordering::SeqCst) {
                println!("Window closed, ending thread.");
                break;
            }

            let line = line.unwrap();
            window
                .emit("event-name", Payload { message: line })
                .unwrap();
        }
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
            match app.get_cli_matches() {
                Ok(matches) => {
                    if let Some(arg_data) = matches.args.get("PATH") {
                        println!("PATH argument: {:?}", arg_data.value);
                    }
                }
                Err(e) => match e {
                    tauri::Error::FailedToExecuteApi(error_message) => {
                        println!("{}", error_message);
                        std::process::exit(1);
                    }
                    _ => panic!("{:?}", e),
                },
            }

            let win = app.get_window("main").unwrap();
            win.open_devtools();
            win.close_devtools();
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
