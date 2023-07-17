use std::sync::{Arc, Mutex};

use crate::stack::Stack;
use crate::store::Store;

pub struct State {
    pub stack: Stack,
    pub store: Store,
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
        Self { stack, store }
    }
}

pub type SharedState = Arc<Mutex<State>>;
