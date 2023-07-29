use std::sync::{Arc, Mutex};

use chrono::prelude::*;
use scru128::Scru128Id;

pub use crate::store::{MimeType, Store};
pub use crate::view::View;

pub struct State {
    pub view: View,
    pub store: Store,
    // skip_change_num is used to prevent double processing of clipboard items.
    // When our app pushes an item to the clipboard, it also records detailed information
    // about the item in the store. To avoid the clipboard poller from duplicating this
    // information, we use skip_change_num to ignore the change id associated with the item.
    pub skip_change_num: Option<i64>,
    pub curr_stack: Option<Scru128Id>,
}

impl State {
    pub fn new(db_path: &str) -> Self {
        let mut state = Self {
            view: View::new(),
            store: Store::new(db_path),
            skip_change_num: None,
            curr_stack: None,
        };
        state.store.scan().for_each(|p| state.view.merge(p));
        state
    }

    pub fn get_curr_stack(&mut self) -> Option<Scru128Id> {
        if let Some(id) = self.curr_stack {
            if let Some(item) = self.view.items.get(&id) {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                let last_touched = item.last_touched.timestamp();
                println!(
                    "HERE: {:?} {:?} {:?}",
                    now,
                    last_touched,
                    now - last_touched
                );
                if now - last_touched < 3_600_000 {
                    return Some(id);
                }
            }
        }

        let local: DateTime<Local> = Local::now();
        let stack_name = format!("# {}", local.format("%a, %b %d %Y, %I:%M %p"));

        let packet = self.store.add(
            &stack_name.as_bytes(),
            MimeType::TextPlain,
            None,
            Some("stream.cross.stacks".to_string()),
        );

        self.curr_stack = Some(packet.id());
        self.view.merge(packet);
        self.curr_stack
    }
}

pub type SharedState = Arc<Mutex<State>>;

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_view_as_expected(store: &Store, view: &View, expected: Vec<(&str, Vec<&str>)>) {
        let actual: Vec<(String, Vec<String>)> = view
            .root()
            .iter()
            .filter_map(|item| {
                let children = view
                    .children(&item)
                    .iter()
                    .filter_map(|child_id| {
                        view.items
                            .get(child_id)
                            .and_then(|child_item| store.cas_read(&child_item.hash))
                            .map(|content| String::from_utf8_lossy(&content).into_owned())
                    })
                    .collect::<Vec<_>>();
                store
                    .cas_read(&item.hash)
                    .map(|s| (String::from_utf8_lossy(&s).into_owned(), children))
            })
            .collect();

        let expected: Vec<(String, Vec<String>)> = expected
            .into_iter()
            .map(|(s, children)| {
                (
                    s.to_string(),
                    children.into_iter().map(|c| c.to_string()).collect(),
                )
            })
            .collect();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_update_item() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();

        let mut store = Store::new(path);
        let mut view = View::new();

        let stack_id = store.add(b"Stack 1", MimeType::TextPlain, None, None).id();
        let item_id = store
            .add(b"Item 1", MimeType::TextPlain, Some(stack_id), None)
            .id();
        // User updates the item
        store.update(
            item_id,
            Some(b"Item 1 - updated"),
            MimeType::TextPlain,
            None,
            None,
        );

        store.scan().for_each(|p| view.merge(p));
        assert_view_as_expected(&store, &view, vec![("Stack 1", vec!["Item 1 - updated"])]);
    }

    #[test]
    fn test_fork_item() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();

        let mut store = Store::new(path);
        let mut view = View::new();

        let stack_id = store.add(b"Stack 1", MimeType::TextPlain, None, None).id();
        let item_id = store
            .add(b"Item 1", MimeType::TextPlain, Some(stack_id), None)
            .id();

        // User forks the original item
        store.fork(
            item_id,
            Some(b"Item 1 - forked"),
            MimeType::TextPlain,
            None,
            None,
        );

        store.scan().for_each(|p| view.merge(p));
        assert_view_as_expected(
            &store,
            &view,
            vec![("Stack 1", vec!["Item 1", "Item 1 - forked"])],
        );
    }

    #[test]
    fn test_move_item_to_new_stack() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();

        let mut store = Store::new(path);
        let mut view = View::new();

        let stack_id = store.add(b"Stack 1", MimeType::TextPlain, None, None).id();
        let item_id = store
            .add(b"Item 1", MimeType::TextPlain, Some(stack_id), None)
            .id();

        // User creates a new Stack "Stack 2"
        let stack_id_2 = store.add(b"Stack 2", MimeType::TextPlain, None, None).id();

        // User moves the original item to "Stack 2"
        store.update(item_id, None, MimeType::TextPlain, Some(stack_id_2), None);

        store.scan().for_each(|p| view.merge(p));
        assert_view_as_expected(
            &store,
            &view,
            vec![("Stack 1", vec![]), ("Stack 2", vec!["Item 1"])],
        );
    }

    #[test]
    fn test_delete_item() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();

        let mut store = Store::new(path);
        let mut view = View::new();

        let stack_id = store.add(b"Stack 1", MimeType::TextPlain, None, None).id();
        let item_id_1 = store
            .add(b"Item 1", MimeType::TextPlain, Some(stack_id), None)
            .id();
        let _item_id_2 = store
            .add(b"Item 2", MimeType::TextPlain, Some(stack_id), None)
            .id();

        // User deletes the first item
        store.delete(item_id_1);

        store.scan().for_each(|p| view.merge(p));
        assert_view_as_expected(&store, &view, vec![("Stack 1", vec!["Item 2"])]);
    }

    #[test]
    fn test_fork_stack() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();

        let mut store = Store::new(path);

        let stack_id = store.add(b"Stack 1", MimeType::TextPlain, None, None).id();
        let item_id_1 = store
            .add(b"Item 1", MimeType::TextPlain, Some(stack_id), None)
            .id();
        let item_id_2 = store
            .add(b"Item 2", MimeType::TextPlain, Some(stack_id), None)
            .id();

        // User forks the stack
        let new_stack_id = store
            .fork(stack_id, Some(b"Stack 2"), MimeType::TextPlain, None, None)
            .id();

        let mut view = View::new();
        store.scan().for_each(|p| view.merge(p));
        assert_view_as_expected(
            &store,
            &view,
            vec![
                ("Stack 1", vec!["Item 1", "Item 2"]),
                ("Stack 2", vec!["Item 1", "Item 2"]),
            ],
        );

        // User forks the items to the new stack
        store.fork(
            item_id_1,
            None,
            MimeType::TextPlain,
            Some(new_stack_id),
            None,
        );
        store.fork(
            item_id_2,
            None,
            MimeType::TextPlain,
            Some(new_stack_id),
            None,
        );

        let mut view = View::new();
        store.scan().for_each(|p| view.merge(p));
        assert_view_as_expected(
            &store,
            &view,
            vec![
                ("Stack 1", vec!["Item 1", "Item 2"]),
                ("Stack 2", vec!["Item 1", "Item 2"]),
            ],
        );
    }

    #[test]
    fn test_state() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();

        let mut state = State::new(path);

        let curr_stack = state.get_curr_stack();
        println!("OH Hai: {:?}", curr_stack);

        let curr_stack = state.get_curr_stack();
        println!("OH Hai: {:?}", curr_stack);
    }
}
