# Stacks

## Features

### Quick Filter

![filter screenshot](./docs/screenshots/filter.webp)

### Dark Mode

![dark mode](./docs/screenshots/dark-mode.webp)

### Edit Clippings

![edit](./docs/screenshots/edit.webp)

## Development

```
npm run tauri dev
# Type checking:
tsc
```

### x-macos-pasteboard

https://github.com/cablehead/workspace/blob/x-macos-pasteboard/Sources/Clip/main.swift

## Storage

content.add: id, hash  # need to be able give a type

content_type is:

- Text
- Link
- Stack
- Image

mime_type is:

- text/plain
- image/png

Potentially source

stack.add: id, id
stack.del: id, id
edit.add: id, id

xs_lib::store_put(&env, Some("clipboard".into()), None, line.clone())
xs_lib::store_put(&env, Some("stack".into()), None, data).unwrap();
xs_lib::store_put(&env, Some("stack".into()), Some("delete".into()), data).unwrap();
xs_lib::store_put(&env, Some("item".into()), None, item).unwrap();
xs_lib::store_put(&env, Some("link".into()), None, data).unwrap();


## Release

```
# make sure dev console is disabled

# update Cargo.toml and tauri.conf.json for new version
# set RELEASE to the new version, e.g
RELEASE=v0.5.2

./scripts/build.sh
# while that builds
vi changes/$RELEASE

# after build completes
cat changes/$RELEASE | ./scripts/release.sh

# commit and push
git commit -a -m "chore: release $RELEASE"

gh release create $RELEASE $RELEASE_PATH/* -n "$(cat changes/$RELEASE)"
```

## Review: Daily/2022-08-11.md

```
        "CREATE TABLE IF NOT EXISTS stream (
           id INTEGER PRIMARY KEY,
           topic TEXT NOT NULL,
           stamp BLOB NOT NULL,
           source_id INTEGER,
           parent_id INTEGER,
           data TEXT,
           err TEXT,
           code INTEGER NOT NULL
```

## Todo

### To release

id source_id parent_id

- add to stack
    - it feels weird the new item in the stack has no touches
    - they're tied together: able to tell what context something is from when
      rendering in a another given content
    - so maybe, the item is touched: but the parent version ignores that, in
      terms of sorting by most recently touched.
    - or:
    - this should fork the item by default
    - the added item shouldn't be marked as touched: result of being forked

- delete from stack
    - and because its fork by default: that just means from the stack, not the
      outer item

- capture
- delete

- while in stack, new items go to that stack
- editting within stack: fork instead of replace
- Copy entire stack puts the entire stack on the clipboard, but doesn't save it
  to the store

- Use the above mechanism so that write_to_clipboard only needs to record a
  touch record: if that's useful at all


### Next release

- truncate long urls
- limit height for status pane

- Incorporate: /Users/andy/.s/sessions/039VQC9VTZ3PCDW3TL9YTVE6L/png-clip

- Focus clips which are generated by Stacks
- Reset to the first item after some period of inactivity (30 secs?)

- Add ability to create a Note

- Potentially rename Clipboard to.. /?

### And then

- Clicking trigger in the Actions Modal doesn't trigger the action

- Copying an item should put it at the top of stacks

- Going to want multi-select to add

- shift-enter to copy but keep stacks open: maybe
- or shift-enter: replace

- Editor
    - access clips while editor is open

- Preference panel

- Editor capture should create an xs.add row: with parent set to the id the
  editor was triggered on: this should be merged version that's put on the
  clipboard

- better cursor handling
    - dedicated focus handling when the filter changes
    - unless the item being added is the item focused, in which case, jump to
      the first?? - maybe
    - reset to start state of 1 minute

- mark a source as don't track (for password managers, etc).

- meta panel. add: image info

- handle clipboard images
    - when the user hits enter

- customize key press
- meta-n opens choice: note / command
- add filter: number of times copied

- Actions menu: Add icons to options
