use std::error::Error;
use std::str::FromStr;

use futures_util::TryStreamExt;

use tokio::net::UnixListener;

use tauri::Manager;

use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full, StreamBody};
use hyper::body::Bytes;
use hyper::body::Frame;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, StatusCode};
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;

use crate::state::SharedState;
use crate::store::{infer_mime_type, InProgressStream, MimeType};

type BoxError = Box<dyn std::error::Error + Send + Sync>;
type HTTPResult = Result<Response<BoxBody<Bytes, BoxError>>, BoxError>;

#[tracing::instrument(skip(state, app_handle))]
async fn handle(
    state: SharedState,
    app_handle: tauri::AppHandle,
    req: Request<hyper::body::Incoming>,
) -> HTTPResult {
    let path = req.uri().path();

    use std::collections::HashMap;
    let params: HashMap<String, String> = req
        .uri()
        .query()
        .map(|v| {
            url::form_urlencoded::parse(v.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_default();

    let as_html = params.contains_key("as-html");

    // Handle CAS routes
    if path.starts_with("/cas") {
        return handle_cas(req.method(), path, state, app_handle).await;
    }

    // Handle stacks routes
    if path == "/stacks" && req.method() == Method::GET {
        return get_stacks_list(state).await;
    }

    // Handle stream routes
    if path == "/stream" && req.method() == Method::GET {
        return get_packet_stream(state).await;
    }

    // Handle search routes
    if path == "/search" && req.method() == Method::GET {
        return handle_search(req.uri().query(), state).await;
    }

    // Handle view routes
    if path == "/view" && req.method() == Method::GET {
        return get_view(state).await;
    }

    if path == "/view/nav" && req.method() == Method::GET {
        return get_view_nav(state).await;
    }

    // Handle delete routes
    if path.starts_with("/delete") && req.method() == Method::DELETE {
        return handle_delete(path, state, app_handle).await;
    }

    // Handle legacy routes
    let id_option = match path.strip_prefix('/') {
        Some("") | None => None, // Path is "/" or empty
        Some(id_str) => scru128::Scru128Id::from_str(id_str).ok(),
    };

    match (req.method(), id_option) {
        (&Method::GET, id) => get(id, state, as_html).await,
        (&Method::POST, None) if path == "/" => post(req, state, app_handle).await,
        _ => response_404(),
    }
}

async fn handle_cas(
    method: &Method,
    path: &str,
    state: SharedState,
    app_handle: tauri::AppHandle,
) -> HTTPResult {
    match (method, path) {
        (&Method::GET, "/cas") => get_cas_list(state).await,
        (&Method::GET, path) if path.starts_with("/cas/") => {
            let hash_str = &path[5..]; // Remove "/cas/" prefix
            match ssri::Integrity::from_str(hash_str) {
                Ok(hash) => get_cas_content(state, hash).await,
                Err(_) => response_404(),
            }
        }
        (&Method::DELETE, path) if path.starts_with("/cas/") => {
            let hash_str = &path[5..]; // Remove "/cas/" prefix
            match ssri::Integrity::from_str(hash_str) {
                Ok(hash) => delete_cas_content(state, hash, app_handle).await,
                Err(_) => response_404(),
            }
        }
        _ => response_404(),
    }
}

async fn get_cas_list(state: SharedState) -> HTTPResult {
    let hashes = state.with_lock(|state| state.store.enumerate_cas());

    // Convert Integrity objects to strings for JSON serialization
    let hash_strings: Vec<String> = hashes.iter().map(|h| h.to_string()).collect();
    let json_response = serde_json::to_string(&hash_strings).unwrap();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(full(json_response))?)
}

async fn get_cas_content(state: SharedState, hash: ssri::Integrity) -> HTTPResult {
    let (content, meta) = state.with_lock(|state| {
        let content = state.store.cas_read(&hash);
        let meta = state.store.get_content_meta(&hash);
        (content, meta)
    });

    match content {
        Some(_content_bytes) => {
            let cache_path = state.with_lock(|state| state.store.cache_path.clone());
            let reader = cacache::Reader::open_hash(cache_path, hash.clone())
                .await
                .unwrap();

            let stream = tokio_util::io::ReaderStream::new(reader);
            let stream = stream
                .map_ok(Frame::data)
                .map_err(|e| Box::new(e) as BoxError);
            let body = BodyExt::boxed(StreamBody::new(stream));

            let content_type = match meta {
                Some(ref meta) => match meta.mime_type {
                    MimeType::TextPlain => "text/plain",
                    MimeType::ImagePng => "image/png",
                },
                None => "application/octet-stream",
            };

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", content_type)
                .header("X-Stacks-CAS-Hash", hash.to_string())
                .body(body)?)
        }
        None => response_404(),
    }
}

async fn delete_cas_content(
    state: SharedState,
    hash: ssri::Integrity,
    app_handle: tauri::AppHandle,
) -> HTTPResult {
    let result = state.with_lock(|state| {
        let purge_result = state.store.purge(&hash);
        if purge_result.is_ok() {
            // Rescan to clean up any dangling references after purge
            state.rescan(None);
        }
        purge_result
    });

    // Notify UI to refresh after successful purge and rescan
    if result.is_ok() {
        app_handle.emit_all("refresh-items", true).unwrap();
    }

    match result {
        Ok(_) => {
            let response_body = format!("Purged content with hash: {hash}");
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain")
                .body(full(response_body))?)
        }
        Err(e) => {
            let error_body = format!("Error purging content: {e}");
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "text/plain")
                .body(full(error_body))?)
        }
    }
}

