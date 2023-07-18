use std::sync::{Arc, Mutex};

use crate::stack::Stack;
use crate::store::Store;

pub struct State {
    pub stack: Stack,
    pub store: Store,
    // skip_change_num is used to prevent double processing of clipboard items.
    // When our app pushes an item to the clipboard, it also records detailed information
    // about the item in the store. To avoid the clipboard poller from duplicating this
    // information, we use skip_change_num to ignore the change id associated with the item.
    pub skip_change_num: Option<i64>,
}

impl State {
    pub fn new(db_path: &str) -> Self {
        let store = Store::new(db_path);
        let mut stack = Stack::new();
        for frame in store.list() {
            let content = store.cat(&frame.hash);
            if let Some(content) = content {
                stack.merge(&frame, &content);
            } else {
                log::warn!("frame with no content: {:?}", frame);
            }
        }
        Self {
            stack,
            store,
            skip_change_num: None,
        }
    }
}

pub type SharedState = Arc<Mutex<State>>;
