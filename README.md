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
parcel watch src/index.html --no-hmr --dist-dir ./site
cargo tauri dev

# Type checking:
./scripts/ts-check.sh
```

## Release

```
RELEASE=v0.5.2
./build.sh
cat changes/$RELEASE | ./scripts/release.sh
# commit and push
gh release create $RELEASE $RELEASE_PATH/* -n "$(cat changes/$RELEASE)"
```

## Todo

- Fix clicking on Capture in the StatusBar

- Preference panel

- Theme: initialize theme to the system preference
    - set a time limit when manually set

- Editor capture should create an xs.add row: with parent set to the id the
  editor was triggered on: this should be merged version that's put on the
  clipboard

- better cursor handling
    - dedicated focus handling when the filter changes
    - if the first item isn't selected and an item is added, move the cursor
      down one to keep focus steady
    - unless the item being added is the item focused, in which case, jump to
      the first?? - maybe
    - reset to start state of 1 minute

- mark a source as don't track (for password managers, etc).

- meta panel. add: image info

- handle clipboard images
    - in preview: improve transparency
    - when the user hits enter

- customize key press
- meta-n opens choice: note / command
- add filter: content type
- add filter: number of times copied

- Actions menu: Add icons to options

- Microlink action: done, but
    - need UI to indicate scrape is in progress
    - need to surface errors
    - feels heavy

