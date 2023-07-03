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

## Release

```
# update Cargo.toml and tauri.conf.json for new version
# set RELEASE to the new version, e.g
RELEASE=v0.5.2
./build.sh
cat changes/$RELEASE | ./scripts/release.sh
# commit and push
gh release create $RELEASE $RELEASE_PATH/* -n "$(cat changes/$RELEASE)"
```

## Todo


### To release

- Delete inside a stack should just remove the item from the stack
- Deleting a Stack should first take away Stack-hood, and then actually delete
- Order items inside a Stack by last touch time
- Handle Adding the same item to a Stack more than once
- Add some actions for New Stack gestures?
- nested stacks don't see all items for some reasons
- clicking on Copy doesn't work
- click on Capture doesn't work


### and then

- Going to want multi-select to add

- shift-enter to copy but keep stacks open: maybe
- or shift-ever: replace

- Editor
    - access clips while editor is open

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
    - when the user hits enter

- customize key press
- meta-n opens choice: note / command
- add filter: number of times copied

- Actions menu: Add icons to options
