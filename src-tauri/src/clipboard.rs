use std::path::Path;

use tauri::api::process::{Command, CommandEvent};

use crate::xs_lib;

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
