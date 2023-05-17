# Tauri + Vanilla

## How to run

```
cargo tauri dev -- -- $(realpath ./s)
parcel watch app.jsx --no-hmr --dist-dir ./src
```

## Todo

### Next

- mix and match clipboard, notes and commands
- integrate clipboard
- show as much of surrounding items as possible
- reduce dependencies
    - i guess that means xs and x-pasteboard should be built-in

### Clipboard

- [ ] show the last time copied
- [ ] deduplicate items
    - show number of times copied

## Leads

- https://github.com/ast-grep/ast-grep

