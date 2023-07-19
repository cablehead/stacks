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

                let change_num = clipped["change"].as_i64().unwrap();
                if let Some(skip_change_num) = state.skip_change_num {
                    if change_num == skip_change_num {
                        log::debug!("CLIPBOARD UPDATE: {} SKIP", &change_num);
                        continue;
                    }
                }

                let types = clipped["types"].as_object().unwrap();
                let source = clipped["source"].as_str();
                let source = source.map(|s| s.to_string());

                if types.contains_key("public.utf8-plain-text") {
                    let content = b64decode(types["public.utf8-plain-text"].as_str().unwrap());
                    state.add_content(source, None, MimeType::TextPlain, &content);
                    app.emit_all("refresh-items", true).unwrap();
                } else if types.contains_key("public.png") {
                    let content = b64decode(types["public.png"].as_str().unwrap());
                    state.add_content(source, None, MimeType::ImagePng, &content);
                    app.emit_all("refresh-items", true).unwrap();
                }
            }
        }
    });
}
