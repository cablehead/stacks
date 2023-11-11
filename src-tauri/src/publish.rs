use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use tracing::{error, info};

use crate::state::SharedState;
use crate::ui;
use crate::view::View;

// tracks previously published state
struct PreviousPublish {
    items: Vec<ui::Item>,
    cache: HashMap<String, String>,
}

impl PreviousPublish {
    fn new() -> Self {
        PreviousPublish {
            items: Vec::new(),
            cache: HashMap::new(),
        }
    }
}

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

/*
fn truncate_scru128(id: &scru128::Scru128Id, len: usize) -> String {
    let id_str = id.to_string();
    id_str
        .chars()
        .rev()
        .take(len)
        .collect::<String>()
        .chars()
        .rev()
        .collect()
}
*/

#[tracing::instrument(skip_all)]
fn generate(state: &SharedState, item: &ui::Item) -> String {
    let (content, theme_mode) = state.with_lock(|state| {
        (
            state.store.get_content(&item.hash),
            state.ui.theme_mode.clone(),
        )
    });
    ui::generate_preview(
        &theme_mode,
        &content,
        &item.mime_type,
        &item.content_type,
        item.ephemeral,
    )
}

#[tracing::instrument(skip_all)]
fn process(state: &SharedState, view: &View, previous: &mut PreviousPublish) {
    let (token, items) = state.with_lock(|state| {
        let settings = state.store.settings_get();
        let token = settings
            .and_then(|s| s.cross_stream_access_token)
            .filter(|t| t.len() == 64);

        if token.is_none() {
            return (token, Vec::new());
        }

        let id = view
            .items
            .iter()
            .filter(|(_, item)| item.cross_stream)
            .map(|(id, _)| *id)
            .next();

        let items = if let Some(id) = id {
            view.children(view.items.get(&id).unwrap())
                .iter()
                .map(|id| ui::with_meta(&state.store, view.items.get(id).unwrap()))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        (token, items)
    });

    let token = match token {
        Some(token) => token,
        None => return,
    };

    if items == previous.items {
        return;
    }

    let previews: Vec<String> = items
        .iter()
        .map(|item| {
            let cache_key = format!("{}{}", item.hash, item.content_type);
            previous
                .cache
                .entry(cache_key.clone())
                .or_insert_with(|| generate(&state, &item))
                .clone()
        })
        .collect();

    let previews = previews
        .iter()
        .map(|x| format!("<div>{}</div>", x))
        .collect::<Vec<String>>()
        .join("");

    match post(&token, &previews) {
        Ok(_) => previous.items = items,
        Err(_) => {}
    }
}

pub fn spawn(state: SharedState, packet_receiver: Receiver<View>) {
    std::thread::spawn(move || {
        let mut previous = PreviousPublish::new();
        for view in packet_receiver {
            process(&state, &view, &mut previous)
        }
    });
}
