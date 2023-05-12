// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::GlobalShortcutManager;
use tauri::Manager;
use tauri::Window;

use std::io::{self, BufRead};
use std::path::PathBuf;

use clap::Parser;

mod producer;

#[tauri::command]
fn js_log(message: String) {
    println!("[JS]: {}", message);
}

#[tauri::command]
fn add_item(item: String) {
    // Add your logic to add the new item
    // For example, you can send the new item to the child process
    println!("New item: {}", item);

    // Run the child process: xs <path> put --topic dn
    let mut child = std::process::Command::new("xs")
        .arg(&ARGS.path)
        .arg("put")
        .arg("--topic")
        .arg("dn")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to execute command");

    // Write the item to the child process's stdin
    if let Some(ref mut stdin) = child.stdin {
        use std::io::Write;
        stdin
            .write_all(item.as_bytes())
            .expect("Failed to write to stdin");
        stdin.flush().expect("Failed to flush stdin");
    }

    // Wait for the child process to finish
    let output = child.wait_with_output().expect("Failed to wait on child");

    println!("Output: {:?}", output);
}

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref ARGS: Args = Args::parse();
    static ref PROCESS_MAP: std::sync::Mutex<HashMap<String, Arc<AtomicBool>>> =
        std::sync::Mutex::new(HashMap::new());
    static ref PRODUCER: producer::Producer = producer::Producer::new();
}

fn start_child_process(path: &PathBuf) {
    let path = path.clone();
    std::thread::spawn(|| {
        let mut child = std::process::Command::new("xs")
            .arg(path)
            .arg("cat")
            .arg("-f")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to execute command");

        if let Some(ref mut stdout) = child.stdout {
            let reader = io::BufReader::new(stdout);
            for line in reader.lines() {
                let line = line.unwrap();
                PRODUCER.send_data(line);
            }
        }

        // Wait for the subprocess to finish
        let _ = child.wait();
    });
}

#[tauri::command]
fn init_process(window: Window) -> Result<Vec<String>, String> {
    let label = window.label().to_string();
    println!("WINDOW: {:?}", label);

    // If there's an existing process for this window, stop it
    let mut process_map = PROCESS_MAP.lock().unwrap();

    if let Some(should_continue) = process_map.get(&label) {
        should_continue.store(false, Ordering::SeqCst);
    } else {
        // only setup an event listener the first time we see this window
        window.on_window_event(move |event| println!("EVENT: {:?}", event));
    }

    let should_continue = Arc::new(AtomicBool::new(true));
    process_map.insert(label.clone(), should_continue.clone());
    drop(process_map); // Explicitly drop the lock

    let (initial_data, consumer) = PRODUCER.add_consumer();

    std::thread::spawn(move || {
        for line in consumer.iter() {
            if !should_continue.load(Ordering::SeqCst) {
                println!("Window closed, ending thread.");
                break;
            }

            window.emit("item", Payload { message: line }).unwrap();
        }
    });

    Ok(initial_data)
}

#[derive(Clone, serde::Serialize)]
struct Payload {
    message: String,
}

fn validate_and_create_path(s: &str) -> Result<PathBuf, String> {
    let path_string = shellexpand::tilde(s).into_owned();
    let path = PathBuf::from(&path_string);

    if !path.exists() {
        std::fs::create_dir_all(&path)
            .map_err(|_| format!("Failed to create directory at `{}`", s))?;
    }

    Ok(path)
}

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to stream
    #[clap(default_value = "~/.config/stacks/stream", value_parser = validate_and_create_path)]
    path: PathBuf,
}

fn main() {
    start_child_process(&ARGS.path);

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![js_log, init_process, add_item])
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::Focused(focused) => {
                if !focused {
                    event.window().hide().unwrap();
                }
            }
            _ => {}
        })
        .setup(move |app| {
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
