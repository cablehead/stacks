use tauri::Manager;

use crate::stack::Item;
use crate::state::SharedState;
use crate::store::MimeType;

use base64::{engine::general_purpose, Engine as _};

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

#[tauri::command]
pub fn store_list_items(
    state: tauri::State<SharedState>,
    stack: Option<ssri::Integrity>,
    filter: String,
    content_type: String,
) -> Vec<Item> {
    let state = state.lock().unwrap();
    println!("FILTER : {:?} {} {}", &stack, &filter, &content_type);
    let filter = if filter.is_empty() {
        None
    } else {
        Some(filter)
    };
    let content_type = if content_type == "All" {
        None
    } else {
        let mut content_type = content_type;
        content_type.truncate(content_type.len() - 1);
        Some(content_type)
    };

    let base_items: Vec<Item> = if let Some(hash) = stack {
        let item = state.stack.items.get(&hash).unwrap();
        item.stack.values().cloned().collect()
    } else {
        state.stack.items.values().cloned().collect()
    };

    let mut recent_items: Vec<Item> = base_items
        .iter()
        .filter(|item| {
            if let Some(curr) = &filter {
                // match case insensitive, unless the filter has upper case, in which, match case
                // sensitive
                if curr == &curr.to_lowercase() {
                    item.terse.to_lowercase().contains(curr)
                } else {
                    item.terse.contains(curr)
                }
            } else {
                true
            }
        })
        .filter(|item| {
            if let Some(content_type) = &content_type {
                &item.content_type == content_type
            } else {
                true
            }
        })
        .cloned()
        .collect();
    recent_items.sort_unstable_by(|a, b| b.ids.last().cmp(&a.ids.last()));
    recent_items.truncate(400);
    recent_items
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
    app: tauri::AppHandle,
    state: tauri::State<SharedState>,
    stack_hash: Option<ssri::Integrity>,
    source_id: scru128::Scru128Id,
) -> Option<()> {
    let mut state = state.lock().unwrap();
    let mut frame = state.store.get(&source_id)?;
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
    state.store.insert(&frame);
    state.merge(&frame);

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

/*
#[tauri::command]
pub fn store_delete(app: tauri::AppHandle, hash: String, store: tauri::State<SharedStore>) {
    let mut store = store.lock().unwrap();
    println!("DEL: {}", &hash);
    if let Some(item) = store.items.remove(&hash) {
        println!("item: {:?}", item);
        let env = xs_lib::store_open(&store.db_path).unwrap();
        xs_lib::store_delete(&env, item.ids).unwrap();
    }
    store.cas.remove(&hash);
    app.emit_all("refresh-items", true).unwrap();
}
*/

//
// Stack related commands

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

    let mut frame = state.store.get(&id).unwrap();
    frame.id = scru128::new();
    frame.source = Some("stream.cross.stacks".into());
    frame.stack_hash = Some(stack_frame.hash);
    state.store.insert(&frame);
    state.merge(&frame);
    app.emit_all("refresh-items", true).unwrap();
}

/*
#[tauri::command]
pub fn store_delete_from_stack(name: String, id: String, store: tauri::State<SharedStore>) {
    let store = store.lock().unwrap();
    let data = serde_json::json!({
        "name": name,
        "id": id
    })
    .to_string();
    println!("DELETE FROM STACK: {}", &data);
    let env = xs_lib::store_open(&store.db_path).unwrap();
    xs_lib::store_put(&env, Some("stack".into()), Some("delete".into()), data).unwrap();
}
*/

#[tauri::command]
pub fn store_list_stacks(filter: String, state: tauri::State<SharedState>) -> Vec<Item> {
    let state = state.lock().unwrap();

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
}

// End stack commands
