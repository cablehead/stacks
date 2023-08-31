use tauri::Manager;

use base64::{engine::general_purpose, Engine as _};

use scru128::Scru128Id;

use crate::state::SharedState;
use crate::store::MimeType;
use crate::ui::Nav;

#[derive(Debug, serde::Serialize)]
pub struct CommandOutput {
    pub out: String,
    pub err: String,
    pub code: i32,
}

#[tauri::command]
pub async fn store_pipe_to_command(
    state: tauri::State<'_, SharedState>,
    source_id: scru128::Scru128Id,
    command: String,
) -> Result<CommandOutput, ()> {
    println!("PIPE: {} {}", &source_id, &command);
    let (cache_path, hash) = {
        let state = state.lock().unwrap();
        let cache_path = state.store.cache_path.clone();
        let item = state.view.items.get(&source_id).unwrap();
        (cache_path, item.hash.clone())
    };

    let home_dir = dirs::home_dir().expect("Could not fetch home directory");
    let shell = match std::env::var("SHELL") {
        Ok(val) => val,
        Err(_) => String::from("/bin/sh"), // default to sh if no SHELL variable is set
    };

    let rc_file = match shell.as_str() {
        "/bin/bash" => ".bashrc",
        "/bin/zsh" => ".zshrc",
        _ => "", // if the shell is neither bash nor zsh, don't source an rc file
    };

    let rc_path = home_dir.join(rc_file);
    let rc_command = format!("source {}\n{}", rc_path.to_str().unwrap_or(""), command);

    let mut cmd = tokio::process::Command::new(shell)
        .arg("-c")
        .arg(rc_command)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = cmd.stdin.take().ok_or("Failed to open stdin").unwrap();
    let mut reader = cacache::Reader::open_hash(cache_path, hash).await.unwrap();
    tokio::io::copy(&mut reader, &mut stdin).await.unwrap();
    drop(stdin);

    let output = cmd.wait_with_output().await.unwrap();
    let output = CommandOutput {
        out: String::from_utf8_lossy(&output.stdout).into_owned(),
        err: String::from_utf8_lossy(&output.stderr).into_owned(),
        code: output.status.code().unwrap_or(-1),
    };
    println!("PIPE, RES: {:?}", &output);
    Ok(output)
}

#[tauri::command]
pub fn store_get_content(
    state: tauri::State<SharedState>,
    hash: ssri::Integrity,
) -> Option<String> {
    println!("CACHE MISS: {}", &hash);
    let state = state.lock().unwrap();
    state
        .store
        .cas_read(&hash)
        .map(|vec| general_purpose::STANDARD.encode(vec))
}

#[tauri::command]
pub fn store_nav_refresh(state: tauri::State<SharedState>) -> Nav {
    let state = state.lock().unwrap();
    state.ui.render(&state.store)
}

#[tauri::command]
pub fn store_nav_reset(state: tauri::State<SharedState>) -> Nav {
    let mut state = state.lock().unwrap();
    let view = state.view.clone();
    state.ui.reset(view);
    state.ui.render(&state.store)
}

#[tauri::command]
pub fn store_nav_set_filter(
    state: tauri::State<SharedState>,
    filter: String,
    content_type: String,
) -> Nav {
    let mut state = state.lock().unwrap();
    // XXX: content_type should be an enum
    let content_type = match content_type.as_str() {
        "Links" => "Link",
        "Images" => "Image",
        _ => "All",
    };
    state.nav_set_filter(&filter, content_type);
    state.ui.render(&state.store)
}

#[tauri::command]
pub fn store_nav_select(state: tauri::State<SharedState>, focused_id: Scru128Id) -> Nav {
    let mut state = state.lock().unwrap();
    state.nav_select(&focused_id);
    state.ui.render(&state.store)
}

#[tauri::command]
pub fn store_nav_select_up(state: tauri::State<SharedState>) -> Nav {
    let mut state = state.lock().unwrap();
    state.ui.select_up();
    state.ui.render(&state.store)
}

#[tauri::command]
pub fn store_nav_select_down(state: tauri::State<SharedState>) -> Nav {
    let mut state = state.lock().unwrap();
    state.ui.select_down();
    state.ui.render(&state.store)
}

#[tauri::command]
pub fn store_nav_select_left(state: tauri::State<SharedState>) -> Nav {
    let mut state = state.lock().unwrap();
    state.ui.select_left();
    state.ui.render(&state.store)
}

#[tauri::command]
pub fn store_nav_select_right(state: tauri::State<SharedState>) -> Nav {
    let mut state = state.lock().unwrap();
    state.ui.select_right();
    state.ui.render(&state.store)
}

