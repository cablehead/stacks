use tauri::Manager;

use scru128::Scru128Id;

use crate::state::SharedState;
use crate::store::{MimeType, Movement, Settings, StackLockStatus, StackSortOrder};
use crate::ui::{generate_preview, with_meta, Item as UIItem, Nav, UI};
use crate::view::View;

#[derive(Debug, serde::Serialize)]
pub struct CommandOutput {
    pub out: String,
    pub err: String,
    pub code: i32,
}

#[tauri::command]
#[tracing::instrument(skip(state))]
pub async fn store_pipe_to_command(
    state: tauri::State<'_, SharedState>,
    source_id: scru128::Scru128Id,
    command: String,
) -> Result<CommandOutput, ()> {
    let (cache_path, hash) = state.with_lock(|state| {
        let cache_path = state.store.cache_path.clone();
        let item = state.view.items.get(&source_id).unwrap();
        (cache_path, item.hash.clone())
    });

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
    tokio::spawn(async move {
        tokio::io::copy(&mut reader, &mut stdin).await.unwrap();
    });

    let output = cmd.wait_with_output().await.unwrap();
    let output = CommandOutput {
        out: String::from_utf8_lossy(&output.stdout).into_owned(),
        err: String::from_utf8_lossy(&output.stderr).into_owned(),
        code: output.status.code().unwrap_or(-1),
    };
    Ok(output)
}

/*
#[tauri::command]
#[tracing::instrument(skip(app, state))]
pub async fn store_pipe_to_gpt(
    state: tauri::State<'_, SharedState>,
    app: tauri::AppHandle,
    source_id: scru128::Scru128Id,
) -> Result<(), ()> {
    let (settings, content, packet) = {
        let mut state = state.lock().unwrap();

        let settings = state.store.settings_get().ok_or(())?.clone();
        let item = state.view.items.get(&source_id).ok_or(())?;

        let content = if let Some(_) = item.stack_id {
            vec![state.store.get_content(&item.hash).unwrap()]
        } else {
            return Ok(());
        };

        let stack_id = item.stack_id.unwrap_or(item.id);
        let packet = state.store.start_stream(Some(stack_id), "".as_bytes());
        state.ui.select(None); // focus first

        (settings, content, packet)
    };

    #[derive(Clone, serde::Serialize)]
    struct Payload {
        id: Scru128Id,
        tiktokens: usize,
        content: String,
    }

    use async_openai::{
        config::OpenAIConfig,
        types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
        Client,
    };
    use futures::StreamExt;

    let config = OpenAIConfig::new().with_api_key(settings.openai_access_token);

    let client = Client::with_config(config);

    let messages: Vec<_> = content
        .into_iter()
        .map(|c| {
            ChatCompletionRequestMessageArgs::default()
                .content(String::from_utf8(c).unwrap())
                .role(Role::User)
                .build()
                .unwrap()
        })
        .collect();

    let request = CreateChatCompletionRequestArgs::default()
        .model(&settings.openai_selected_model)
        .max_tokens(512u16)
        .messages(messages)
        .build()
        .unwrap();

    let mut stream = client.chat().create_stream(request).await.unwrap();

    let mut packet = packet;

    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => {
                let aggregate = response
                    .choices
                    .iter()
                    .filter_map(|chat_choice| chat_choice.delta.content.as_ref())
                    .flat_map(|content| content.as_bytes().iter().cloned())
                    .collect::<Vec<u8>>();

                {
                    let mut state = state.lock().unwrap();
                    packet = state.store.update_stream(packet.id, &aggregate);
                    state.merge(&packet);
                    app.emit_all("refresh-items", true).unwrap();
                }
            }
            Err(err) => {
                error!("GPT error: {:#?}", err);
            }
        }
    }

    let mut state = state.lock().unwrap();
    packet = state.store.end_stream(packet.id);
    state.merge(&packet);
    app.emit_all("refresh-items", true).unwrap();

    Ok(())
}
*/

fn truncate_hash(hash: &ssri::Integrity, len: usize) -> String {
    hash.hashes.first().map_or_else(
        || "No hash present".to_owned(),
        |h| h.digest.chars().take(len).collect(),
    )
}

use base64::{engine::general_purpose, Engine as _};

