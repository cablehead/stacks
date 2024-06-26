use std::sync::mpsc::Sender;
use std::sync::Arc;

use chrono::prelude::*;
use scru128::Scru128Id;

use tracing_mutex_span::TracingMutexSpan;

pub use crate::store::{Packet, StackLockStatus, Store};
pub use crate::ui::UI;
pub use crate::view::View;

pub struct State {
    pub view: View,
    pub store: Store,
    pub ui: UI,
    // skip_change_num is used to prevent double processing of clipboard items.
    // When our app pushes an item to the clipboard, it also records detailed information
    // about the item in the store. To avoid the clipboard poller from duplicating this
    // information, we use skip_change_num to ignore the change id associated with the item.
    pub skip_change_num: Option<i64>,
    pub packet_sender: Sender<View>,
}

impl State {
    pub fn new(db_path: &str, packet_sender: Sender<View>) -> Self {
        let store = Store::new(db_path);
        let mut view = View::new();
        store.scan().for_each(|p| view.merge(&p));

        let ui = UI::new(&view);
        let state = Self {
            view,
            store,
            ui,
            skip_change_num: None,
            packet_sender,
        };
        let _ = state.packet_sender.send(state.view.clone());
        state
    }

    pub fn nav_set_filter(&mut self, filter: &str, content_type: &str) {
        self.ui
            .set_filter(&self.store, &self.view, filter, content_type);
    }

    pub fn nav_select(&mut self, focused_id: &Scru128Id) {
        let focused = self.view.get_focus_for_id(focused_id);
        self.ui.select(focused);
    }

    pub fn get_curr_stack(&mut self) -> Scru128Id {
        let curr_stack = self
            .view
            .root()
            .iter()
            .find(|&&item| !item.locked)
            .map(|&item| item.id);

        if let Some(id) = curr_stack {
            if let Some(item) = self.view.items.get(&id) {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                let last_touched = item.last_touched.timestamp();
                if now - last_touched < 3_600_000 {
                    return id;
                }
            }
        }

        let local: DateTime<Local> = Local::now();
        let stack_name = format!("{}", local.format("%a, %b %d %Y, %I:%M %p"));

        let packet = self
            .store
            .add_stack(stack_name.as_bytes(), StackLockStatus::Unlocked);

        self.merge(&packet);
        packet.id
    }

    pub fn merge(&mut self, packet: &Packet) {
        self.view.merge(packet);
        self.ui.refresh_view(&self.view);
        let _ = self.packet_sender.send(self.view.clone());
    }
}

pub type SharedState = Arc<TracingMutexSpan<State>>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_state_get_curr_stack() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let (sender, _receiver) = std::sync::mpsc::channel();
        let mut state = State::new(path, sender);
        let _ = state.get_curr_stack();
        let _ = state.get_curr_stack();
    }
}
