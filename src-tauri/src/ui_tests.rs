use crate::state::State;
use crate::store::{MimeType, StackLockStatus};

use crate::ui::Nav;

type NavExpected<'a> = (
    Option<(&'a str, Vec<&'a str>, bool)>, // root
    Option<(&'a str, Vec<&'a str>, bool)>, // sub
);

macro_rules! assert_nav_as_expected {
    ($nav:expr, $expected:expr $(,)?) => {
        assert_nav_as_expected($nav, $expected, std::panic::Location::caller())
    };
}

fn assert_nav_as_expected<'a>(
    nav: &Nav,
    expected: NavExpected<'a>,
    location: &'static std::panic::Location,
) {
    let root_expected = &expected.0;
    let sub_expected = &expected.1;

    let root_actual = nav.root.as_ref().map(|root| {
        (
            root.selected.name.clone(),
            root.items
                .iter()
                .map(|item| item.name.clone())
                .collect::<Vec<_>>(),
            root.is_focus,
        )
    });

    let sub_actual = nav.sub.as_ref().map(|sub| {
        (
            sub.selected.name.clone(),
            sub.items
                .iter()
                .map(|item| item.name.clone())
                .collect::<Vec<_>>(),
            sub.is_focus,
        )
    });

    assert_eq!(
        root_actual,
        root_expected.as_ref().map(|(s, v, b)| (
            s.to_string(),
            v.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            *b
        )),
        "Failure at {}:{}",
        location.file(),
        location.line()
    );

    assert_eq!(
        sub_actual,
        sub_expected.as_ref().map(|(s, v, b)| (
            s.to_string(),
            v.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            *b
        )),
        "Failure at {}:{}",
        location.file(),
        location.line()
    );
}

#[test]
fn test_ui_render() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    let (sender, _receiver) = std::sync::mpsc::channel();
    let mut state = State::new(path, sender);

    let stack_ids: Vec<_> = (1..=3)
        .map(|i| {
            state
                .store
                .add_stack(format!("Stack {}", i).as_bytes(), StackLockStatus::Unlocked)
                .id
        })
        .collect();

    for (i, stack_id) in stack_ids.iter().enumerate() {
        let _ = state.store.add(
            format!("https://stack-{}.com", i + 1).as_bytes(),
            MimeType::TextPlain,
            *stack_id,
        );
        for j in 1..=3 {
            let _ = state.store.add(
                format!("S{}::Item {}", i + 1, j).as_bytes(),
                MimeType::TextPlain,
                *stack_id,
            );
        }
    }

    // empty state
    assert_nav_as_expected!(&state.ui.render(&state.store), (None, None));

    // post initial merge state
    state.store.scan().for_each(|p| state.merge(&p));
    assert_nav_as_expected!(
        &state.ui.render(&state.store),
        (
            Some(("Stack 3", vec!["Stack 3", "Stack 2", "Stack 1"], false)),
            Some((
                "S3::Item 3",
                vec![
                    "S3::Item 3",
                    "S3::Item 2",
                    "S3::Item 1",
                    "https://stack-3.com"
                ],
                true,
            )),
        ),
    );

    // user press: opt + down
    state.ui.select_down_stack();
    assert_nav_as_expected!(
        &state.ui.render(&state.store),
        (
            Some(("Stack 2", vec!["Stack 3", "Stack 2", "Stack 1"], false)),
            Some((
                "S2::Item 3",
                vec![
                    "S2::Item 3",
                    "S2::Item 2",
                    "S2::Item 1",
                    "https://stack-2.com"
                ],
                true,
            )),
        ),
    );

    // user press: opt + up
    state.ui.select_up_stack();
    assert_nav_as_expected!(
        &state.ui.render(&state.store),
        (
            Some(("Stack 3", vec!["Stack 3", "Stack 2", "Stack 1"], false)),
            Some((
                "S3::Item 3",
                vec![
                    "S3::Item 3",
                    "S3::Item 2",
                    "S3::Item 1",
                    "https://stack-3.com"
                ],
                true,
            )),
        ),
    );

    // user press: down
    state.ui.select_down();
    assert_nav_as_expected!(
        &state.ui.render(&state.store),
        (
            Some(("Stack 3", vec!["Stack 3", "Stack 2", "Stack 1"], false)),
            Some((
                "S3::Item 2",
                vec![
                    "S3::Item 3",
                    "S3::Item 2",
                    "S3::Item 1",
                    "https://stack-3.com"
                ],
                true,
            )),
        ),
    );

    // user press: up
    state.ui.select_up();
    assert_nav_as_expected!(
        &state.ui.render(&state.store),
        (
            Some(("Stack 3", vec!["Stack 3", "Stack 2", "Stack 1"], false)),
            Some((
                "S3::Item 3",
                vec![
                    "S3::Item 3",
                    "S3::Item 2",
                    "S3::Item 1",
                    "https://stack-3.com"
                ],
                true,
            )),
        ),
    );

    // user press: delete # this is the top item in the first stack
    let packet = state
        .store
        .delete(state.ui.focused.as_ref().unwrap().item.id);
    state.merge(&packet);
    assert_nav_as_expected!(
        &state.ui.render(&state.store),
        (
            Some(("Stack 3", vec!["Stack 3", "Stack 2", "Stack 1"], false)),
            Some((
                "S3::Item 2",
                vec!["S3::Item 2", "S3::Item 1", "https://stack-3.com"],
                true
            )),
        ),
    );

    // user press: left + right # we're checking navigation works ok post delete
    state.ui.select_left();
    state.ui.select_right();
    assert_nav_as_expected!(
        &state.ui.render(&state.store),
        (
            Some(("Stack 3", vec!["Stack 3", "Stack 2", "Stack 1"], false)),
            Some((
                "S3::Item 2",
                vec!["S3::Item 2", "S3::Item 1", "https://stack-3.com"],
                true
            )),
        ),
    );

    // user press: down
    state.ui.select_down();
    assert_nav_as_expected!(
        &state.ui.render(&state.store),
        (
            Some(("Stack 3", vec!["Stack 3", "Stack 2", "Stack 1"], false)),
            Some((
                "S3::Item 1",
                vec!["S3::Item 2", "S3::Item 1", "https://stack-3.com"],
                true
            )),
        ),
    );

    // user press: left
    state.ui.select_left();
    assert_nav_as_expected!(
        &state.ui.render(&state.store),
        (
            Some(("Stack 3", vec!["Stack 3", "Stack 2", "Stack 1"], true)),
            Some((
                "S3::Item 1",
                vec!["S3::Item 2", "S3::Item 1", "https://stack-3.com"],
                false
            )),
        ),
    );

    // user press: down
    state.ui.select_down();
    assert_nav_as_expected!(
        &state.ui.render(&state.store),
        (
            Some(("Stack 2", vec!["Stack 3", "Stack 2", "Stack 1"], true)),
            Some((
                "S2::Item 3",
                vec![
                    "S2::Item 3",
                    "S2::Item 2",
                    "S2::Item 1",
                    "https://stack-2.com"
                ],
                false,
            )),
        ),
    );

    // user press: right, twice: there was a bug where this would bump the ui back to the top most
    // item. assert when right most, hitting right doesn't do anything
    state.ui.select_right();
    state.ui.select_right();
    assert_nav_as_expected!(
        &state.ui.render(&state.store),
        (
            Some(("Stack 2", vec!["Stack 3", "Stack 2", "Stack 1"], false)),
            Some((
                "S2::Item 3",
                vec![
                    "S2::Item 3",
                    "S2::Item 2",
                    "S2::Item 1",
                    "https://stack-2.com"
                ],
                true,
            )),
        ),
    );

    // user press: left
    // back to stack level navigation
    state.ui.select_left();

    // user set: filter
    state.nav_set_filter("item 1", "");
    assert_nav_as_expected!(
        &state.ui.render(&state.store),
        (
            Some(("Stack 2", vec!["Stack 3", "Stack 2", "Stack 1"], true)),
            Some(("S2::Item 1", vec!["S2::Item 1"], false)),
        ),
    );

    // user set: filter # clear
    state.nav_set_filter("", "All");
    assert_nav_as_expected!(
        &state.ui.render(&state.store),
        (
            Some(("Stack 2", vec!["Stack 3", "Stack 2", "Stack 1"], true)),
            Some((
                "S2::Item 3",
                vec![
                    "S2::Item 3",
                    "S2::Item 2",
                    "S2::Item 1",
                    "https://stack-2.com"
                ],
                false,
            )),
        ),
    );

    // user set: content_filter # Links
    state.nav_set_filter("", "Link");
    assert_nav_as_expected!(
        &state.ui.render(&state.store),
        (
            Some(("Stack 2", vec!["Stack 3", "Stack 2", "Stack 1"], true)),
            Some(("https://stack-2.com", vec!["https://stack-2.com"], false,)),
        ),
    );

    // user set: filter
    state.nav_set_filter("item 3", "");
    assert_nav_as_expected!(
        &state.ui.render(&state.store),
        (
            Some(("Stack 2", vec!["Stack 2", "Stack 1"], true)),
            Some(("S2::Item 3", vec!["S2::Item 3"], false)),
        ),
    );

    // user set: filter # no matches
    state.nav_set_filter("FOOBAR", "");
    assert_nav_as_expected!(&state.ui.render(&state.store), (None, None));
}

