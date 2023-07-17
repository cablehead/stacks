use tauri::api::process::{Command, CommandEvent};
use tauri::Manager;

use serde_json::Value;

use crate::state::SharedState;
use crate::store::MimeType;

use base64::{engine::general_purpose, Engine as _};

fn b64decode(encoded: &str) -> Vec<u8> {
    general_purpose::STANDARD.decode(encoded).unwrap()
}

pub fn start(app: tauri::AppHandle, state: &SharedState) {
    let (mut rx, _child) = Command::new_sidecar("x-macos-pasteboard")
        .unwrap()
        .spawn()
        .unwrap();

    let state = state.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            if let CommandEvent::Stdout(line) = event {
                let mut state = state.lock().unwrap();

                let clipped: Value = serde_json::from_str(&line).unwrap();
                let types = clipped["types"].as_object().unwrap();
                let source = clipped["source"].as_str();
                let source = source.map(|s| s.to_string());

                if types.contains_key("public.utf8-plain-text") {
                    let content = b64decode(types["public.utf8-plain-text"].as_str().unwrap());
                    let frame = state
                        .store
                        .put(source.clone(), MimeType::TextPlain, &content);
                    state.stack.merge(&frame, &content);
                    app.emit_all("refresh-items", true).unwrap();
                } else if types.contains_key("public.png") {
                    let content = b64decode(types["public.png"].as_str().unwrap());
                    let frame = state
                        .store
                        .put(source.clone(), MimeType::ImagePng, &content);
                    state.stack.merge(&frame, &content);
                    app.emit_all("refresh-items", true).unwrap();
                }
            }
        }
    });
}
