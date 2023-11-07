use crate::state::SharedState;
use crate::ui;
use crate::view::View;
use std::sync::mpsc::Receiver;
use tracing::{error, info};

pub fn publish_previews(state: SharedState, packet_receiver: Receiver<View>) {
    std::thread::spawn(move || {
        let mut previous_preview = String::new();
        for view in packet_receiver {
            let state = state.lock().unwrap();
            let settings = state.store.settings_get();
            let cross_stream_token = match settings.and_then(|s| s.cross_stream_access_token) {
                Some(token) => token,
                None => continue,
            };

            if cross_stream_token.len() != 64 {
                continue;
            }

            let cross_stream_id = view
                .items
                .iter()
                .filter(|(_, item)| item.cross_stream)
                .map(|(id, _)| *id)
                .next();
            let previews = if let Some(id) = cross_stream_id {
                let children = view.children(view.items.get(&id).unwrap());
                let mut previews = Vec::new();
                for child_id in &children {
                    let child = view.items.get(child_id).unwrap();
                    let content = state.store.get_content(&child.hash);
                    let mut ui_item = ui::with_meta(&state.store, child);

                    if ui_item.content_type == "Text" {
                        ui_item.content_type = "Markdown".into();
                    }
                    info!("{:?}", &ui_item.content_type);

                    let preview = ui::generate_preview(&state.ui.theme_mode, &ui_item, &content);
                    previews.push(preview);
                }
                previews
                    .iter()
                    .map(|preview| format!("<div>{}</div>", preview))
                    .collect::<Vec<String>>()
                    .join("")
            } else {
                "".to_string()
            };

            if previews != previous_preview {
                let client = reqwest::blocking::Client::new();
                let res = client
                    .post("https://cross.stream")
                    .header("Authorization", format!("Bearer {}", cross_stream_token))
                    .body(previews.clone())
                    .send();
                match res {
                    Ok(_) => {
                        info!("Successfully posted preview of {} bytes", previews.len());
                        previous_preview = previews;
                    }
                    Err(e) => error!("Failed to POST preview: {}", e),
                }
            }
        }
    });
}
