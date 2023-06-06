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
                let path = path.clone();
                let env = xs_lib::store_open(&path).unwrap();
                log::info!(
                    "{}",
                    xs_lib::store_put(&env, Some("clipboard".into()), None, line).unwrap()
                );
            }
        }
    });
}
