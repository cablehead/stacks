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

- page init has become really slow
    - use invoke on copy: do copy on Rust side
    - bring back metadata
    - bring back icon
    - use invoke on filter: do filter on Rust side
    - bring back basic image support
    - Bring back cursor handling
        - on new items
        - on resume
    - add some todos:
        - MRU on CAS
        - handle scrolling passed 400 items
        - bring back incremental update

- new clipboard items can stop updating

- delete items

- mark a source as don't track (for password managers, etc).
- clean up focus handling
    - reset to start state of 1 minute
- clean up meta panel. add:
    - image info
- handle clipboard images
    - in preview
    - when the user hits enter
- customize key press
- meta-n opens choice: note / command
- add filter: content type
- add filter: number of times copied
