use tokio::io::AsyncReadExt;

use tauri::Manager;

use scru128::Scru128Id;

use crate::content_type::process_command;
use crate::state::SharedState;
use crate::store::{
    InProgressStream, MimeType, Movement, Settings, StackLockStatus, StackSortOrder,
};
use crate::ui::{generate_preview, with_meta, Item as UIItem, Nav, UI};
use crate::view::View;

#[derive(Debug, Clone, serde::Serialize)]
struct ExecStatus {
    exec_id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    out: Option<Cacheable>,
    #[serde(skip_serializing_if = "Option::is_none")]
    err: Option<Cacheable>,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<i32>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct Cacheable {
    id: Scru128Id,
    hash: Option<ssri::Integrity>,
    ephemeral: bool,
}

#[tauri::command]
#[tracing::instrument(skip(state, app))]
pub async fn store_pipe_to_command(
    state: tauri::State<'_, SharedState>,
    app: tauri::AppHandle,
    exec_id: u32,
    source_id: scru128::Scru128Id,
    command: String,
) -> Result<(), ()> {
    let (cache_path, hash, stack_id) = state.with_lock(|state| {
        let cache_path = state.store.cache_path.clone();
        let item = state.view.items.get(&source_id).unwrap();
        (cache_path, item.hash.clone(), item.stack_id)
    });

    let (cooked_command, content_type) = process_command(&command);

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
    let rc_command = format!(
        "source {}\n{}",
        rc_path.to_str().unwrap_or(""),
        cooked_command
    );

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

    let mut stdout = cmd.stdout.take().unwrap();

    let read_stdout = {
        let state = state.inner().clone();
        let app = app.clone();

        tokio::spawn(async move {
            let mut buffer = [0u8; 4096];
            let size = stdout.read(&mut buffer).await.unwrap();

            // stdout is empty
            if size == 0 {
                return;
            }

            let m = infer::Infer::new().get(&buffer[..size]);
            let (mime_type, content_type_2) = match m.map(|m| m.mime_type()) {
                None => (
                    MimeType::TextPlain,
                    content_type.clone().unwrap_or("Text".to_string()),
                ),
                Some("image/png") => (MimeType::ImagePng, "Image".to_string()),
                _ => todo!(),
            };

            let mut streamer = state.with_lock(|state| {
                let stack = state.get_curr_stack();
                let mut streamer = InProgressStream::new(stack, mime_type.clone(), content_type_2);
                if mime_type == MimeType::TextPlain {
                    state.merge(&streamer.packet);
                    app.emit_all("refresh-items", true).unwrap();
                }
                streamer.append(&buffer[..size]);
                streamer
            });

            app.emit_all(
                "pipe-to-shell",
                ExecStatus {
                    exec_id,
                    out: Some(Cacheable {
                        id: streamer.packet.id,
                        hash: None,
                        ephemeral: true,
                    }),
                    err: None,
                    code: None,
                },
            )
            .unwrap();

            loop {
                match stdout.read(&mut buffer).await {
                    Ok(size) => {
                        if size == 0 {
                            break; // End of stream
                        }
                        streamer.append(&buffer[..size]);

                        if mime_type == MimeType::TextPlain {
                            let preview = generate_preview(
                                "dark",
                                &Some(streamer.content.clone()),
                                &streamer.content_meta.mime_type,
                                &streamer.content_meta.content_type,
                                true,
                            );
                            let content = String::from_utf8_lossy(&streamer.content);
                            let content = Content {
                                mime_type: streamer.content_meta.mime_type.clone(),
                                content_type: streamer.content_meta.content_type.clone(),
                                terse: content.chars().take(100).collect(),
                                tiktokens: 0,
                                words: content.split_whitespace().count(),
                                chars: content.chars().count(),
                                preview,
                            };

                            app.emit_all("streaming", (streamer.packet.id, content))
                                .unwrap();
                        }
                    }
                    Err(e) => {
                        tracing::error!("Error reading bytes from command stdout: {}", e);
                        break;
                    }
                }
            }

            state.with_lock(|state| {
                let packet = streamer.end_stream(&mut state.store);
                state.merge(&packet);
                state.store.insert_packet(&packet);
                app.emit_all(
                    "pipe-to-shell",
                    ExecStatus {
                        exec_id,
                        out: Some(Cacheable {
                            id: packet.id,
                            hash: packet.hash,
                            ephemeral: false,
                        }),
                        err: None,
                        code: None,
                    },
                )
                .unwrap();
            });
            app.emit_all("refresh-items", true).unwrap();
        })
    };

    let mut stderr = cmd.stderr.take().unwrap();
    let mut buff = Vec::new();
    stderr.read_to_end(&mut buff).await.unwrap();
    let stderr = buff;
    if !stderr.is_empty() {
        state.with_lock(|state| {
            let stack_id = stack_id.unwrap_or_else(|| state.get_curr_stack());
            let packet = state.store.add(&stderr, MimeType::TextPlain, stack_id);
            state.merge(&packet);
            app.emit_all(
                "pipe-to-shell",
                ExecStatus {
                    exec_id,
                    out: None,
                    err: Some(Cacheable {
                        id: packet.id,
                        hash: packet.hash,
                        ephemeral: false,
                    }),
                    code: None,
                },
            )
            .unwrap();
        })
    }

    let status = cmd.wait().await.unwrap();

    let _ = read_stdout.await.expect("Task failed");
    app.emit_all(
        "pipe-to-shell",
        ExecStatus {
            exec_id,
            out: None,
            err: None,
            code: status.code(),
        },
    )
    .unwrap();

    state.with_lock(|state| {
        let stack_id = stack_id.unwrap_or_else(|| state.get_curr_stack());
        let packet = state
            .store
            .add(command.as_bytes(), MimeType::TextPlain, stack_id);
        state.merge(&packet);
        let packet = state
            .store
            .update_content_type(packet.hash.unwrap(), "Shell".to_string());
        state.merge(&packet);
    });

    app.emit_all("refresh-items", true).unwrap();
    Ok(())
}

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
            "Source Code" => "Source Code",
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
pub fn store_nav_select_down_stack(state: tauri::State<SharedState>) -> Nav {
    state.with_lock(|state| {
        state.ui.select_down_stack();
        state.ui.render(&state.store)
    })
}

#[tauri::command]
#[tracing::instrument(skip(state))]
pub fn store_nav_select_up_stack(state: tauri::State<SharedState>) -> Nav {
    state.with_lock(|state| {
        state.ui.select_up_stack();
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
    win.set_size(tauri::PhysicalSize::new(1954, 978)).unwrap();
    win.set_position(tauri::PhysicalPosition::new(722, 678))
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
        let meta = state
            .view
            .items
            .get(&source_id)
            .and_then(|source| state.store.get_content_meta(&source.hash));
        if meta.is_none() {
            tracing::warn!("source or meta not found");
            return;
        }
        let meta = meta.unwrap();

        let packet = state.store.update(
            source_id,
            Some(content.as_bytes()),
            MimeType::TextPlain,
            None,
        );
        state.merge(&packet);

        if let Some(hash) = packet.hash {
            if meta.content_type != "Text" {
                let packet = state.store.update_content_type(hash, meta.content_type);
                state.merge(&packet);
            }
        }

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
    app.emit_all("content", hash).unwrap();
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
