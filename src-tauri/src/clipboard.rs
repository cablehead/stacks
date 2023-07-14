use std::path::Path;

use tauri::api::process::{Command, CommandEvent};
use tauri::Manager;

use crate::xs_lib;
use crate::store::{SharedStore};

pub fn start(path: &Path) {
    let path = path.to_path_buf();
    let (mut rx, _child) = Command::new_sidecar("x-macos-pasteboard")
        .unwrap()
        .spawn()
        .unwrap();

    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            if let CommandEvent::Stdout(line) = event {
                loop {
                    let path = path.clone();
                    let result = (|| -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                        let env = xs_lib::store_open(&path).map_err(|e| format!("{}", e))?;
                        log::info!(
                            "{}",
                            xs_lib::store_put(&env, Some("clipboard".into()), None, line.clone())
                                .map_err(|e| format!("{}", e))?
                        );
                        Ok(())
                    })();

                    match result {
                        Ok(_) => break,
                        Err(e) => {
                            log::error!("Error: {}", e);
                            // Sleep for 1 second before retrying
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        }
                    }
                }
            }
        }
    });
}

// POLL_INTERVAL is the number of milliseconds to wait between polls when watching for
// additions to the stream
// todo: investigate switching to: https://docs.rs/notify/latest/notify/
const POLL_INTERVAL: u64 = 10;

pub fn start_child_process(app: tauri::AppHandle, path: &Path, store: SharedStore) {
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
                        let mut store = store.lock().unwrap();
                        store.add_frame(&frame);
                    }
                    app.emit_all("refresh-items", true)?;
                }
                Ok(())
            })();

            if let Err(e) = pump {
                log::error!("Error processing frames: {}", e);
            }

            if counter % 1000 == 0 {
                log::info!(
                    "start_child_process::last_id: {}",
                    last_id.map_or(String::from("None"), |id| id.to_string())
                );
            }
            counter += 1;
            std::thread::sleep(std::time::Duration::from_millis(POLL_INTERVAL));
        }
    });
}
