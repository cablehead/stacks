use crate::state::State;
use crate::store::MimeType;

use crate::ui::Nav;

type NavExpected<'a> = (
    Option<(&'a str, Vec<&'a str>, bool)>, // root
    Option<(&'a str, Vec<&'a str>, bool)>, // sub
);

fn assert_nav_as_expected<'a>(nav: &Nav, expected: NavExpected<'a>) {
    let root_expected = &expected.0;
    let sub_expected = &expected.1;

    let root_actual = nav.root.as_ref().map(|root| {
        (
            root.selected.terse.clone(),
            root.items
                .iter()
                .map(|item| item.terse.clone())
                .collect::<Vec<_>>(),
            root.is_focus,
        )
    });

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
        root_expected.as_ref().map(|(s, v, b)| (
            s.to_string(),
            v.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            *b
        ))
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

    // empty state
    assert_nav_as_expected(&state.ui.render(&state.store), (None, None));

    // post initial merge state
    state.store.scan().for_each(|p| state.merge(p));
    assert_nav_as_expected(
        &state.ui.render(&state.store),
        (
            Some(("Stack 3", vec!["Stack 3", "Stack 2", "Stack 1"], false)),
            Some((
                "S3::Item 3",
                vec!["S3::Item 3", "S3::Item 2", "S3::Item 1"],
                true,
            )),
        ),
    );

    // user press: down
    state.nav_select_down();
    assert_nav_as_expected(
        &state.ui.render(&state.store),
        (
            Some(("Stack 3", vec!["Stack 3", "Stack 2", "Stack 1"], false)),
            Some((
                "S3::Item 2",
                vec!["S3::Item 3", "S3::Item 2", "S3::Item 1"],
                true,
            )),
        ),
    );

    // user press: up
    state.nav_select_up();
    assert_nav_as_expected(
        &state.ui.render(&state.store),
        (
            Some(("Stack 3", vec!["Stack 3", "Stack 2", "Stack 1"], false)),
            Some((
                "S3::Item 3",
                vec!["S3::Item 3", "S3::Item 2", "S3::Item 1"],
                true,
            )),
        ),
    );

    // user press: delete # this is the top item in the first stack
    let packet = state.store.delete(state.ui.focused.as_ref().unwrap().id);
    state.merge(packet);
    assert_nav_as_expected(
        &state.ui.render(&state.store),
        (
            Some(("Stack 3", vec!["Stack 3", "Stack 2", "Stack 1"], false)),
            Some(("S3::Item 2", vec!["S3::Item 2", "S3::Item 1"], true)),
        ),
    );

    // user press: left + right # we're checking navigation works ok post delete
    state.ui.select_left();
    state.ui.select_right();
    assert_nav_as_expected(
        &state.ui.render(&state.store),
        (
            Some(("Stack 3", vec!["Stack 3", "Stack 2", "Stack 1"], false)),
            Some(("S3::Item 2", vec!["S3::Item 2", "S3::Item 1"], true)),
        ),
    );

    // user press: down
    state.nav_select_down();
    assert_nav_as_expected(
        &state.ui.render(&state.store),
        (
            Some(("Stack 3", vec!["Stack 3", "Stack 2", "Stack 1"], false)),
            Some(("S3::Item 1", vec!["S3::Item 2", "S3::Item 1"], true)),
        ),
    );

    // user press: left
    state.nav_select_left();
    assert_nav_as_expected(
        &state.ui.render(&state.store),
        (
            Some(("Stack 3", vec!["Stack 3", "Stack 2", "Stack 1"], true)),
            Some(("S3::Item 1", vec!["S3::Item 2", "S3::Item 1"], false)),
        ),
    );

    // user press: down
    state.nav_select_down();
    assert_nav_as_expected(
        &state.ui.render(&state.store),
        (
            Some(("Stack 2", vec!["Stack 3", "Stack 2", "Stack 1"], true)),
            Some((
                "S2::Item 3",
                vec!["S2::Item 3", "S2::Item 2", "S2::Item 1"],
                false,
            )),
        ),
    );

    // user set: filter
    state.nav_set_filter("item 1", "");
    assert_nav_as_expected(
        &state.ui.render(&state.store),
        (
            Some(("Stack 2", vec!["Stack 3", "Stack 2", "Stack 1"], true)),
            Some(("S2::Item 1", vec!["S2::Item 1"], false)),
        ),
    );

    // user set: filter # clear
    state.nav_set_filter("", "All");
    assert_nav_as_expected(
        &state.ui.render(&state.store),
        (
            Some(("Stack 2", vec!["Stack 3", "Stack 2", "Stack 1"], true)),
            Some((
                "S2::Item 3",
                vec!["S2::Item 3", "S2::Item 2", "S2::Item 1"],
                false,
            )),
        ),
    );

    state.nav_set_filter("item 3", "");
    assert_nav_as_expected(
        &state.ui.render(&state.store),
        (
            Some(("Stack 2", vec!["Stack 2", "Stack 1"], true)),
            Some(("S2::Item 3", vec!["S2::Item 3"], false)),
        ),
    );

    // user set: filter # no matches
    state.nav_set_filter("FOOBAR", "");
    assert_nav_as_expected(&state.ui.render(&state.store), (None, None));
}
