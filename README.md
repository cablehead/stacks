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

### Next release

- truncate long urls
- limit height for status pane

- Add ability to create a Note

- Potentially rename Clipboard to.. /?

### And then

- Clicking trigger in the Actions Modal doesn't trigger the action

- Going to want multi-select to add

- shift-enter to copy but keep stacks open: maybe
- or shift-enter: replace

- Editor
    - access clips while editor is open

- Preference panel

- Editor capture should create an xs.add row: with parent set to the id the
  editor was triggered on: this should be merged version that's put on the
  clipboard

- mark a source as don't track (for password managers, etc).

- meta panel. add: image info

- handle clipboard images
    - when the user hits enter

- customize key press
- meta-n opens choice: note / command
- add filter: number of times copied

- Actions menu: Add icons to options
