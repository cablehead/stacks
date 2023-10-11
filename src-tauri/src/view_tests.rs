pub use crate::state::State;
pub use crate::store::{MimeType, Packet, StackLockStatus, Store};
pub use crate::view::{Item, View};

macro_rules! assert_view_as_expected {
    ($store:expr, $view:expr, $expected:expr $(,)?) => {
        assert_view_as_expected($store, $view, $expected, std::panic::Location::caller())
    };
}

fn assert_view_as_expected(
    store: &Store,
    view: &View,
    expected: Vec<(&str, Vec<&str>)>,
    location: &'static std::panic::Location,
) {
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
                        .and_then(|child_item| store.get_content(&child_item.hash))
                        .map(|content| String::from_utf8_lossy(&content).into_owned())
                })
                .collect::<Vec<_>>();
            store
                .get_content(&item.hash)
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

    assert_eq!(
        actual,
        expected,
        "Failure at {}:{}",
        location.file(),
        location.line()
    );
}

#[test]
fn test_update_item() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    let mut store = Store::new(path);
    let mut view = View::new();

    let stack_id = store.add_stack(b"Stack 1", StackLockStatus::Unlocked).id;
    let item_id = store.add(b"Item 1", MimeType::TextPlain, stack_id).id;
    // User updates the item
    store.update(
        item_id,
        Some(b"Item 1 - updated"),
        MimeType::TextPlain,
        None,
    );

    store.scan().for_each(|p| view.merge(&p));
    assert_view_as_expected!(&store, &view, vec![("Stack 1", vec!["Item 1 - updated"])]);
}

#[test]
fn test_fork_item() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    let mut store = Store::new(path);
    let mut view = View::new();

    let stack_id = store.add_stack(b"Stack 1", StackLockStatus::Unlocked).id;
    let item_id = store.add(b"Item 1", MimeType::TextPlain, stack_id).id;

    // User forks the original item
    store.fork(item_id, Some(b"Item 1 - forked"), MimeType::TextPlain, None);

    store.scan().for_each(|p| view.merge(&p));
    assert_view_as_expected!(
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

    let stack_id = store.add_stack(b"Stack 1", StackLockStatus::Unlocked).id;
    let item_id = store.add(b"Item 1", MimeType::TextPlain, stack_id).id;

    // User creates a new Stack "Stack 2"
    let stack_id_2 = store.add_stack(b"Stack 2", StackLockStatus::Unlocked).id;

    // User moves the original item to "Stack 2"
    store.update(item_id, None, MimeType::TextPlain, Some(stack_id_2));

    store.scan().for_each(|p| view.merge(&p));
    assert_view_as_expected!(
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

    let stack_id = store.add_stack(b"Stack 1", StackLockStatus::Unlocked).id;
    let item_id_1 = store.add(b"Item 1", MimeType::TextPlain, stack_id).id;
    let _item_id_2 = store.add(b"Item 2", MimeType::TextPlain, stack_id).id;

    let stack_id_2 = store.add_stack(b"Stack 2", StackLockStatus::Unlocked).id;
    let _item_id_3 = store.add(b"Item 3", MimeType::TextPlain, stack_id_2).id;

    // User deletes the first item
    store.delete(item_id_1);
    // User deletes the second stack
    store.delete(stack_id_2);

    store.scan().for_each(|p| view.merge(&p));
    assert_view_as_expected!(&store, &view, vec![("Stack 1", vec!["Item 2"])]);
}

#[test]
fn test_no_duplicate_entry_on_same_hash() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    let mut state = State::new(path);

    let stack_id = state
        .store
        .add_stack(b"Stack 1", StackLockStatus::Unlocked)
        .id;
    let id1 = state.store.add(b"Item 1", MimeType::TextPlain, stack_id).id;

    // Add second item with same hash
    let id2 = state.store.add(b"Item 1", MimeType::TextPlain, stack_id).id;

    state.store.scan().for_each(|p| state.merge(&p));

    // Check that the stack item only has one child and that the item has been updated correctly
    assert_view_as_expected!(&state.store, &state.view, vec![("Stack 1", vec!["Item 1"])]);

    // Check that the item has been updated correctly
    let item = state.view.items.get(&id1).unwrap();
    assert_eq!(item.touched, vec![id1, id2]);
    assert_eq!(item.last_touched, id2);
}

#[test]
fn test_stream() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    let mut store = Store::new(path);
    let mut view = View::new();

    let stack_id = store.add_stack(b"Stack 1", StackLockStatus::Unlocked).id;
    store.scan().for_each(|p| view.merge(&p));

    // Start the stream with the content "oh, "
    let packet = store.start_stream(Some(stack_id), b"oh, ");
    view.merge(&packet);
    assert_view_as_expected!(&store, &view, vec![("Stack 1", vec!["oh, "])]);

    // Add "hai " to the stream
    let packet = store.update_stream(packet.id, b"hai ");
    view.merge(&packet);
    assert_view_as_expected!(&store, &view, vec![("Stack 1", vec!["oh, hai "])]);

    // Add "123" to the stream
    let packet = store.update_stream(packet.id, b"123");
    view.merge(&packet);
    assert_view_as_expected!(&store, &view, vec![("Stack 1", vec!["oh, hai 123"])]);

    // check that get_content_meta finds the ephemeral copy
    let meta = store.get_content_meta(&packet.hash.unwrap());
    assert_eq!(meta.unwrap().terse, "oh, hai 123");

    // Complete the stream
    let packet = store.end_stream(packet.id);
    view.merge(&packet);
    assert_view_as_expected!(&store, &view, vec![("Stack 1", vec!["oh, hai 123"])]);

    // Check that the item is no longer ephemeral
    let item = view.items.get(&packet.id).unwrap();
    assert!(!item.ephemeral);

    // Simulate a full restart
    let mut view = View::new();
    store.scan().for_each(|p| view.merge(&p));
    assert_view_as_expected!(&store, &view, vec![("Stack 1", vec!["oh, hai 123"])]);
    let item = view.items.get(&packet.id).unwrap();
    assert!(!item.ephemeral);
    let meta = store.get_content_meta(&packet.hash.unwrap());
    assert_eq!(meta.unwrap().terse, "oh, hai 123");
}

#[test]
fn test_no_duplicate_entry_on_same_hash_on_stream_end() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    let mut state = State::new(path);

    let stack_id = state
        .store
        .add_stack(b"Stack 1", StackLockStatus::Unlocked)
        .id;
    let id1 = state.store.add(b"Item 1", MimeType::TextPlain, stack_id).id;
    state.store.scan().for_each(|p| state.merge(&p));

    // Start a stream with the same content
    let packet = state.store.start_stream(Some(stack_id), b"Item");
    state.merge(&packet);
    assert_view_as_expected!(
        &state.store,
        &state.view,
        vec![("Stack 1", vec!["Item", "Item 1"])]
    );

    // Update the stream with the same content
    let packet = state.store.update_stream(packet.id, b" 1");
    state.merge(&packet);
    assert_view_as_expected!(
        &state.store,
        &state.view,
        vec![("Stack 1", vec!["Item 1", "Item 1"])]
    );

    // End the stream
    let packet = state.store.end_stream(packet.id);
    state.merge(&packet);
    // Check that the stack item only has one child
    assert_view_as_expected!(&state.store, &state.view, vec![("Stack 1", vec!["Item 1"])]);
    // Check that the item has been updated correctly
    let item = state.view.items.get(&id1).unwrap();
    assert_eq!(item.touched, vec![id1, packet.id]);
    assert_eq!(item.last_touched, packet.id);
}
