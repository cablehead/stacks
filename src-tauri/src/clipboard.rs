use std::io::Write;
use std::path::PathBuf;

use tauri::api::process::{Command, CommandEvent};

pub fn start(path: &PathBuf) {
    let path = path.clone().into_os_string().into_string().unwrap();
    let (mut rx, _child) = Command::new_sidecar("x-macos-pasteboard")
        .unwrap()
        .spawn()
        .unwrap();

    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            let path = path.clone();
            if let CommandEvent::Stdout(line) = event {
                let mut child = std::process::Command::new("xs")
                    .arg(path)
                    .arg("put")
                    .arg("--topic")
                    .arg("clipboard")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .unwrap();
                if let Some(ref mut stdin) = child.stdin {
                    stdin.write_all(line.as_bytes()).unwrap();
                }
                let _ = child.wait();
            }
        }
    });
}