async fn get_stacks_list(state: SharedState) -> HTTPResult {
    let stacks = state.with_lock(|state| {
        // Find all items that are stacks (stack_id is None)
        let stack_items: Vec<_> = state
            .view
            .items
            .values()
            .filter(|item| item.stack_id.is_none())
            .cloned()
            .collect();

        // Convert to UI items with full metadata
        stack_items
            .into_iter()
            .map(|item| crate::ui::with_meta(&state.store, &item))
            .collect::<Vec<_>>()
    });

    let json_response = serde_json::to_string(&stacks).unwrap();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(full(json_response))?)
}

async fn get_packet_stream(state: SharedState) -> HTTPResult {
    let packets: Vec<_> = state.with_lock(|state| state.store.scan().collect());

    let json_response = serde_json::to_string(&packets).unwrap();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(full(json_response))?)
}

async fn handle_search(query_str: Option<&str>, state: SharedState) -> HTTPResult {
    let query_str = query_str.unwrap_or("");

    // Parse query parameters
    let params: std::collections::HashMap<String, String> =
        url::form_urlencoded::parse(query_str.as_bytes())
            .into_owned()
            .collect();

    let query = match params.get("q") {
        Some(q) => q,
        None => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "text/plain")
                .body(full("Missing 'q' parameter"))?);
        }
    };

    let limit = params.get("limit").and_then(|l| l.parse::<usize>().ok());

    let results =
        state.with_lock(|state| state.store.index.query(query, limit).unwrap_or_default());

    // Convert results to JSON format
    let json_results: Vec<serde_json::Value> = results
        .into_iter()
        .map(|(hash, score)| {
            serde_json::json!({
                "hash": hash.to_string(),
                "score": score
            })
        })
        .collect();

    let json_response = serde_json::to_string(&json_results).unwrap();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(full(json_response))?)
}

async fn get_view(state: SharedState) -> HTTPResult {
    let view = state.with_lock(|state| state.view.clone());

    let json_response = serde_json::to_string(&view).unwrap();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(full(json_response))?)
}

async fn get_view_nav(state: SharedState) -> HTTPResult {
    let nav = state.with_lock(|state| state.ui.render(&state.store));

    let json_response = serde_json::to_string(&nav).unwrap();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(full(json_response))?)
}

async fn handle_delete(path: &str, state: SharedState, app_handle: tauri::AppHandle) -> HTTPResult {
    // Parse the ID from the path: /delete/{id} or /delete/ (empty for default)
    let id_str = path.strip_prefix("/delete/").unwrap_or("");
    let id_option = if id_str.is_empty() {
        None
    } else {
        scru128::Scru128Id::from_str(id_str).ok()
    };

    let result = state.with_lock(|state| {
        // Find the item to delete (either specific ID or default to first item)
        let item = if let Some(id) = id_option {
            state.view.items.get(&id).cloned()
        } else {
            state.view.first().map(|focus| focus.item.clone())
        };

        let Some(item) = item else {
            return Err("Item not found".to_string());
        };

        // Get the item's hash for CAS cleanup
        let hash = item.hash.clone();

        // Delete the item (creates Delete packet)
        let _packet = state.store.delete(item.id);

        // Purge the CAS content
        if let Err(e) = state.store.purge(&hash) {
            tracing::warn!("Failed to purge CAS content for {}: {}", hash, e);
        }

        // Trigger rescan to clean up dangling references
        state.rescan(None);

        Ok(format!("Deleted item: {}", item.id))
    });

    // Notify UI to refresh after successful deletion
    if result.is_ok() {
        app_handle.emit_all("refresh-items", true).unwrap();
    }

    match result {
        Ok(message) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/plain")
            .body(full(message))?),
        Err(error) => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "text/plain")
            .body(full(error))?),
    }
}

