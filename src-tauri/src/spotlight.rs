use tauri::AppHandle;
use tauri_nspanel::ManagerExt;
use tauri_nspanel::WindowExt;

use cocoa::{
    appkit::{NSWindow, NSWindowCollectionBehavior},
    base::id,
};
use objc::{msg_send, sel, sel_impl};

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Shortcut {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub command: bool,
}

impl Shortcut {
    // Method to generate a macOS-compatible shortcut string
    pub fn to_macos_shortcut(&self) -> String {
        let mut parts = vec![];
        if self.shift {
            parts.push("Shift");
        }
        if self.ctrl {
            parts.push("Control");
        }
        if self.alt {
            parts.push("Option");
        }
        if self.command {
            parts.push("Command");
        }
        parts.push("Space");
        parts.join("+")
    }
}

use tauri::{GlobalShortcutManager, Window, WindowEvent, Wry};

#[derive(Debug)]
pub enum Error {
    RegisterShortcut,
    GetNSWindow,
}

#[allow(non_upper_case_globals)]
const NSWindowStyleMaskNonActivatingPanel: i32 = 1 << 7;

pub fn init(window: &Window<Wry>) -> Result<(), Error> {
    handle_focus_state_change(window);

    let panel = window.to_panel().unwrap();
    panel.set_collection_behaviour(
        NSWindowCollectionBehavior::NSWindowCollectionBehaviorMoveToActiveSpace
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorTransient
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorIgnoresCycle,
    );

    let current_mask: i32 = unsafe { msg_send![panel, styleMask] };
    panel.set_style_mask(current_mask | NSWindowStyleMaskNonActivatingPanel);

    set_window_level(window, 7)?;
    Ok(())
}

pub fn register_shortcut(app: AppHandle<Wry>, shortcut: &str) -> Result<(), Error> {
    let mut shortcut_manager = app.global_shortcut_manager();
    shortcut_manager
        .unregister_all()
        .map_err(|_| Error::RegisterShortcut)?;

    shortcut_manager
        .register(shortcut, move || {
            let panel = app.get_panel("main").unwrap_or_else(|e| {
                eprintln!("Failed to get panel: {e:?}");
                panic!("Panel not found")
            });

            if panel.is_visible() {
                panel.order_out(None);
            } else {
                panel.show();
            }
        })
        .map_err(|_| Error::RegisterShortcut)?;
    Ok(())
}

pub fn hide(app: &tauri::AppHandle) -> Result<(), Error> {
    let panel = app.get_panel("main").unwrap();
    panel.order_out(None);
    Ok(())
}

fn handle_focus_state_change(window: &Window<Wry>) {
    let w = window.to_owned();
    window.on_window_event(move |event| {
        if let WindowEvent::Focused(false) = event {
            w.hide().unwrap();
        }
    });
}

fn set_window_level(window: &Window<Wry>, level: i32) -> Result<(), Error> {
    let handle: id = window.ns_window().map_err(|_| Error::GetNSWindow)? as _;
    unsafe { handle.setLevel_((level).into()) };
    Ok(())
}
