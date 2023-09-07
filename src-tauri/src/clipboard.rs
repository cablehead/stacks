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
                let _source = source.map(|s| s.to_string());

                let curr_stack = Some(state.get_curr_stack());

                let packet = if types.contains_key("public.utf8-plain-text") {
                    let content = b64decode(types["public.utf8-plain-text"].as_str().unwrap());
                    if let Ok(str_ref) = std::str::from_utf8(&content) {
                        if str_ref.trim().is_empty() {
                            continue;
                        }
                    }

                    Some(
                        state
                            .store
                            .add(&content, MimeType::TextPlain, curr_stack),
                    )
                } else if types.contains_key("public.png") {
                    let content = b64decode(types["public.png"].as_str().unwrap());
                    Some(
                        state
                            .store
                            .add(&content, MimeType::ImagePng, curr_stack),
                    )
                } else {
                    None
                };

                if let Some(packet) = packet {
                    state.merge(packet);
                }

                app.emit_all("refresh-items", true).unwrap();
            }
        }
    });
}
