use std::sync::{Arc, Mutex};

use crate::stack::Stack;

pub struct State {
    pub stack: Stack,
}

impl State {
    pub fn new() -> Self {
        Self {
            stack: Stack::new(),
        }
    }
}

pub type SharedState = Arc<Mutex<State>>;
