// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use tauri::GlobalShortcutManager;
use tauri::Manager;

use clap::Parser;

use lazy_static::lazy_static;

lazy_static! {
    static ref ARGS: Args = Args::parse();
}

#[derive(Clone, serde::Serialize)]
struct Payload {
    message: String,
}

fn validate_and_create_path(s: &str) -> Result<PathBuf, String> {
    let path_string = shellexpand::tilde(s).into_owned();
    let path = PathBuf::from(&path_string);

    if !path.exists() {
        std::fs::create_dir_all(&path)
            .map_err(|_| format!("Failed to create directory at `{}`", s))?;
    }

    Ok(path)
}

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to stream
    #[clap(default_value = "~/.config/stacks/stream", value_parser = validate_and_create_path)]
    path: PathBuf,
}

#[derive(Clone, serde::Serialize)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

#[tauri::command]
fn run_command(command: &str) -> Result<CommandOutput, String> {
    let parts = shlex::split(command).ok_or("Failed to parse command")?;
    let program = parts.get(0).ok_or("No program specified")?;
    let args = &parts[1..];

    let output = std::process::Command::new(program)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    let stdout = String::from_utf8(output.stdout).unwrap_or_else(|_| String::new());
    let stderr = String::from_utf8(output.stderr).unwrap_or_else(|_| String::new());
    let exit_code = output.status.code().unwrap_or(-1);

    Ok(CommandOutput {
        stdout,
        stderr,
        exit_code,
    })
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![run_command])
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::Focused(focused) => {
                if !focused {
                    event.window().hide().unwrap();
                }
            }
            _ => {}
        })
        .setup(move |app| {
            let win = app.get_window("main").unwrap();
            let mut shortcut = app.global_shortcut_manager();
            shortcut
                .register("Cmd+Shift+G", move || {
                    if win.is_visible().unwrap() {
                        win.hide().unwrap();
                    } else {
                        win.show().unwrap();
                        win.set_focus().unwrap();
                    }
                })
                .unwrap_or_else(|err| println!("{:?}", err));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
