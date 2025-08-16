## Version Bumping

To bump the current version:

1. Update version in `src-tauri/Cargo.toml` (line 3)
2. Update version in `src-tauri/tauri.conf.json` (line 4)
3. Run `cargo check` to update Cargo.lock
4. Both files must have matching versions

Example: `0.15.14-dev` → `0.15.15-dev`
