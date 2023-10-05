# Tauri Plugin: Spotlight

A Tauri plugin that provides a MacOS Spotlight-like search functionality for Tauri windows.

## Overview

Spotlight is a Tauri plugin that provides a user-friendly and intuitive way to interact with
your desktop applications - the Spotlight search-like interface.

This plugin is currently implemented for macOS, but has basic implementations for other platforms.

Features:

1. Allows users to define hotkeys for showing and hiding the window
2. Any window can register to implement the features provided by this plugin
3. Window will automatically hide when losing focus
4. Supports multiple displays (currently only available on macOS)
5. Window will always appear on top and reactivate the previously active window upon hiding (currently only available on macOS)

## Installation

Install the Core plugin by adding the following to your Cargo.toml file:

`src-tauri/Cargo.toml`

```toml
[dependencies]
tauri-plugin-spotlight = { git = "https://github.com/zzzze/tauri-plugin-spotlight" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

```bash
pnpm add tauri-plugin-spotlight-api
# or
npm add tauri-plugin-spotlight-api
# or
yarn add tauri-plugin-spotlight-api
```

## Usage

### Backend

There are three ways to configure the plugin:

1. Register the spotlight plugin with Tauri:

`src-tauri/src/main.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_spotlight::init(Some(tauri_plugin_spotlight::PluginConfig {
            windows: Some(vec![
                tauri_plugin_spotlight::WindowConfig {
                    label: String::from("main"),
                    shortcut: String::from("Ctrl+Shift+J"),
                    macos_window_level: Some(20), // Default 24
                },
            ]),
            global_close_shortcut: Some(String::from("Escape")),
        })))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

2. Configure the plugin in your Tauri app's configuration file:

`src-tauri/tauri.conf.json`

```json
{
  "plugins": {
    "spotlight": {
      "windows": [{
        "label": "main",
        "shortcut": "Ctrl+Shift+J",
        "macos_window_level": 20
      }],
      "global_close_shortcut": "Escape"
    }
  }
}
```

`src-tauri/src/main.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_spotlight::init(None))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

3. Manually register window shortcut keys

`src-tauri/src/main.rs`

```rust
use tauri_plugin_spotlight::ManagerExt;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_spotlight::init(Some(tauri_plugin_spotlight::PluginConfig {
            windows: None,
            global_close_shortcut: Some(String::from("Escape")),
        })))
        .setup(|mut app| {
            if let Some(window) = app.get_window("main") {
                app.spotlight().init_spotlight_window(&window, "Ctrl+Shift+J").unwrap();
            }
            app_modifier::apply(&mut app);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running application");
}
```

The configuration parameters written in `tauri.conf.json` and `tauri_plugin_spotlight::init`
will be automatically merged with `tauri_plugin_spotlight::init` taking higher priority.

### Frontend

Use the `hide` function to make a spotlight window invisible:

```typescript
import { hide } from 'tauri-plugin-spotlight-api';

void hide();
```

## Example App

### Prepare

1. Build frontend API the plugin.

```bash
pnpm i
pnpm build
```

2. Install dependencies of example app.

```bash
cd examples/react-app
pnpm i
```

3. Start example app.

```bash
pnpm tauri dev
```

## Thanks

This plugin was inspired by the [tauri-macos-spotlight-example](https://github.com/ahkohd/tauri-macos-spotlight-example)
project by [ahkohd](https://github.com/ahkohd), and borrows heavily from its codebase. Thanks to [ahkohd](https://github.com/ahkohd) and the contributors
to [tauri-macos-spotlight-example](https://github.com/ahkohd/tauri-macos-spotlight-example) for their hard work and open-source contributions!
