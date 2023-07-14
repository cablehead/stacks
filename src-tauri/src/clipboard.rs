use std::path::Path;

use tauri::api::process::{Command, CommandEvent};
use tauri::Manager;

use crate::store::SharedStore;
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
                loop {
                    let path = path.clone();
                    let result = (|| -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                        let env = xs_lib::store_open(&path).map_err(|e| format!("{}", e))?;
                        log::info!(
                            "{}",
                            xs_lib::store_put(&env, Some("clipboard".into()), None, line.clone())
                                .map_err(|e| format!("{}", e))?
                        );
                        Ok(())
                    })();

                    match result {
                        Ok(_) => break,
                        Err(e) => {
                            log::error!("Error: {}", e);
                            // Sleep for 1 second before retrying
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        }
                    }
                }
            }
        }
    });
}

// POLL_INTERVAL is the number of milliseconds to wait between polls when watching for
// additions to the stream
// todo: investigate switching to: https://docs.rs/notify/latest/notify/
const POLL_INTERVAL: u64 = 10;

pub fn start_child_process(app: tauri::AppHandle, path: &Path, store: SharedStore) {
    let path = path.to_path_buf();
    std::thread::spawn(move || {
        let mut last_id = None;
        let mut counter = 0;
        loop {
            let pump = (|| -> Result<(), Box<dyn std::error::Error>> {
                let env = xs_lib::store_open(&path)?;
                let frames = xs_lib::store_cat(&env, last_id)?;
                if !frames.is_empty() {
                    for frame in frames {
                        last_id = Some(frame.id);
                        let mut store = store.lock().unwrap();
                        store.add_frame(&frame);
                    }
                    app.emit_all("refresh-items", true)?;
                }
                Ok(())
            })();

            if let Err(e) = pump {
                log::error!("Error processing frames: {}", e);
            }

            if counter % 1000 == 0 {
                log::info!(
                    "start_child_process::last_id: {}",
                    last_id.map_or(String::from("None"), |id| id.to_string())
                );
            }
            counter += 1;
            std::thread::sleep(std::time::Duration::from_millis(POLL_INTERVAL));
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
                let clipped: Value = serde_json::from_str(&frame.data).unwrap();
                let types = clipped["types"].as_object().unwrap();

                if types.contains_key("public.utf8-plain-text") {
                    #[allow(deprecated)]
                    let content =
                        base64::decode(types["public.utf8-plain-text"].as_str().unwrap()).unwrap();

                    let content_type = if is_valid_https_url(&content) {
                        "Link"
                    } else {
                        "Text"
                    };

                    let _ = self.create_or_merge(
                        frame.id,
                        "text/plain",
                        content_type,
                        String::from_utf8(content.clone())
                            .unwrap()
                            .chars()
                            .take(100)
                            .collect(),
                        content,
                    );
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
