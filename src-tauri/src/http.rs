use futures::StreamExt;
use std::str::FromStr;
use tauri::Manager;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Method, Request, Response, Server, StatusCode};

use tracing::error;

use crate::state::SharedState;

async fn handle(
    req: Request<Body>,
    state: SharedState,
    app_handle: tauri::AppHandle,
) -> Result<Response<Body>, Error> {
    let path = req.uri().path();
    let id = path
        .strip_prefix("/")
        .and_then(|id| scru128::Scru128Id::from_str(id).ok());

    match (req.method(), id) {
        (&Method::GET, Some(id)) => get(id, state).await,
        (&Method::POST, None) if path == "/" => post(req, state.clone(), app_handle.clone()).await,
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap()),
    }
}

async fn get(id: scru128::Scru128Id, state: SharedState) -> Result<Response<Body>, Error> {
    let item = state.with_lock(|state| {
        state.view.items.get(&id).cloned()
    });

    match item {
        Some(item) => {
            let cache_path = state.with_lock(|state| {
                state.store.cache_path.clone()
            });
            let reader = cacache::Reader::open_hash(cache_path, item.hash)
                .await
                .unwrap();
            let stream = Body::wrap_stream(tokio_util::io::ReaderStream::new(reader));
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(stream)
                .unwrap())
        }
        None => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap()),
    }
}


async fn post(
    req: Request<Body>,
    state: SharedState,
    app_handle: tauri::AppHandle,
) -> Result<Response<Body>, Error> {
    let id = state.with_lock(|state| {
        let stack = state.get_curr_stack();
        state.ui.select(None); // focus first
        let packet = state.store.start_stream(Some(stack), "".as_bytes());
        packet.id.clone()
    });

    let mut bytes_stream = req.into_body();

    while let Some(chunk_result) = bytes_stream.next().await {
        if let Ok(chunk) = chunk_result {
            let data = chunk.to_vec();
            state.with_lock(|state| {
                let packet = state.store.update_stream(id, &data);
                state.merge(&packet);
            });
            app_handle.emit_all("refresh-items", true).unwrap();
        } else {
            // TODO: handle error
        }
    }

    state.with_lock(|state| {
        let packet = state.store.end_stream(id);
        state.merge(&packet);
    });
    app_handle.emit_all("refresh-items", true).unwrap();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(id.to_string()))
        .unwrap())
}


pub fn start(app_handle: tauri::AppHandle, state: SharedState) {
    tauri::async_runtime::spawn(async move {
        let addr = ([127, 0, 0, 1], 9146).into();

        let make_svc = make_service_fn(move |_conn| {
            let state = state.clone();
            let app_handle = app_handle.clone();
            async move {
                Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                    handle(req, state.clone(), app_handle.clone())
                }))
            }
        });

        let server = Server::bind(&addr).serve(make_svc);

        if let Err(e) = server.await {
            error!("server error: {}", e);
        }
    });
}
