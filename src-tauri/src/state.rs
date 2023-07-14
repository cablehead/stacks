use std::sync::{Arc, Mutex};

use crate::stack::Stack;
use crate::store::Store;

pub struct State {
    pub stack: Stack,
    pub store: Store,
}

impl State {
    pub fn new(db_path: &str) -> Self {
        Self {
            stack: Stack::new(),
            store: Store::new(db_path),
        }
    }
}

pub type SharedState = Arc<Mutex<State>>;
