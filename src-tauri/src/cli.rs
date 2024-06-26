use std::io::Write;
use std::path::Path;

use http_body_util::BodyExt;
use hyper_util::rt::TokioIo;

use clap::{ArgGroup, Parser};

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
#[clap(group(ArgGroup::new("output").args(&["meta", "html"]).required(false)))]
struct Args {
    /// clip id to retrieve
    #[clap(value_parser)]
    id: Option<String>,

    /// output metadata, instead of content
    #[clap(long, action = clap::ArgAction::SetTrue, group = "output")]
    meta: bool,

    /// output in HTML format
    #[clap(long, action = clap::ArgAction::SetTrue, group = "output")]
    html: bool,
}

pub async fn cli(db_path: &str) {
    let args = Args::parse();

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

    // we should just do a HEAD request if --meta is set
    let request = Request::builder()
        .method("GET")
        .uri(&format!(
            "/{}{}",
            args.id.unwrap_or_default(),
            if args.html { "?as-html" } else { "" }
        ))
        .body(Empty::<Bytes>::new())
        .unwrap();

    let mut res = request_sender.send_request(request).await.unwrap();
    assert!(res.status() == StatusCode::OK);

    if args.meta {
        if let Some(metadata) = res.headers().get("X-Stacks-Clip-Metadata") {
            println!("{}", metadata.to_str().unwrap());
            return;
        }
    }

    while let Some(next) = res.frame().await {
        let frame = next.expect("Error reading frame");
        if let Some(chunk) = frame.data_ref() {
            // i was seeing some corruption using `tokio::io::stdout()`
            // https://discord.com/channels/500028886025895936/670880858630258689/1217899402325393500
            // switching to std's blocking io worked around the issue
            std::io::stdout()
                .write_all(chunk)
                .expect("Error writing to stdout");
        }
    }
}
