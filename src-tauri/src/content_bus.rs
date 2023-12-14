use tokio::sync::broadcast;

use tauri::Manager;

use crate::state::SharedState;
use crate::store::{count_tiktokens, MimeType};

pub fn spawn_tiktokens(app: tauri::AppHandle, state: SharedState) {
    let (cache_path, mut rx) = state.with_lock(|state| {
        (
            state.store.cache_path.clone(),
            state.store.content_bus_tx.subscribe(),
        )
    });

    tokio::spawn(async move {
        tracing::info!(name = "content_bus::tiktokens", "booting");
        loop {
            let cache_path = cache_path.clone();
            match rx.recv().await {
                Ok(content_meta) => {
                    if content_meta.mime_type == MimeType::TextPlain {
                        let hash = content_meta.hash.clone();
                        let tiktokens = tokio::task::spawn_blocking(move || {
                            let content =
                                cacache::read_hash_sync(&cache_path, &content_meta.hash).unwrap();
                            let content = String::from_utf8_lossy(&content);
                            let tiktokens = count_tiktokens(&content);
                        tracing::info!(name = "content_bus::tiktokens", hash = %content_meta.hash, tiktokens = tiktokens);
                        tiktokens
                        })
                        .await
                        .unwrap();

                        state.with_lock(|state| {
                            state.store.update_tiktokens(hash.clone(), tiktokens);
                        });
                        app.emit_all("content", hash).unwrap();
                    }
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    tracing::warn!(
                        name = "content_bus::tiktokens",
                        skipped = skipped,
                        "channel lagged"
                    );
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    });
}
