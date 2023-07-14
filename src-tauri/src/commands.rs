
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
