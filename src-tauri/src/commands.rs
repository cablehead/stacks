use crate::store::{Item, SharedStore};

#[tauri::command]
pub fn store_get_content(hash: String, store: tauri::State<SharedStore>) -> Option<String> {
    println!("CACHE MISS: {}", &hash);
    let store = store.lock().unwrap();
    store.cat(&hash)
}

#[tauri::command]
pub fn store_list_stacks(filter: String, store: tauri::State<SharedStore>) -> Vec<Item> {
    let store = store.lock().unwrap();

    let mut ret: Vec<Item> = store
        .items
        .values()
        .filter(|item| {
            if &item.content_type != "Stack" {
                return false;
            }

            return if filter == filter.to_lowercase() {
                item.terse.to_lowercase().contains(&filter)
            } else {
                item.terse.contains(&filter)
            };
        })
        .cloned()
        .collect();
    ret.sort_unstable_by(|a, b| b.ids.last().cmp(&a.ids.last()));
    ret.truncate(400);
    ret
}

#[tauri::command]
pub fn store_list_items(
    stack: Option<String>,
    filter: String,
    content_type: String,
    store: tauri::State<SharedStore>,
) -> Vec<Item> {
    let store = store.lock().unwrap();
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
        let item = store.items.get(&hash).unwrap();
        item.stack.values().cloned().collect()
    } else {
        store.items.values().cloned().collect()
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

#[tauri::command]
pub fn store_add_to_stack(name: String, id: String, store: tauri::State<SharedStore>) {
    let store = store.lock().unwrap();
    let data = serde_json::json!({
        "name": name,
        "id": id
    })
    .to_string();
    println!("ADD TO STACK: {}", &data);
    let env = xs_lib::store_open(&store.db_path).unwrap();
    xs_lib::store_put(&env, Some("stack".into()), None, data).unwrap();
}

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

// Saves item to the cas
// If source_id is present creates a link to the source
// If stack_name is present, adds item to the stack
// if stack_name and source are present, removes source from stack
#[tauri::command]
pub fn store_capture(
    item: String,
    source_id: Option<String>,
    stack_name: Option<String>,
    store: tauri::State<SharedStore>,
) {
    println!("CAPTURE: {} {:?} {:?}", item, source_id, stack_name);
    let store = store.lock().unwrap();

    let env = xs_lib::store_open(&store.db_path).unwrap();

    let id = xs_lib::store_put(&env, Some("item".into()), None, item).unwrap();

    if let Some(source_id) = &source_id {
        let data = serde_json::json!({
            "source_id": source_id,
            "id": id
        })
        .to_string();
        xs_lib::store_put(&env, Some("link".into()), None, data).unwrap();
    }

    if let Some(stack_name) = stack_name {
        let data = serde_json::json!({
            "name": stack_name,
            "id": id
        })
        .to_string();
        xs_lib::store_put(&env, Some("stack".into()), None, data).unwrap();

        if let Some(source_id) = &source_id {
            let data = serde_json::json!({
                "name": stack_name,
                "id": source_id
            })
            .to_string();
            xs_lib::store_put(&env, Some("stack".into()), Some("delete".into()), data).unwrap();
        }
    }
}
*/