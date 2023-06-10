# Stacks

## Features

### Quick Filter

![filter screenshot](./docs/filter-screenshot.webp)

### Dark Mode

![dark mode](./docs/dark-mode.webp)

## Development

```
parcel watch src/index.html --no-hmr --dist-dir ./site
cargo tauri dev

# Type checking:
./scripts/ts-check.sh
```

## Todo

- add an action to perform screenshot scrape
- preference panel

- better cursor handling
    - dedicated focus handling when the filter changes
    - if the first item isn't selected and an item is added, move the cursor
      down one to keep focus steady
    - unless the item being added is the item focused, in which case, jump to
      the first?? - maybe
    - reset to start state of 1 minute

- mark a source as don't track (for password managers, etc).

- clean up meta panel. add:
    - image info

- handle clipboard images
    - in preview: improve transparency
    - when the user hits enter

- new clipboard items can stop updating

- customize key press
- meta-n opens choice: note / command
- add filter: content type
- add filter: number of times copied