#[tauri::command]
#[tracing::instrument(skip(state), fields(%hash = truncate_hash(&hash, 8)))]
pub fn store_get_raw_content(
    state: tauri::State<SharedState>,
    hash: ssri::Integrity,
) -> Option<String> {
    state.with_lock(|state| {
        state
            .store
            .get_content(&hash)
            .map(|vec| general_purpose::STANDARD.encode(vec))
    })
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct Content {
    pub mime_type: MimeType,
    pub content_type: String,
    pub terse: String,
    pub tiktokens: usize,
    pub words: usize,
    pub chars: usize,
    pub preview: String,
}

#[tauri::command]
#[tracing::instrument(skip(state), fields(%hash = truncate_hash(&hash, 8)))]
pub fn store_get_content(state: tauri::State<SharedState>, hash: ssri::Integrity) -> Content {
    state.with_lock(|state| {
        let content = state.store.get_content(&hash);
        let meta = state.store.get_content_meta(&hash).unwrap();

        let (words, chars) = match (&meta.mime_type, &content) {
            (MimeType::TextPlain, Some(bytes)) => {
                let str_slice = std::str::from_utf8(bytes).expect("Invalid UTF-8");
                (
                    str_slice.split_whitespace().count(),
                    str_slice.chars().count(),
                )
            }
            _ => (0, 0),
        };

        let preview = generate_preview(
            &state.ui.theme_mode,
            &content,
            &meta.mime_type,
            &meta.content_type,
            false,
        );

        Content {
            mime_type: meta.mime_type,
            content_type: meta.content_type,
            terse: meta.terse,
            tiktokens: meta.tiktokens,
            words,
            chars,
            preview,
        }
    })
}

#[tauri::command]
#[tracing::instrument(skip(state))]
pub fn store_get_root(state: tauri::State<SharedState>) -> Vec<UIItem> {
    state.with_lock(|state| {
        state
            .view
            .root()
            .iter()
            .map(|item| with_meta(&state.store, item))
            .collect()
    })
}

#[tauri::command]
#[tracing::instrument(skip(state))]
pub fn store_nav_refresh(state: tauri::State<SharedState>) -> Nav {
    state.with_lock(|state| state.ui.render(&state.store))
}

#[tauri::command]
#[tracing::instrument(skip(state))]
pub fn store_nav_reset(state: tauri::State<SharedState>) -> Nav {
    state.with_lock(|state| {
        let view = state.view.clone();
        state.ui.reset(view);
        state.ui.render(&state.store)
    })
}

#[tauri::command]
#[tracing::instrument(skip(state))]
pub fn store_nav_set_filter(
    state: tauri::State<SharedState>,
    filter: String,
    content_type: String,
) -> Nav {
    state.with_lock(|state| {
        // XXX: content_type should be an enum
        let content_type = match content_type.as_str() {
            "Links" => "Link",
            "Images" => "Image",
            "Markdown" => "Markdown",
            _ => "All",
        };
        state.nav_set_filter(&filter, content_type);
        state.ui.render(&state.store)
    })
}

#[tauri::command]
#[tracing::instrument(skip(state))]
pub fn store_nav_select(state: tauri::State<SharedState>, focused_id: Scru128Id) -> Nav {
    state.with_lock(|state| {
        state.nav_select(&focused_id);
        state.ui.render(&state.store)
    })
}

#[tauri::command]
#[tracing::instrument(skip(state))]
pub fn store_nav_select_up(state: tauri::State<SharedState>) -> Nav {
    state.with_lock(|state| {
        state.ui.select_up();
        state.ui.render(&state.store)
    })
}

#[tauri::command]
#[tracing::instrument(skip(state))]
pub fn store_nav_select_down(state: tauri::State<SharedState>) -> Nav {
    state.with_lock(|state| {
        state.ui.select_down();
        state.ui.render(&state.store)
    })
}

#[tauri::command]
#[tracing::instrument(skip(state))]
pub fn store_nav_select_left(state: tauri::State<SharedState>) -> Nav {
    state.with_lock(|state| {
        state.ui.select_left();
        state.ui.render(&state.store)
    })
}

#[tauri::command]
#[tracing::instrument(skip(state))]
pub fn store_nav_select_right(state: tauri::State<SharedState>) -> Nav {
    state.with_lock(|state| {
        state.ui.select_right();
        state.ui.render(&state.store)
    })
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
#[tracing::instrument(skip(state))]
pub fn store_copy_to_clipboard(
    state: tauri::State<SharedState>,
    source_id: scru128::Scru128Id,
) -> Option<()> {
    state.with_lock(|state| {
        if let Some(item) = state.view.items.get(&source_id) {
            let meta = state.store.get_content_meta(&item.hash).unwrap();

            let mime_type = match &meta.mime_type {
                MimeType::TextPlain => "public.utf8-plain-text",
                MimeType::ImagePng => "public.png",
            };
            let content = state.store.get_content(&item.hash).unwrap();

            let _change_num = write_to_clipboard(mime_type, &content);
            Some(())
        } else {
            None
        }
    })
}

#[tauri::command]
#[tracing::instrument(skip(app, state))]
pub fn store_new_note(
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    content: String,
    stack_id: Option<scru128::Scru128Id>,
) {
    state.with_lock(|state| {
        let stack_id = stack_id.unwrap_or_else(|| state.get_curr_stack());

        let packet = state
            .store
            .add(content.as_bytes(), MimeType::TextPlain, stack_id);

        let id = packet.id;
        state.merge(&packet);

        let focus = state.view.get_focus_for_id(&id);

        state.ui.select(focus);

        state.skip_change_num = write_to_clipboard("public.utf8-plain-text", content.as_bytes());
    });
    app.emit_all("refresh-items", true).unwrap();
}

#[tauri::command]
#[tracing::instrument(skip(app))]
pub fn store_win_move(app: tauri::AppHandle) {
    let win = app.get_window("main").unwrap();
    // use tauri_plugin_positioner::{Position, WindowExt};
    // let _ = win.move_window(Position::TopRight);
    win.set_size(tauri::PhysicalSize::new(1920, 1080))
        .unwrap();
}

#[tauri::command]
#[tracing::instrument(skip(app, state))]
pub fn store_edit_note(
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    source_id: scru128::Scru128Id,
    content: String,
) {
    state.with_lock(|state| {
        let packet = state.store.update(
            source_id,
            Some(content.as_bytes()),
            MimeType::TextPlain,
            None,
        );
        state.merge(&packet);

        state.skip_change_num = write_to_clipboard("public.utf8-plain-text", content.as_bytes());
    });
    app.emit_all("refresh-items", true).unwrap();
}

#[tauri::command]
#[tracing::instrument(skip(app, state))]
pub fn store_touch(
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    source_id: scru128::Scru128Id,
) {
    state.with_lock(|state| {
        let packet = state.store.update_touch(source_id);
        state.merge(&packet);
    });
    app.emit_all("refresh-items", true).unwrap();
}

#[tauri::command]
#[tracing::instrument(skip(app, state))]
pub fn store_set_content_type(
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    hash: ssri::Integrity,
    content_type: String,
) {
    state.with_lock(|state| {
        let packet = state.store.update_content_type(
            hash.clone(),
            if content_type == "Plain Text" {
                "Text".to_string()
            } else {
                content_type
            },
        );
        state.merge(&packet);
    });

    let content = store_get_content(state, hash.clone());
    app.emit_all("content", (hash, content)).unwrap();
}

#[tauri::command]
#[tracing::instrument(skip(app, state))]
pub fn store_set_theme_mode(app: tauri::AppHandle, state: tauri::State<SharedState>, mode: String) {
    state.with_lock(|state| {
        state.ui.theme_mode = mode;
    });
    app.emit_all("refresh-items", true).unwrap();
}

#[tauri::command]
#[tracing::instrument(skip(app, state))]
pub fn store_delete(
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    id: scru128::Scru128Id,
) {
    state.with_lock(|state| {
        let packet = state.store.delete(id);
        state.merge(&packet);
    });
    app.emit_all("refresh-items", true).unwrap();
}

#[tauri::command]
#[tracing::instrument(skip(app, state))]
pub fn store_undo(app: tauri::AppHandle, state: tauri::State<SharedState>) {
    state.with_lock(|state| {
        if let Some(item) = state.view.undo.clone() {
            state.store.remove_packet(&item.last_touched);
            let mut view = View::new();
            state.store.scan().for_each(|p| view.merge(&p));
            let mut ui = UI::new(&view);
            ui.select(view.get_focus_for_id(&item.id));
            state.view = view;
            state.ui = ui;
        }
    });
    app.emit_all("refresh-items", true).unwrap();
}

//
// Settings related commands

#[tauri::command]
#[tracing::instrument(skip(state))]
pub fn store_settings_save(state: tauri::State<SharedState>, settings: Settings) {
    state.with_lock(|state| {
        state.store.settings_save(settings);
    });
}

#[tauri::command]
#[tracing::instrument(skip(state))]
pub fn store_settings_get(state: tauri::State<SharedState>) -> Option<Settings> {
    state.with_lock(|state| state.store.settings_get())
}

//
// Stack related commands

#[tauri::command]
#[tracing::instrument(skip(app, state))]
pub fn store_add_to_stack(
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    stack_id: scru128::Scru128Id,
    source_id: scru128::Scru128Id,
) {
    state.with_lock(|state| {
        let packet = state
            .store
            .fork(source_id, None, MimeType::TextPlain, Some(stack_id));
        state.merge(&packet);
    });
    app.emit_all("refresh-items", true).unwrap();
}

#[tauri::command]
#[tracing::instrument(skip(app, state))]
pub fn store_new_stack(app: tauri::AppHandle, state: tauri::State<SharedState>, name: String) {
    state.with_lock(|state| {
        let packet = state
            .store
            .add_stack(name.as_bytes(), StackLockStatus::Unlocked);
        state.merge(&packet);
        state.ui.select(None); // focus first
    });
    app.emit_all("refresh-items", true).unwrap();
}

#[tauri::command]
#[tracing::instrument(skip(app, state))]
pub fn store_add_to_new_stack(
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    name: String,
    source_id: scru128::Scru128Id,
) {
    state.with_lock(|state| {
        let stack_packet = state
            .store
            .add_stack(name.as_bytes(), StackLockStatus::Locked);
        state.merge(&stack_packet);

        let item_packet =
            state
                .store
                .fork(source_id, None, MimeType::TextPlain, Some(stack_packet.id));
        state.merge(&item_packet);
    });
    app.emit_all("refresh-items", true).unwrap();
}

#[tauri::command]
#[tracing::instrument(skip(app, state))]
pub fn store_move_up(
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    source_id: scru128::Scru128Id,
) {
    state.with_lock(|state| {
        let packet = state.store.update_move(source_id, Movement::Up);
        state.merge(&packet);
    });
    app.emit_all("refresh-items", true).unwrap();
}

#[tauri::command]
#[tracing::instrument(skip(app, state))]
pub fn store_mark_as_cross_stream(
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    stack_id: scru128::Scru128Id,
) {
    state.with_lock(|state| {
        let packet = state.store.mark_as_cross_stream(stack_id);
        state.merge(&packet);
    });
    app.emit_all("refresh-items", true).unwrap();
}

#[tauri::command]
#[tracing::instrument(skip(app, state))]
pub fn store_move_down(
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    source_id: scru128::Scru128Id,
) {
    state.with_lock(|state| {
        let packet = state.store.update_move(source_id, Movement::Down);
        state.merge(&packet);
    });
    app.emit_all("refresh-items", true).unwrap();
}

#[tauri::command]
#[tracing::instrument(skip(app, state))]
pub fn store_stack_lock(
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    source_id: scru128::Scru128Id,
) {
    state.with_lock(|state| {
        let packet = state
            .store
            .update_stack_lock_status(source_id, StackLockStatus::Locked);
        state.merge(&packet);
    });
    app.emit_all("refresh-items", true).unwrap();
}

#[tauri::command]
#[tracing::instrument(skip(app, state))]
pub fn store_stack_unlock(
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    source_id: scru128::Scru128Id,
) {
    state.with_lock(|state| {
        let packet = state
            .store
            .update_stack_lock_status(source_id, StackLockStatus::Unlocked);
        state.merge(&packet);
    });
    app.emit_all("refresh-items", true).unwrap();
}

#[tauri::command]
#[tracing::instrument(skip(app, state))]
pub fn store_stack_sort_manual(
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    source_id: scru128::Scru128Id,
) {
    state.with_lock(|state| {
        let packet = state
            .store
            .update_stack_sort_order(source_id, StackSortOrder::Manual);
        state.merge(&packet);
    });
    app.emit_all("refresh-items", true).unwrap();
}

#[tauri::command]
#[tracing::instrument(skip(app, state))]
pub fn store_stack_sort_auto(
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    source_id: scru128::Scru128Id,
) {
    state.with_lock(|state| {
        let packet = state
            .store
            .update_stack_sort_order(source_id, StackSortOrder::Auto);
        state.merge(&packet);
    });
    app.emit_all("refresh-items", true).unwrap();
}
