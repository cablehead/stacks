use tauri::api::process::{Command, CommandEvent};
use tauri::Manager;

use serde_json::Value;

use tracing::info;

use crate::state;
use crate::state::SharedState;
use crate::store::MimeType;
use crate::util;

#[tracing::instrument(skip_all)]
fn handle_clipboard_update(state: &mut state::State, line: &str, app: &tauri::AppHandle) {
    let clipped: Value = serde_json::from_str(line).unwrap();

    let change_num = clipped["change"].as_i64().unwrap();
    if let Some(skip_change_num) = state.skip_change_num {
        if change_num == skip_change_num {
            info!("CLIPBOARD UPDATE: {} SKIP", &change_num);
            return;
        }
    }

    let types = clipped["types"].as_object().unwrap();
    let source = clipped["source"].as_str();
    let _source = source.map(|s| s.to_string());

    let curr_stack = state.get_curr_stack();

    let packet = if types.contains_key("public.utf8-plain-text") {
        let content = util::b64decode(types["public.utf8-plain-text"].as_str().unwrap());
        if let Ok(str_ref) = std::str::from_utf8(&content) {
            if str_ref.trim().is_empty() {
                return;
            }
        }

        Some(state.store.add(&content, MimeType::TextPlain, curr_stack))
    } else if types.contains_key("public.png") {
        let content = util::b64decode(types["public.png"].as_str().unwrap());
        Some(state.store.add(&content, MimeType::ImagePng, curr_stack))
    } else {
        None
    };

    if let Some(packet) = packet {
        state.merge(&packet);

        // if Stacks isn't active, focus the new clip
        if !state.ui.is_visible {
            let focus = state.view.get_focus_for_id(&packet.id);
            state.ui.select(focus);
        }

        app.emit_all("refresh-items", true).unwrap();
    }
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
                state.with_lock(|state| {
                    handle_clipboard_update(state, &line, &app);
                });
            }
        }
    });
}
