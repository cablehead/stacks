use std::str::FromStr;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Request, Response, Server, StatusCode};

use crate::state::SharedState;

async fn handle(req: Request<Body>, state: SharedState) -> Result<Response<Body>, Error> {
    let path = req.uri().path();
    let item = path
        .strip_prefix("/")
        .and_then(|id| scru128::Scru128Id::from_str(id).ok())
        .and_then(|id| {
            let state = state.lock().unwrap();
            state.view.items.get(&id).map(|item| {
                println!("{:?}", item);
                item.clone()
            })
        });

    match item {
        Some(item) => {
            let cache_path = {
                let state = state.lock().unwrap();
                state.store.cache_path.clone()
            };
            let reader = cacache::Reader::open_hash(cache_path, item.hash)
                .await
                .unwrap();

            let stream = Body::wrap_stream(tokio_util::io::ReaderStream::new(reader));
            let response = Response::builder()
                .status(StatusCode::OK)
                .body(stream)
                .unwrap();

            Ok(response)
        }
        None => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap()),
    }
}

pub fn start(app_handle: tauri::AppHandle, state: SharedState) {
    tauri::async_runtime::spawn(async move {
        let addr = ([127, 0, 0, 1], 9146).into();

        let make_svc = make_service_fn(move |_conn| {
            let state = state.clone();
            async move {
                Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                    handle(req, state.clone())
                }))
            }
        });

        let server = Server::bind(&addr).serve(make_svc);

        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    });
}