use cocoa::base::nil;
use cocoa::foundation::NSString;
use objc::{msg_send, sel, sel_impl};

pub fn write_to_clipboard(mime_type: &str, data: &[u8]) -> Option<i64> {
    unsafe {
        let nsdata: *mut objc::runtime::Object = msg_send![objc::class!(NSData), alloc];
        let nsdata: *mut objc::runtime::Object =
            msg_send![nsdata, initWithBytes:data.as_ptr() length:data.len()];

        let pasteboard: *mut objc::runtime::Object =
            msg_send![objc::class!(NSPasteboard), generalPasteboard];

        let png_type = NSString::alloc(nil).init_str(mime_type);

        let i: i64 = msg_send![pasteboard, clearContents];
        let success: bool = msg_send![pasteboard, setData: nsdata forType: png_type];

        // After the data is set, release the nsdata object to prevent a memory leak.
        let () = msg_send![nsdata, release];
        let () = msg_send![png_type, release];

        if !success {
            return None;
        }
        Some(i)
    }
}

#[tauri::command]
pub fn store_copy_to_clipboard(
    state: tauri::State<SharedState>,
    source_id: scru128::Scru128Id,
) -> Option<()> {
    let state = state.lock().unwrap();

    if let Some(item) = state.view.items.get(&source_id) {
        let meta = state.store.get_content_meta(&item.hash).unwrap();

        let mime_type = match &meta.mime_type {
            MimeType::TextPlain => "public.utf8-plain-text",
            MimeType::ImagePng => "public.png",
        };
        let content = state.store.cas_read(&item.hash).unwrap();

        let _change_num = write_to_clipboard(mime_type, &content);
        Some(())
    } else {
        None
    }
}

#[tauri::command]
pub fn store_new_note(
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    content: String,
    stack_id: Option<scru128::Scru128Id>,
) {
    let mut state = state.lock().unwrap();

    let stack_id = stack_id.unwrap_or_else(|| state.get_curr_stack());

    let packet = state.store.add(
        content.as_bytes(),
        MimeType::TextPlain,
        Some(stack_id),
        None,
    );
    state.merge(packet);

    state.skip_change_num = write_to_clipboard("public.utf8-plain-text", content.as_bytes());
    app.emit_all("refresh-items", true).unwrap();
}

#[tauri::command]
pub fn store_edit_note(
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    source_id: scru128::Scru128Id,
    content: String,
) {
    let mut state = state.lock().unwrap();
    let packet = state.store.update(
        source_id,
        Some(content.as_bytes()),
        MimeType::TextPlain,
        None,
        None,
    );
    state.merge(packet);

    state.skip_change_num = write_to_clipboard("public.utf8-plain-text", content.as_bytes());
    app.emit_all("refresh-items", true).unwrap();
}

#[tauri::command]
pub fn store_delete(
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    id: scru128::Scru128Id,
) {
    let mut state = state.lock().unwrap();
    let packet = state.store.delete(id);
    state.merge(packet);
    app.emit_all("refresh-items", true).unwrap();
}

//
// Settings related commands

#[tauri::command]
pub fn store_settings_save(state: tauri::State<SharedState>, settings: serde_json::Value) {
    let state = state.lock().unwrap();
    state
        .store
        .meta
        .insert("settings", settings.to_string().as_bytes())
        .unwrap();
}

#[tauri::command]
pub fn store_settings_get(state: tauri::State<SharedState>) -> serde_json::Value {
    let state = state.lock().unwrap();
    let res = state.store.meta.get("settings").unwrap();
    match res {
        Some(bytes) => {
            let str = std::str::from_utf8(bytes.as_ref()).unwrap();
            serde_json::from_str(str).unwrap()
        }
        None => serde_json::Value::Object(Default::default()),
    }
}

//
// Stack related commands

#[tauri::command]
pub fn store_add_to_stack(
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    stack_id: scru128::Scru128Id,
    source_id: scru128::Scru128Id,
) {
    let mut state = state.lock().unwrap();

    let packet = state
        .store
        .fork(source_id, None, MimeType::TextPlain, Some(stack_id), None);
    state.merge(packet);

    app.emit_all("refresh-items", true).unwrap();
}

#[tauri::command]
pub fn store_add_to_new_stack(
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    name: String,
    source_id: scru128::Scru128Id,
) {
    let mut state = state.lock().unwrap();

    let packet = state
        .store
        .add(name.as_bytes(), MimeType::TextPlain, None, None);
    state.merge(packet.clone());

    let packet = state.store.fork(
        source_id,
        None,
        MimeType::TextPlain,
        Some(packet.id()),
        None,
    );
    state.merge(packet);

    app.emit_all("refresh-items", true).unwrap();
}
