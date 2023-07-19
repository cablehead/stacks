use std::sync::{Arc, Mutex};

use crate::stack::Stack;
use crate::store::{Frame, MimeType, Packet, Store};

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
        let mut state = Self {
            stack: Stack::new(),
            store,
            skip_change_num: None,
        };
        for frame in state.store.list() {
            state.merge(&frame);
        }
        state
    }

    pub fn add_content(
        &mut self,
        source: Option<String>,
        stack_hash: Option<ssri::Integrity>,
        mime_type: MimeType,
        content: &[u8],
    ) -> Frame {
        let hash = self.store.cas_write(content);

        let frame = Frame {
            id: scru128::new(),
            source,
            stack_hash,
            mime_type,
            hash,
        };
        let packet = self.store.insert_frame(&frame);
        self.merge(&packet);
        frame
    }

    pub fn merge(&mut self, packet: &Packet) {
        match packet {
            Packet::Frame(frame) => {
                let content = self.store.cat(&frame.hash);
                if let Some(content) = content {
                    self.stack.merge(&frame, &content);
                } else {
                    log::warn!("frame with no content: {:?}", frame);
                }
            }
            Packet::DeleteFrame(frame) => {
                self.stack.merge_delete(frame);
            }
        }
    }
}

pub type SharedState = Arc<Mutex<State>>;
