// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::GlobalShortcutManager;
use tauri::Manager;
use tauri::Window;

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

    // let should_continue = Arc::clone(&should_continue);

    window.on_window_event(
        /*
        match event {
         WindowEvent::CloseRequested { .. } | WindowEvent::Destroyed => {
             window_is_open_clone.store(false, Ordering::SeqCst);
         }
         _ => {}
        }
         */
        move |event| println!("EVENT: {:?}", event),
    );

    std::thread::spawn(move || loop {
        if !should_continue.load(Ordering::SeqCst) {
            println!("Window closed, ending thread.");
            break;
        }

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

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use flume::{unbounded, Receiver, Sender};

    struct Producer {
        data: Arc<Mutex<Vec<String>>>,
        senders: Arc<Mutex<Vec<Sender<String>>>>,
    }

    impl Producer {
        fn new() -> Self {
            Producer {
                data: Arc::new(Mutex::new(Vec::new())),
                senders: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn add_consumer(&self) -> (Vec<String>, Receiver<String>) {
            let receiver = {
                let (sender, receiver) = unbounded();
                let mut senders = self.senders.lock().unwrap();
                senders.push(sender);
                receiver
            };

            let data = {
                let data = self.data.lock().unwrap();
                data.clone()
            };

            (data, receiver)
        }

        fn send_data(&self, line: String) {
            self.data.lock().unwrap().push(line.clone());

            // Get a copy of the current senders
            let senders = self.senders.lock().unwrap().clone();

            for sender in &senders {
                sender
                    .send(line.clone())
                    .expect("Failed to send data to consumer");
            }
        }

        fn run<I>(&self, iterator: I)
        where
            I: IntoIterator<Item = String>,
        {
            for line in iterator {
                self.send_data(line);
            }
            let mut senders = self.senders.lock().unwrap();
            senders.clear();
        }
    }

    #[test]
    fn test_data_sent_to_consumer() {
        let producer = Arc::new(Producer::new());

        let (initial_data, receiver) = producer.add_consumer();
        // Check that the initial data is empty (since the producer hasn't sent anything yet)
        assert!(initial_data.is_empty());

        // Data to send
        let (sender, iterator_receiver) = unbounded();

        // Start a new thread for the producer to run the command
        let producer_clone = Arc::clone(&producer);
        let producer_thread = std::thread::spawn(move || {
            producer_clone.run(iterator_receiver.into_iter());
        });

        // Send the first two data
        sender.send("Hello, World!".to_string()).unwrap();
        sender.send("Another string".to_string()).unwrap();

        // Check that the receiver receives the correct data
        for _ in 0..2 {
            match receiver.recv() {
                Ok(data) => assert!(["Hello, World!", "Another string"].contains(&data.as_str())),
                Err(_) => panic!("Failed to receive data"),
            }
        }

        // Add another consumer
        let (initial_data, receiver2) = producer.add_consumer();

        // Check that the initial data contains the first two data
        assert_eq!(
            initial_data,
            vec!["Hello, World!".to_string(), "Another string".to_string(),]
        );

        // Send the next two data
        sender.send("And another one".to_string()).unwrap();
        sender.send("Final string".to_string()).unwrap();

        // Check that both receivers receive the correct data
        for _ in 0..2 {
            match receiver.recv() {
                Ok(data) => assert!(["And another one", "Final string"].contains(&data.as_str())),
                Err(_) => panic!("Failed to receive data"),
            }

            match receiver2.recv() {
                Ok(data) => assert!(["And another one", "Final string"].contains(&data.as_str())),
                Err(_) => panic!("Failed to receive data"),
            }
        }

        drop(sender);
        producer_thread.join().unwrap();

        // Check for RecvError (producer stopped)
        match receiver.recv() {
            Err(_) => {}
            _ => panic!("Expected RecvError"),
        }

        match receiver2.recv() {
            Err(_) => {}
            _ => panic!("Expected RecvError"),
        }
    }
}
