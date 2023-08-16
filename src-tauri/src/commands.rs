// use tauri::Manager;

// use base64::{engine::general_purpose, Engine as _};

use crate::state::SharedState;


/*
#[derive(Debug, serde::Serialize)]
pub struct CommandOutput {
    pub out: String,
    pub err: String,
    pub code: i32,
}

#[tauri::command]
pub async fn store_pipe_to_command(
    state: tauri::State<'_, SharedState>,
    hash: ssri::Integrity,
    command: String,
) -> Result<CommandOutput, ()> {
    println!("PIPE: {} {}", &hash, &command);
    let cache_path = {
        let state = state.lock().unwrap();
        state.store.cache_path.clone()
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
        .cat(&hash)
        .map(|vec| general_purpose::STANDARD.encode(vec))
}
*/

#[tauri::command]
pub fn store_list_items(
    state: tauri::State<SharedState>,
    stack: Option<ssri::Integrity>,
    filter: String,
    content_type: String,
) -> String {
    let state = state.lock().unwrap();
    serde_json::to_string(&*state).unwrap()
}

/*

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
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    stack_hash: Option<ssri::Integrity>,
    source_id: scru128::Scru128Id,
) -> Option<()> {
    let mut state = state.lock().unwrap();
    let mut frame = match state.store.get_frame(&source_id) {
        Some(frame) => frame,
        None => {
            log::warn!("No frame found with id: {:?}", source_id);
            return None;
        }
    };
    let content = state.store.cat(&frame.hash)?;

    let mime_type = match &frame.mime_type {
        MimeType::TextPlain => "public.utf8-plain-text",
        MimeType::ImagePng => "public.png",
    };

    let change_num = write_to_clipboard(mime_type, &content)?;
    state.skip_change_num = Some(change_num);

    frame.id = scru128::new();
    frame.source = Some("stream.cross.stacks".into());
    frame.stack_hash = stack_hash;
    let packet = state.store.insert_frame(&frame);
    state.merge(&packet);

    app.emit_all("refresh-items", true).unwrap();
    Some(())
}

#[tauri::command]
pub fn store_capture(
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    stack_hash: Option<ssri::Integrity>,
    content: String,
) {
    let mut state = state.lock().unwrap();
    let content = content.as_bytes();
    state.add_content(
        Some("stream.cross.stacks".into()),
        stack_hash,
        MimeType::TextPlain,
        content,
    );

    let change_num = write_to_clipboard("public.utf8-plain-text", content).unwrap();
    state.skip_change_num = Some(change_num);

    app.emit_all("refresh-items", true).unwrap();
}

#[tauri::command]
pub fn store_delete(
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    hash: ssri::Integrity,
    stack_hash: Option<ssri::Integrity>,
) {
    let mut state = state.lock().unwrap();
    let packet = state.store.delete(&hash, &stack_hash);
    state.merge(&packet);
    app.emit_all("refresh-items", true).unwrap();
}
*/

//
// Stack related commands

#[tauri::command]
pub fn store_set_current_stack(
    state: tauri::State<SharedState>,
    stack_id: Option<scru128::Scru128Id>,
) {
    let mut state = state.lock().unwrap();
    state.curr_stack = stack_id;
}

/*
#[tauri::command]
pub fn store_add_to_stack(
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    name: String,
    id: scru128::Scru128Id,
) {
    let mut state = state.lock().unwrap();

    let stack_frame = state.add_content(
        Some("stream.cross.stacks".into()),
        None,
        MimeType::TextPlain,
        name.as_bytes(),
    );

    let mut frame = match state.store.get_frame(&id) {
        Some(frame) => frame,
        None => {
            log::warn!("No frame found with id: {:?}", id);
            return;
        }
    };

    frame.id = scru128::new();
    frame.source = Some("stream.cross.stacks".into());
    frame.stack_hash = Some(stack_frame.hash);
    let packet = state.store.insert_frame(&frame);
    state.merge(&packet);
    app.emit_all("refresh-items", true).unwrap();
}

*/

#[tauri::command]
// s/String/Item
pub fn store_list_stacks(filter: String, state: tauri::State<SharedState>) -> Vec<String> {
    let state = state.lock().unwrap();

    return Vec::new();

    /*
    let mut ret: Vec<Item> = state
        .stack
        .items
        .values()
        .filter(|item| {
            if &item.content_type != "Stack" {
                return false;
            }

            if filter == filter.to_lowercase() {
                item.terse.to_lowercase().contains(&filter)
            } else {
                item.terse.contains(&filter)
            }
        })
        .cloned()
        .collect();
    ret.sort_unstable_by(|a, b| b.ids.last().cmp(&a.ids.last()));
    ret.truncate(400);
    ret
    */
}

/*

#[tauri::command]
pub fn store_copy_entire_stack_to_clipboard(
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    stack_hash: ssri::Integrity,
) -> Option<()> {
    let mut state = state.lock().unwrap();

    let stack = state.stack.items.get(&stack_hash)?.clone();

    let mut items: Vec<&Item> = stack
        .stack
        .values()
        .filter(|item| !item.ids.is_empty())
        .collect();

    items.sort_by(|a, b| b.ids.last().cmp(&a.ids.last()));

    let content: Vec<String> = items
        .into_iter()
        .filter(|item| item.mime_type == MimeType::TextPlain)
        .map(|item| item.hash.clone())
        .filter_map(|hash| state.store.cat(&hash))
        .map(|bytes| String::from_utf8(bytes).unwrap_or_default())
        .collect();

    let content = content.join("\n");
    let change_num = write_to_clipboard("public.utf8-plain-text", content.as_bytes())?;
    state.skip_change_num = Some(change_num);

    let frame = Frame {
        id: scru128::new(),
        source: Some("stream.cross.stacks".into()),
        stack_hash: None,
        mime_type: MimeType::TextPlain,
        hash: stack.hash.clone(),
    };
    let packet = state.store.insert_frame(&frame);
    state.merge(&packet);

    app.emit_all("refresh-items", true).unwrap();
    Some(())
}
// End stack commands
*/
