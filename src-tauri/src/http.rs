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
        return handle_cas(req.method(), path, state).await;
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

async fn handle_cas(method: &Method, path: &str, state: SharedState) -> HTTPResult {
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
                Ok(hash) => delete_cas_content(state, hash).await,
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

async fn delete_cas_content(state: SharedState, hash: ssri::Integrity) -> HTTPResult {
    let result = state.with_lock(|state| state.store.purge(&hash));

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
