use tauri::Manager;

use base64::{engine::general_purpose, Engine as _};

use scru128::Scru128Id;

use crate::state::SharedState;
use crate::store::{MimeType, Settings};
use crate::ui::{with_meta, Item as UIItem, Nav, UI};
use crate::view::View;

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
pub async fn store_pipe_to_gpt(
    _app: tauri::AppHandle,
    _state: tauri::State<'_, SharedState>,
    _source_id: scru128::Scru128Id,
) -> Result<(), ()> {
    /*
        let (packet, settings, content) = {
            let mut state = state.lock().unwrap();

            let settings = state.store.settings_get().ok_or(())?.clone();
            let item = state.view.items.get(&source_id).ok_or(())?;
            let stack_id = item.stack_id.ok_or(())?;

            println!("GPT: {:?} {:?}", source_id, stack_id);

            let hash = item.hash.clone().ok_or(())?;

            let meta = state.store.get_content_meta(&hash).unwrap();
            if meta.mime_type != MimeType::TextPlain {
                return Ok(());
            }

            let content = state.store.cas_read(&hash).unwrap();

            let packet = Packet::Add(AddPacket {
                id: scru128::new(),
                hash: None,
                stack_id: Some(stack_id),
                source: None,
            });

            state.merge(packet.clone());

            let item = state.view.items.get(&packet.id()).cloned();
            state.ui.select(item.as_ref());
            app.emit_all("refresh-items", true).unwrap();
            (packet, settings, content)
        };

        #[derive(Clone, serde::Serialize)]
        struct Payload {
            id: Scru128Id,
            tiktokens: usize,
            content: String,
        }

        use async_openai::{
            types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
            Client, config::OpenAIConfig,
        };
        use futures::StreamExt;

        let config = OpenAIConfig::new().with_api_key(settings.openai_access_token);

        let client = Client::with_config(config);

        let request = CreateChatCompletionRequestArgs::default()
            .model(&settings.openai_selected_model)
            .max_tokens(512u16)
            .messages([ChatCompletionRequestMessageArgs::default()
                .content(String::from_utf8(content).unwrap())
                .role(Role::User)
                .build()
                .unwrap()])
            .build()
            .unwrap();

        let mut stream = client.chat().create_stream(request).await.unwrap();

        let mut aggregate = String::new();

        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    response.choices.iter().for_each(|chat_choice| {
                        if let Some(ref content) = chat_choice.delta.content {
                            aggregate.push_str(content);
                        }
                    });

                    app.emit_all(
                        "foo",
                        Payload {
                            id: packet.id(),
                            tiktokens: count_tiktokens(&aggregate),
                            content: general_purpose::STANDARD.encode(&aggregate),
                        },
                    )
                    .unwrap();
                }
                Err(err) => {
                    println!("GPT error: {:#?}", err);
                }
            }
        }
    */

    Ok(())
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
pub fn store_get_root(state: tauri::State<SharedState>) -> Vec<UIItem> {
    let state = state.lock().unwrap();
    state
        .view
        .root()
        .iter()
        .map(|item| with_meta(&state.store, item))
        .collect()
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
    );

    let id = packet.id;
    state.merge(packet);
    state.ui.focused = state.view.items.get(&id).cloned();

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

#[tauri::command]
pub fn store_undo(app: tauri::AppHandle, state: tauri::State<SharedState>) {
    let mut state = state.lock().unwrap();
    if let Some(item) = state.view.undo.clone() {
        state.store.remove_packet(&item.last_touched);
        let mut view = View::new();
        state.store.scan().for_each(|p| view.merge(p));
        let mut ui = UI::new(&view);
        ui.select(view.items.get(&item.id));
        state.view = view;
        state.ui = ui;
        app.emit_all("refresh-items", true).unwrap();
    }
}

//
// Settings related commands

#[tauri::command]
pub fn store_settings_save(state: tauri::State<SharedState>, settings: Settings) {
    let mut state = state.lock().unwrap();
    state.store.settings_save(settings);
}

#[tauri::command]
pub fn store_settings_get(state: tauri::State<SharedState>) -> Option<Settings> {
    let state = state.lock().unwrap();
    state.store.settings_get()
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
        .fork(source_id, None, MimeType::TextPlain, Some(stack_id));

    let id = packet.id;
    state.merge(packet);
    state.ui.focused = state.view.items.get(&id).cloned();

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
        .add(name.as_bytes(), MimeType::TextPlain, None);
    state.merge(packet.clone());

    let packet = state.store.fork(
        source_id,
        None,
        MimeType::TextPlain,
        Some(packet.id),
    );

    let id = packet.id;
    state.merge(packet);
    state.ui.focused = state.view.items.get(&id).cloned();

    app.emit_all("refresh-items", true).unwrap();
}
