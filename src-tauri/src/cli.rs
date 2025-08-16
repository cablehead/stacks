use std::io::Write;
use std::path::Path;

use http_body_util::BodyExt;
use hyper_util::rt::TokioIo;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Option<Commands>,

    /// clip id to retrieve (when no subcommand is used)
    #[clap(value_parser)]
    id: Option<String>,

    /// output metadata, instead of content
    #[clap(long, action = clap::ArgAction::SetTrue)]
    meta: bool,

    /// output in HTML format
    #[clap(long, action = clap::ArgAction::SetTrue)]
    html: bool,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// List all stacks with full metadata (JSONL format)
    List,
    /// Content-Addressable Storage operations
    Cas {
        #[clap(subcommand)]
        command: CasCommand,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum CasCommand {
    /// List all CAS hashes
    List,
    /// Get content by hash
    Get { hash: String },
    /// Purge content by hash
    Purge { hash: String },
}

pub async fn cli(db_path: &str) {
    let args = Args::parse();

    let socket_path = Path::new(db_path).join("sock");
    let stream = tokio::net::UnixStream::connect(socket_path)
        .await
        .expect("Failed to connect to server");
    let io = TokioIo::new(stream);

    use hyper::client::conn;

    let (mut request_sender, connection) = conn::http1::handshake(io).await.unwrap();

    // spawn a task to poll the connection and drive the HTTP state
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Error in connection: {e}");
        }
    });

    match args.command {
        Some(Commands::List) => {
            handle_list_command(&mut request_sender).await;
        }
        Some(Commands::Cas { command }) => {
            handle_cas_command(command, &mut request_sender).await;
        }
        None => {
            // Legacy behavior for backward compatibility
            handle_legacy_request(args, &mut request_sender).await;
        }
    }
}

async fn handle_list_command(
    request_sender: &mut hyper::client::conn::http1::SendRequest<
        http_body_util::Empty<bytes::Bytes>,
    >,
) {
    use bytes::Bytes;
    use http_body_util::Empty;
    use hyper::{Method, Request, StatusCode};

    let request = Request::builder()
        .method(Method::GET)
        .uri("/stacks")
        .body(Empty::<Bytes>::new())
        .unwrap();

    let mut res = request_sender.send_request(request).await.unwrap();

    if res.status() != StatusCode::OK {
        eprintln!("Request failed with status: {}", res.status());
        return;
    }

    // Parse JSON response and output each stack as a line (JSONL format)
    let mut body_bytes = Vec::new();
    while let Some(next) = res.frame().await {
        let frame = next.expect("Error reading frame");
        if let Some(chunk) = frame.data_ref() {
            body_bytes.extend_from_slice(chunk);
        }
    }

    let body_str = String::from_utf8(body_bytes.clone()).unwrap_or_else(|_| {
        eprintln!("Server returned invalid UTF-8");
        String::from_utf8_lossy(&body_bytes).to_string()
    });

    match serde_json::from_str::<Vec<serde_json::Value>>(&body_str) {
        Ok(stacks) => {
            for stack in stacks {
                println!("{}", serde_json::to_string(&stack).unwrap());
            }
        }
        Err(e) => {
            eprintln!("Failed to parse JSON response: {e}");
            eprintln!("Raw response: {body_str}");
        }
    }
}

async fn handle_cas_command(
    command: CasCommand,
    request_sender: &mut hyper::client::conn::http1::SendRequest<
        http_body_util::Empty<bytes::Bytes>,
    >,
) {
    use bytes::Bytes;
    use http_body_util::Empty;
    use hyper::{Method, Request, StatusCode};

    let (method, uri) = match &command {
        CasCommand::List => (Method::GET, "/cas".to_string()),
        CasCommand::Get { hash } => (Method::GET, format!("/cas/{hash}")),
        CasCommand::Purge { hash } => (Method::DELETE, format!("/cas/{hash}")),
    };

    let request = Request::builder()
        .method(method)
        .uri(uri)
        .body(Empty::<Bytes>::new())
        .unwrap();

    let mut res = request_sender.send_request(request).await.unwrap();

    if res.status() != StatusCode::OK {
        eprintln!("Request failed with status: {}", res.status());
        return;
    }

    match command {
        CasCommand::List => {
            // Parse JSON response and output one hash per line
            let mut body_bytes = Vec::new();
            while let Some(next) = res.frame().await {
                let frame = next.expect("Error reading frame");
                if let Some(chunk) = frame.data_ref() {
                    body_bytes.extend_from_slice(chunk);
                }
            }

            let body_str = String::from_utf8(body_bytes.clone()).unwrap_or_else(|_| {
                eprintln!("Server returned invalid UTF-8");
                String::from_utf8_lossy(&body_bytes).to_string()
            });
            match serde_json::from_str::<Vec<String>>(&body_str) {
                Ok(hashes) => {
                    for hash in hashes {
                        println!("{hash}");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse JSON response: {e}");
                    eprintln!("Raw response: {body_str}");
                }
            }
        }
        CasCommand::Get { .. } => {
            // Stream content to stdout
            while let Some(next) = res.frame().await {
                let frame = next.expect("Error reading frame");
                if let Some(chunk) = frame.data_ref() {
                    std::io::stdout()
                        .write_all(chunk)
                        .expect("Error writing to stdout");
                }
            }
        }
        CasCommand::Purge { .. } => {
            // Output success/error message
            while let Some(next) = res.frame().await {
                let frame = next.expect("Error reading frame");
                if let Some(chunk) = frame.data_ref() {
                    std::io::stdout()
                        .write_all(chunk)
                        .expect("Error writing to stdout");
                }
            }
        }
    }
}

async fn handle_legacy_request(
    args: Args,
    request_sender: &mut hyper::client::conn::http1::SendRequest<
        http_body_util::Empty<bytes::Bytes>,
    >,
) {
    use bytes::Bytes;
    use http_body_util::Empty;
    use hyper::{Request, StatusCode};

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
