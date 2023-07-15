use tauri::api::process::{Command, CommandEvent};
use tauri::Manager;

use serde_json::Value;

use crate::state::SharedState;
use crate::store::MimeType;

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
                    #[allow(deprecated)]
                    let content =
                        base64::decode(types["public.utf8-plain-text"].as_str().unwrap()).unwrap();

                    let frame = state.store.put(source, MimeType::TextPlain, &content);
                    state.stack.create_or_merge(&frame, &content);

                    app.emit_all("refresh-items", true).unwrap();
                }
            }
        }
    });
}

/*
 * TO PORT:
 *

    fn find_item_by_id(&mut self, id: &str) -> Option<Item> {
        let id = id.parse::<scru128::Scru128Id>().ok()?;
        for item in self.items.values() {
            if item.ids.contains(&id) {
                return Some(item.clone());
            }
        }
        None
    }

    pub fn add_frame(&mut self, frame: &xs_lib::Frame) {
        match &frame.topic {
            Some(topic) if topic == "clipboard" => {
                } else if types.contains_key("public.png") {
                    let content = types["public.png"].as_str().unwrap().as_bytes();
                    self.create_or_merge(
                        frame.id,
                        "image/png",
                        "Image",
                        clipped["source"].to_string(),
                        content.to_vec(),
                    );
                } else {
                    log::info!(
                        "add_frame TODO: topic: clipboard id: {}, types: {:?}, frame.data size: {}",
                        frame.id,
                        types.keys().collect::<Vec<_>>(),
                        frame.data.len()
                    );
                }
            }

            Some(topic) if topic == "stack" => {
                let data: Value = serde_json::from_str(&frame.data).unwrap();

                let id = data["id"].as_str();
                if let None = id {
                    return;
                }
                let id = id.unwrap();

                let target = self.find_item_by_id(id);
                if let None = target {
                    return;
                }
                let mut target = target.unwrap();

                let content = data["name"].as_str().unwrap();

                if let Some(attr) = &frame.attribute {
                    if attr == "delete" {
                        let hash = format!("{:x}", Sha256::digest(&content));
                        if let Some(stack) = self.items.get_mut(&hash) {
                            let id = id.parse::<scru128::Scru128Id>().ok().unwrap();
                            stack.stack.retain(|_, item| !item.ids.contains(&id));
                        }
                        return;
                    }
                }

                let hash = self.create_or_merge(
                    frame.id,
                    "text/plain",
                    "Stack",
                    content.chars().take(100).collect(),
                    content.as_bytes().to_vec(),
                );

                let item = self.items.get_mut(&hash).unwrap();
                item.content_type = "Stack".to_string();
                target.ids.push(frame.id);
                item.stack.insert(target.hash.to_string(), target);
            }

            Some(topic) => {
                log::info!("add_frame TODO: topic: {} id: {}", topic, frame.id,);
                if topic != "link" {
                    let _ = self.create_or_merge(
                        frame.id,
                        "text/plain",
                        "Text",
                        frame.data.chars().take(100).collect(),
                        frame.data.as_bytes().to_vec(),
                    );
                }
            }
            None => (),
        };
    }
*/
