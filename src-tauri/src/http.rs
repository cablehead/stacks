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
use crate::ui::generate_preview;

type BoxError = Box<dyn std::error::Error + Send + Sync>;
type HTTPResult = Result<Response<BoxBody<Bytes, BoxError>>, BoxError>;

async fn handle(
    state: SharedState,
    app_handle: tauri::AppHandle,
    req: Request<hyper::body::Incoming>,
) -> HTTPResult {
    let path = req.uri().path();
    let id = path
        .strip_prefix("/")
        .and_then(|id| scru128::Scru128Id::from_str(id).ok());

    match (req.method(), id) {
        (&Method::GET, Some(id)) => get(id, state).await,
        (&Method::POST, None) if path == "/" => post(req, state, app_handle).await,
        _ => response_404(),
    }
}

async fn get(id: scru128::Scru128Id, state: SharedState) -> HTTPResult {
    let (item, meta) = state.with_lock(|state| {
        let item = state.view.items.get(&id).cloned();
        let meta = item
            .as_ref()
            .and_then(|i| state.store.get_content_meta(&i.hash));
        (item, meta)
    });

    match item {
        Some(item) => {
            let cache_path = state.with_lock(|state| state.store.cache_path.clone());
            let reader = cacache::Reader::open_hash(cache_path, item.hash)
                .await
                .unwrap();

            let stream = tokio_util::io::ReaderStream::new(reader);
            let stream = stream
                .map_ok(Frame::data)
                .map_err(|e| Box::new(e) as BoxError);
            let body = BodyExt::boxed(StreamBody::new(stream));

            let content_type = match meta {
                Some(meta) => match meta.mime_type {
                    MimeType::TextPlain => "text/plain",
                    MimeType::ImagePng => "image/png",
                },
                None => "application/octet-stream",
            };

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", content_type)
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
        let preview = generate_preview(
            "dark",
            &Some(streamer.content.clone()),
            &MimeType::TextPlain,
            &"Text".to_string(),
            true,
        );

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
                        println!("Error serving connection: {:?}", err);
                    }
                }
            });
        }
    });
}
