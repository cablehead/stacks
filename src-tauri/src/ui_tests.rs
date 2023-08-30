use crate::state::State;
use crate::store::MimeType;

use crate::ui::Nav;

type NavExpected<'a> = (
    (&'a str, Vec<&'a str>, bool),         // root
    Option<(&'a str, Vec<&'a str>, bool)>, // sub
);

fn assert_nav_as_expected<'a>(nav: &Nav, expected: NavExpected<'a>) {
    let root_expected = &expected.0;
    let sub_expected = &expected.1;

    let root_actual = (
        nav.root.selected.terse.clone(),
        nav.root
            .items
            .iter()
            .map(|item| item.terse.clone())
            .collect::<Vec<_>>(),
        nav.root.is_focus,
    );
    let sub_actual = nav.sub.as_ref().map(|sub| {
        (
            sub.selected.terse.clone(),
            sub.items
                .iter()
                .map(|item| item.terse.clone())
                .collect::<Vec<_>>(),
            sub.is_focus,
        )
    });

    assert_eq!(
        root_actual,
        (
            root_expected.0.to_string(),
            root_expected
                .1
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
            root_expected.2
        )
    );
    assert_eq!(
        sub_actual,
        sub_expected.as_ref().map(|(s, v, b)| (
            s.to_string(),
            v.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            *b
        ))
    );
}

#[test]
fn test_ui_render() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    let mut state = State::new(path);

    let stack_ids: Vec<_> = (1..=3)
        .map(|i| {
            state
                .store
                .add(
                    format!("Stack {}", i).as_bytes(),
                    MimeType::TextPlain,
                    None,
                    None,
                )
                .id()
        })
        .collect();

    let mut items = Vec::new();
    for (i, stack_id) in stack_ids.iter().enumerate() {
        for j in 1..=3 {
            let item = state.store.add(
                format!("S{}::Item {}", i + 1, j).as_bytes(),
                MimeType::TextPlain,
                Some(*stack_id),
                None,
            );
            items.push(item.id());
        }
    }
    state.store.scan().for_each(|p| state.merge(p));

    let nav = state.ui.render(&state.store, &state.view);
    assert_nav_as_expected(
        &nav,
        (
            ("Stack 3", vec!["Stack 3", "Stack 2", "Stack 1"], false),
            Some((
                "S3::Item 3",
                vec!["S3::Item 3", "S3::Item 2", "S3::Item 1"],
                true,
            )),
        ),
    );

    state.nav_select_down();
    let nav = state.ui.render(&state.store, &state.view);
    assert_nav_as_expected(
        &nav,
        (
            ("Stack 3", vec!["Stack 3", "Stack 2", "Stack 1"], false),
            Some((
                "S3::Item 2",
                vec!["S3::Item 3", "S3::Item 2", "S3::Item 1"],
                true,
            )),
        ),
    );

    state.nav_select_up();
    let nav = state.ui.render(&state.store, &state.view);
    assert_nav_as_expected(
        &nav,
        (
            ("Stack 3", vec!["Stack 3", "Stack 2", "Stack 1"], false),
            Some((
                "S3::Item 3",
                vec!["S3::Item 3", "S3::Item 2", "S3::Item 1"],
                true,
            )),
        ),
    );

    let packet = state.store.delete(state.ui.focused.as_ref().unwrap().id);
    state.merge(packet);
    let nav = state.ui.render(&state.store, &state.view);
    assert_nav_as_expected(
        &nav,
        (
            ("Stack 3", vec!["Stack 3", "Stack 2", "Stack 1"], false),
            Some(("S3::Item 2", vec!["S3::Item 2", "S3::Item 1"], true)),
        ),
    );

    state.nav_select_down();
    let nav = state.ui.render(&state.store, &state.view);
    assert_nav_as_expected(
        &nav,
        (
            ("Stack 3", vec!["Stack 3", "Stack 2", "Stack 1"], false),
            Some(("S3::Item 1", vec!["S3::Item 2", "S3::Item 1"], true)),
        ),
    );

    state.nav_select_left();
    let nav = state.ui.render(&state.store, &state.view);
    assert_nav_as_expected(
        &nav,
        (
            ("Stack 3", vec!["Stack 3", "Stack 2", "Stack 1"], true),
            Some(("S3::Item 1", vec!["S3::Item 2", "S3::Item 1"], false)),
        ),
    );

    state.nav_select_down();
    let nav = state.ui.render(&state.store, &state.view);
    assert_nav_as_expected(
        &nav,
        (
            ("Stack 2", vec!["Stack 3", "Stack 2", "Stack 1"], true),
            Some((
                "S2::Item 3",
                vec!["S2::Item 3", "S2::Item 2", "S2::Item 1"],
                false,
            )),
        ),
    );

    state.nav_set_filter("item 1", "");

    println!("");
    println!("{:?}", state.ui.matches.as_ref().unwrap().len());
    println!("");

    let nav = state.ui.render(&state.store, &state.view);
    assert_nav_as_expected(
        &nav,
        (
            ("Stack 2", vec!["Stack 3", "Stack 2", "Stack 1"], true),
            Some(("S2::Item 1", vec!["S2::Item 1"], false)),
        ),
    );
}
