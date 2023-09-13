use crate::state::SharedState;

#[cfg(debug_assertions)]
pub fn start(app_handle: tauri::AppHandle, state: SharedState) {
    let state = state.clone();
    tauri::async_runtime::spawn(async move {
        use hyper::service::{make_service_fn, service_fn};
        use hyper::{Body, Error, Request, Response, Server, StatusCode};

        async fn handle(req: Request<Body>, state: SharedState) -> Result<Response<Body>, Error> {
            let path = req.uri().path();
            if let Some(id) = path.strip_prefix("/") {
                println!("Received id: {}", id);
                let mut state = state.lock().unwrap();
                Ok(Response::new(Body::from(id.to_string())))
            } else {
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("Not Found"))
                    .unwrap())
            }
        }

        let addr = ([127, 0, 0, 1], 9146).into();
        let make_svc = make_service_fn(|_conn| {
            let state = state.clone();
            async {
                Ok::<_, Error>(service_fn(move |req: Request<Body>| {
                    let state = state.clone();
                    async move { handle(req, state) }
                }))
            }
        });

        let server = Server::bind(&addr).serve(make_svc);

        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    });
}
