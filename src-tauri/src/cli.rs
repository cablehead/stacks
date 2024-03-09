use std::path::Path;

use http_body_util::BodyExt;
use hyper_util::rt::TokioIo;
use tokio::io::AsyncWriteExt as _;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the store
    #[clap(value_parser)]
    id: Option<String>,
}

pub async fn cli(db_path: &str) {
    let _args = Args::parse();

    let socket_path = Path::new(db_path).join("sock");
    let stream = tokio::net::UnixStream::connect(socket_path)
        .await
        .expect("Failed to connect to server");
    let io = TokioIo::new(stream);

    use bytes::Bytes;
    use http_body_util::Empty;
    use hyper::client::conn;
    use hyper::{Request, StatusCode};

    let (mut request_sender, connection) = conn::http1::handshake(io).await.unwrap();

    // spawn a task to poll the connection and drive the HTTP state
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Error in connection: {}", e);
        }
    });

    let request = Request::builder()
        .method("GET")
        .uri("/03BBMWT9FVUC6JUOOOLLTFEEZ")
        .body(Empty::<Bytes>::new())
        .unwrap();

    let mut res = request_sender.send_request(request).await.unwrap();
    assert!(res.status() == StatusCode::OK);

    // Stream the body, writing each chunk to stdout as we get it
    while let Some(next) = res.frame().await {
        let frame = next.expect("Error reading frame");
        if let Some(chunk) = frame.data_ref() {
            tokio::io::stdout()
                .write_all(&chunk)
                .await
                .expect("Error writing to stdout");
        }
    }
    // eprintln!("{:?} {:?} {:?}", db_path, args, res);
}