#[test]
fn test_ui_generate_preview() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    let (sender, _receiver) = std::sync::mpsc::channel();
    let mut state = State::new(path, sender);

    let stack = state
        .store
        .add_stack(b"Code Snippets Stack", StackLockStatus::Unlocked);

    // Add the Rust snippet to the store
    let rust_code = r#"
    fn main() {
        println!("Hello, world!");
    }
    "#;
    let rust_packet = state
        .store
        .add(rust_code.as_bytes(), MimeType::TextPlain, stack.id);
    state
        .store
        .update_content_type(rust_packet.hash.clone().unwrap(), "Rust".to_string());

    // Add the Nushell snippet to the same stack
    let nushell_code = r#"
    let x = 10
    echo $x
    "#;
    let nushell_packet = state
        .store
        .add(nushell_code.as_bytes(), MimeType::TextPlain, stack.id);
    state
        .store
        .update_content_type(nushell_packet.hash.clone().unwrap(), "Nushell".to_string());

    // Scan and merge to update the state
    state.store.scan().for_each(|p| state.merge(&p));

    // Test Rust preview generation
    let rust_content = state
        .store
        .get_content(&rust_packet.hash.clone().unwrap())
        .unwrap();
    let rust_meta = state
        .store
        .get_content_meta(&rust_packet.hash.clone().unwrap())
        .unwrap();
    let _rust_preview = state.ui.generate_preview(
        &Some(rust_content),
        &rust_meta.mime_type,
        &rust_meta.content_type,
        false,
    );

    // Test Nushell preview generation
    let nushell_content = state
        .store
        .get_content(&nushell_packet.hash.clone().unwrap())
        .unwrap();
    let nushell_meta = state
        .store
        .get_content_meta(&nushell_packet.hash.clone().unwrap())
        .unwrap();
    let _nushell_preview = state.ui.generate_preview(
        &Some(nushell_content),
        &nushell_meta.mime_type,
        &nushell_meta.content_type,
        false,
    );
}