fn get_as_html(state: SharedState, hash: ssri::Integrity) -> HTTPResult {
    let preview = state.with_lock(|state| {
        let content = state.store.get_content(&hash);
        let meta = state.store.get_content_meta(&hash).unwrap();
        state
            .ui
            .generate_preview(&content, &meta.mime_type, &meta.content_type, false)
    });

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .body(full(preview))?)
}

async fn get(id: Option<scru128::Scru128Id>, state: SharedState, as_html: bool) -> HTTPResult {
    let (item, meta) = state.with_lock(|state| {
        let item = if let Some(id) = id {
            state.view.items.get(&id).cloned()
        } else {
            state.view.first().map(|focus| focus.item.clone())
        };

        let meta = item
            .as_ref()
            .and_then(|i| state.store.get_content_meta(&i.hash));
        (item, meta)
    });

    match item {
        Some(item) => {
            if as_html {
                return get_as_html(state, item.hash);
            }

            let cache_path = state.with_lock(|state| state.store.cache_path.clone());
            let reader = cacache::Reader::open_hash(cache_path, item.hash.clone())
                .await
                .unwrap();

            let stream = tokio_util::io::ReaderStream::new(reader);
            let stream = stream
                .map_ok(Frame::data)
                .map_err(|e| Box::new(e) as BoxError);
            let body = BodyExt::boxed(StreamBody::new(stream));

            let content_type = match meta {
                Some(ref meta) => match meta.mime_type {
                    MimeType::TextPlain => "text/plain",
                    MimeType::ImagePng => "image/png",
                },
                None => "application/octet-stream",
            };

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", content_type)
                .header(
                    "X-Stacks-Clip-Metadata",
                    serde_json::json!({"clip": &item, "content":&meta}).to_string(),
                )
                .body(body)?)
        }
        None => response_404(),
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct Content {
    pub mime_type: MimeType,
    pub content_type: String,
    pub terse: String,
    pub tiktokens: usize,
    pub words: usize,
    pub chars: usize,
    pub preview: String,
}

async fn post(
    req: Request<hyper::body::Incoming>,
    state: SharedState,
    app_handle: tauri::AppHandle,
) -> HTTPResult {
    let mut streamer = state.with_lock(|state| {
        let stack = state.get_curr_stack();
        state.ui.select(None); // focus first
        let (mime_type, content_type) = infer_mime_type("".as_bytes(), MimeType::TextPlain);
        let streamer = InProgressStream::new(stack, mime_type, content_type);
        state.merge(&streamer.packet);
        app_handle.emit_all("refresh-items", true).unwrap();
        streamer
    });

    let mut body = req.into_body();

    while let Some(frame) = body.frame().await {
        let data = frame?.into_data().unwrap();
        streamer.append(&data);
        let preview = state.with_lock(|state| {
            state.ui.generate_preview(
                &Some(streamer.content.clone()),
                &MimeType::TextPlain,
                &"Text".to_string(),
                true,
            )
        });

        let content = String::from_utf8_lossy(&streamer.content);
        let content = Content {
            mime_type: MimeType::TextPlain,
            content_type: "Text".to_string(),
            terse: content.chars().take(100).collect(),
            tiktokens: 0,
            words: content.split_whitespace().count(),
            chars: content.chars().count(),
            preview,
        };

        app_handle
            .emit_all("streaming", (streamer.packet.id, content))
            .unwrap();
    }

    state.with_lock(|state| {
        let packet = streamer.end_stream(&mut state.store);
        state.merge(&packet);
        state.store.insert_packet(&packet);
    });
    app_handle.emit_all("refresh-items", true).unwrap();

    let response_body = streamer.packet.id.to_string();
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(full(response_body))?)
}

fn response_404() -> HTTPResult {
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(empty())?)
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, BoxError> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

fn empty() -> BoxBody<Bytes, BoxError> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

pub fn start(app_handle: tauri::AppHandle, state: SharedState, db_path: &str) {
    let socket_path = std::path::Path::new(db_path).join("sock");
    let _ = std::fs::remove_file(&socket_path);
    let listener = UnixListener::bind(socket_path).unwrap();

    tauri::async_runtime::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let io = TokioIo::new(stream);

            let state_cloned = state.clone();
            let app_handle_clonded = app_handle.clone();

            tauri::async_runtime::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(
                        io,
                        service_fn(move |req| {
                            handle(state_cloned.clone(), app_handle_clonded.clone(), req)
                        }),
                    )
                    .await
                {
                    // Match against the error kind to selectively ignore `NotConnected` errors
                    if let Some(std::io::ErrorKind::NotConnected) =
                        err.source().and_then(|source| {
                            source
                                .downcast_ref::<std::io::Error>()
                                .map(|io_err| io_err.kind())
                        })
                    {
                        // Silently ignore the NotConnected error
                    } else {
                        // Handle or log other errors
                        println!("Error serving connection: {err:?}");
                    }
                }
            });
        }
    });
}
