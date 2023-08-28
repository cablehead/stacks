use std::sync::{Arc, Mutex};

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use chrono::prelude::*;
use scru128::Scru128Id;

pub use crate::store::{MimeType, Store, Packet};
pub use crate::view::{Item, View};

/*
export interface Item {
  id: Scru128Id;
  stack_id: Scru128Id | null;
  last_touched: string;
  touched: string[];
  hash: SSRI;
  mime_type: string;
  content_type: string;
  terse: string;
  tiktokens: number;
}

export interface Layer {
  items: Item[];
  selected: Item;
}

export interface Neo {
  root: Layer;
  sub?: Layer;
  focusedId: Scru128Id;
}
*/


pub struct State {
    pub view: View,
    pub store: Store,
    // skip_change_num is used to prevent double processing of clipboard items.
    // When our app pushes an item to the clipboard, it also records detailed information
    // about the item in the store. To avoid the clipboard poller from duplicating this
    // information, we use skip_change_num to ignore the change id associated with the item.
    pub skip_change_num: Option<i64>,
}

impl State {
    pub fn new(db_path: &str) -> Self {
        let mut state = Self {
            view: View::new(),
            store: Store::new(db_path),
            skip_change_num: None,
        };
        state.store.scan().for_each(|p| state.merge(p));
        state
    }

    pub fn to_serde_value(&self, filter: &str) -> serde_json::Value {
        let root = self
            .view
            .root()
            .iter()
            .map(|item| item.id)
            .collect::<Vec<_>>();
        let serialized_items: std::collections::HashMap<_, _> = self
            .view
            .items
            .iter()
            .map(|(id, item)| (id, self.view_item_serializer(item)))
            .collect();
        let content_meta = self.store.scan_content_meta();
        let matches = if filter.is_empty() {
            None
        } else {
            Some(self.store.index.query(filter))
        };

        serde_json::json!({
            "root": root,
            "items": serialized_items,
            "content_meta": content_meta,
            "matches": matches
        })
    }

    pub fn get_curr_stack(&mut self) -> Scru128Id {
        let curr_stack = self.view.root().first().map(|item| item.id);

        if let Some(id) = curr_stack {
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
                    return id;
                }
            }
        }

        let local: DateTime<Local> = Local::now();
        let stack_name = format!("{}", local.format("%a, %b %d %Y, %I:%M %p"));

        let packet = self.store.add(
            &stack_name.as_bytes(),
            MimeType::TextPlain,
            None,
            Some("stream.cross.stacks".to_string()),
        );

        let id = packet.id();
        self.merge(packet);
        id
    }

    pub fn merge(&mut self, packet: Packet) {
        self.view.merge(packet);
    }

    pub fn view_item_serializer<'a>(&'a self, item: &'a Item) -> ViewItemSerializer<'a> {
        ViewItemSerializer {
            item: item,
            state: self,
        }
    }
}

pub struct ViewItemSerializer<'a> {
    item: &'a Item,
    state: &'a State,
}

impl<'a> Serialize for ViewItemSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Item", 6)?;
        s.serialize_field("id", &self.item.id)?;
        s.serialize_field("last_touched", &self.item.last_touched)?;
        s.serialize_field("touched", &self.item.touched)?;
        s.serialize_field("hash", &self.item.hash)?;
        s.serialize_field("stack_id", &self.item.stack_id)?;
        s.serialize_field("children", &self.state.view.children(self.item))?;
        s.end()
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
            vec![("Stack 1", vec!["Item 1 - forked", "Item 1"])],
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
            vec![("Stack 2", vec!["Item 1"]), ("Stack 1", vec![])],
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

        let stack_id_2 = store.add(b"Stack 2", MimeType::TextPlain, None, None).id();
        let _item_id_3 = store
            .add(b"Item 3", MimeType::TextPlain, Some(stack_id_2), None)
            .id();

        // User deletes the first item
        store.delete(item_id_1);
        // User deletes the second stack
        store.delete(stack_id_2);

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
                ("Stack 2", vec!["Item 2", "Item 1"]),
                ("Stack 1", vec!["Item 2", "Item 1"]),
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
                ("Stack 2", vec!["Item 2", "Item 1"]),
                ("Stack 1", vec!["Item 2", "Item 1"]),
            ],
        );
    }

    #[test]
    fn test_state_get_curr_stack() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();

        let mut state = State::new(path);

        let curr_stack = state.get_curr_stack();
        println!("OH Hai: {:?}", curr_stack);

        let curr_stack = state.get_curr_stack();
        println!("OH Hai: {:?}", curr_stack);
    }

    #[test]
    fn test_state_view_item_serializer() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();

        let mut state = State::new(path);

        let stack_id = state
            .store
            .add(b"Stack 1", MimeType::TextPlain, None, None)
            .id();
        let item_id_1 = state
            .store
            .add(b"Item 1", MimeType::TextPlain, Some(stack_id), None)
            .id();
        let _item_id_2 = state
            .store
            .add(b"Item 2", MimeType::TextPlain, Some(stack_id), None)
            .id();

        state.store.scan().for_each(|p| state.merge(p));
        assert_view_as_expected(
            &state.store,
            &state.view,
            vec![("Stack 1", vec!["Item 2", "Item 1"])],
        );

        let root = state.view.root();
        let root: Vec<_> = root
            .iter()
            .map(|item| state.view_item_serializer(&item))
            .collect();
        let got = serde_json::to_string(&root).unwrap();
        println!("{}", got);

        let got = serde_json::to_string(&state.to_serde_value("")).unwrap();
        println!("{}", got);
    }

    #[test]
    fn test_no_duplicate_entry_on_same_hash() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();

        let mut state = State::new(path);

        let stack_id = state
            .store
            .add(b"Stack 1", MimeType::TextPlain, None, None)
            .id();
        let id1 = state
            .store
            .add(b"Item 1", MimeType::TextPlain, Some(stack_id), None)
            .id();

        // Add second item with same hash
        let id2 = state
            .store
            .add(b"Item 1", MimeType::TextPlain, Some(stack_id), None)
            .id();

        state.store.scan().for_each(|p| state.merge(p));

        // Check that the stack item only has one child and that the item has been updated correctly
        assert_view_as_expected(&state.store, &state.view, vec![("Stack 1", vec!["Item 1"])]);

        // Check that the item has been updated correctly
        let item = state.view.items.get(&id1).unwrap();
        assert_eq!(item.touched, vec![id1, id2]);
        assert_eq!(item.last_touched, id2);
    }
}
