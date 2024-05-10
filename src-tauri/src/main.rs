// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![recursion_limit = "512"]

use tracing::info;

mod cli;
mod clipboard;
mod commands;
mod content_bus;
mod content_type;
mod http;
mod publish;
mod serve;
mod spotlight;
mod state;
mod store;
mod ui;
mod util;
mod view;

#[cfg(test)]
mod store_tests;

#[cfg(test)]
mod ui_tests;

#[cfg(test)]
mod view_tests;

#[tokio::main]
async fn main() {
    let context = tauri::generate_context!();

    let system_app_data_dir = tauri::api::path::data_dir()
        .unwrap()
        .join(&context.config().tauri.bundle.identifier);

    let db_path = match std::env::var("STACK_DB_PATH") {
        Ok(path) => path,
        Err(_) => {
            let data_dir = system_app_data_dir;
            data_dir.join("store-v3.0").to_str().unwrap().to_string()
        }
    };
    info!(db_path, "let's go");

    if command_name() == "stacks" {
        cli::cli(&db_path).await;
    } else {
        serve::serve(context, db_path).await;
    }
}

fn command_name() -> String {
    std::env::args()
        .next()
        .map(|arg| {
            std::path::Path::new(&arg)
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or("")
                .to_string()
        })
        .unwrap()
}
