use crate::state::SharedState;
use crate::ui;
use crate::view::View;
use std::sync::mpsc::Receiver;
use tracing::{error, info};

#[tracing::instrument(skip_all)]
fn post(cross_stream_token: &str, previews: &str) -> Result<(), reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post("https://cross.stream")
        .header("Authorization", format!("Bearer {}", cross_stream_token))
        .body(previews.to_string())
        .send();

    response
        .map(|_| {
            info!("Successfully posted preview of {} bytes", previews.len());
        })
        .map_err(|e| {
            error!("Failed to POST preview: {}", e);
            e
        })
}

fn truncate_scru128(id: &scru128::Scru128Id, len: usize) -> String {
    let id_str = id.to_string();
    id_str.chars().rev().take(len).collect::<String>().chars().rev().collect()
}

#[tracing::instrument(skip(state, view), fields(%id = truncate_scru128(&id, 16)))]
fn generate_previews(state: &SharedState, view: &View, id: scru128::Scru128Id) -> String {
    state.with_lock(|state| {
        let children = view.children(view.items.get(&id).unwrap());

        let current_span = tracing::Span::current();
        current_span.record("n", children.len());

        let mut previews = Vec::new();
        for child_id in &children {
            let child = view.items.get(child_id).unwrap();
            let content = state.store.get_content(&child.hash);
            let mut ui_item = ui::with_meta(&state.store, child);

            if ui_item.content_type == "Text" {
                ui_item.content_type = "Markdown".into();
            }

            let preview = ui::generate_preview(&state.ui.theme_mode, &ui_item, &content);
            previews.push(preview);
        }
        previews
            .iter()
            .map(|preview| format!("<div>{}</div>", preview))
            .collect::<Vec<String>>()
            .join("")
    })
}

#[tracing::instrument(skip_all)]
fn process(state: &SharedState, view: &View, previous_preview: &mut String) {
    let (cross_stream_token, cross_stream_id) = state.with_lock(|state| {
        let settings = state.store.settings_get();
        let token = settings
            .and_then(|s| s.cross_stream_access_token)
            .filter(|t| t.len() == 64);
        let id = view
            .items
            .iter()
            .filter(|(_, item)| item.cross_stream)
            .map(|(id, _)| *id)
            .next();
        (token, id)
    });

    let cross_stream_token = match cross_stream_token {
        Some(token) => token,
        None => return,
    };

    let previews = if let Some(id) = cross_stream_id {
        generate_previews(&state, &view, id)
    } else {
        "".to_string()
    };

    if previews != *previous_preview {
        match post(&cross_stream_token, &previews) {
            Ok(_) => {
                *previous_preview = previews;
            }
            Err(_) => {}
        }
    }
}

pub fn publish_previews(state: SharedState, packet_receiver: Receiver<View>) {
    std::thread::spawn(move || {
        let mut previous_preview = String::new();
        for view in packet_receiver {
            process(&state, &view, &mut previous_preview);
        }
    });
}
