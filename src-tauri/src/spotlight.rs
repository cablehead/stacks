// based on: https://github.com/zzzze/tauri-plugin-spotlight 🙏

use std::sync::Mutex;

use cocoa::{
    appkit::{
        CGFloat, NSApplicationActivateIgnoringOtherApps, NSWindow, NSWindowCollectionBehavior,
    },
    base::{id, nil, BOOL, NO, YES},
    foundation::{NSPoint, NSRect},
};
use objc::{
    class, msg_send,
    runtime::{Class, Object},
    sel, sel_impl,
};

use tauri::{GlobalShortcutManager, Manager, Window, Wry};

static SELF_KEY_PREFIX: &'static str = "self:";

#[derive(Debug)]
pub enum Error {
    FailedToLockMutex,
    FailedToRegisterShortcut,
    FailedToGetNSWindow,
    FailedToGetNSWorkspaceClass,
    FailedToCheckWindowVisibility,
    FailedToHideWindow,
    FailedToShowWindow,
}

pub fn init(window: &Window<Wry>) -> Result<(), Error> {
    set_spotlight_window_collection_behavior(&window)?;
    set_window_level(&window, 7)?;
    Ok(())
}

pub fn register_shortcut(window: &Window<Wry>, shortcut: &str) -> Result<(), Error> {
    let window = window.to_owned();
    let mut shortcut_manager = window.app_handle().global_shortcut_manager();
    shortcut_manager
        .register(shortcut, move || {
            if window.is_visible().unwrap() {
                hide(&window).unwrap();
            } else {
                show(&window).unwrap();
            }
        })
        .map_err(|_| Error::FailedToRegisterShortcut)?;
    Ok(())
}

pub fn show(window: &Window<Wry>) -> Result<(), Error> {
    if !window
        .is_visible()
        .map_err(|_| Error::FailedToCheckWindowVisibility)?
    {
        set_previous_app(get_frontmost_app_path());
        window.set_focus().map_err(|_| Error::FailedToShowWindow)?;
    }
    Ok(())
}

pub fn hide(window: &Window<Wry>) -> Result<(), Error> {
    if window
        .is_visible()
        .map_err(|_| Error::FailedToCheckWindowVisibility)?
    {
        window.hide().map_err(|_| Error::FailedToHideWindow)?;
        if let Ok(Some(prev_frontmost_window_path)) = get_previous_app() {
            if prev_frontmost_window_path.starts_with(SELF_KEY_PREFIX) {
                if let Some(window_label) = prev_frontmost_window_path.strip_prefix(SELF_KEY_PREFIX)
                {
                    if let Some(window) = window.app_handle().get_window(window_label) {
                        window.set_focus().map_err(|_| Error::FailedToShowWindow)?;
                    }
                }
            } else {
                active_another_app(&prev_frontmost_window_path)?;
            }
        }
    }
    Ok(())
}

use lazy_static::lazy_static;

lazy_static! {
    static ref PREVIOUS_APP: Mutex<Option<String>> = Mutex::new(None);
}

pub fn set_previous_app(value: Option<String>) {
    let mut previous_app = PREVIOUS_APP.lock().unwrap();
    *previous_app = value;
}

pub fn get_previous_app() -> Result<Option<String>, Error> {
    let previous_app = PREVIOUS_APP.lock().map_err(|_| Error::FailedToLockMutex)?;
    Ok((*previous_app).clone())
}

#[macro_export]
macro_rules! nsstring_to_string {
    ($ns_string:expr) => {{
        use objc::{sel, sel_impl};
        let utf8: id = objc::msg_send![$ns_string, UTF8String];
        let string = if !utf8.is_null() {
            Some({
                std::ffi::CStr::from_ptr(utf8 as *const std::ffi::c_char)
                    .to_string_lossy()
                    .into_owned()
            })
        } else {
            None
        };
        string
    }};
}

fn active_another_app(bundle_url: &str) -> Result<(), Error> {
    let workspace = unsafe {
        if let Some(workspace_class) = Class::get("NSWorkspace") {
            let shared_workspace: *mut Object = msg_send![workspace_class, sharedWorkspace];
            shared_workspace
        } else {
            return Err(Error::FailedToGetNSWorkspaceClass);
        }
    };
    let running_apps = unsafe {
        let running_apps: *mut Object = msg_send![workspace, runningApplications];
        running_apps
    };
    let target_app = unsafe {
        let count = msg_send![running_apps, count];
        let mut target_app: Option<*mut Object> = None;
        for i in 0..count {
            let app: *mut Object = msg_send![running_apps, objectAtIndex: i];
            let app_bundle_url: id = msg_send![app, bundleURL];
            let path: id = msg_send![app_bundle_url, path];
            let app_bundle_url_str = nsstring_to_string!(path);
            if let Some(app_bundle_url_str) = app_bundle_url_str {
                if app_bundle_url_str == bundle_url.to_string() {
                    target_app = Some(app);
                    break;
                }
            }
        }
        target_app
    };
    if let Some(app) = target_app {
        unsafe {
            let _: () = msg_send![app, activateWithOptions: NSApplicationActivateIgnoringOtherApps];
        };
    }
    Ok(())
}

/*
fn register_shortcut_for_window(
    window: &Window<Wry>,
    window_config: &WindowConfig,
) -> Result<(), Error> {
    let window = window.to_owned();
    let mut shortcut_manager = window.app_handle().global_shortcut_manager();
    shortcut_manager
        .register(&window_config.shortcut, move || {
            let app_handle = window.app_handle();
            let manager = app_handle.state::<SpotlightManager>();
            if window.is_visible().unwrap() {
                manager.hide(&window).unwrap();
            } else {
                manager.show(&window).unwrap();
            }
        })
        .map_err(|_| Error::FailedToRegisterShortcut)?;
    Ok(())
}
*/

/*
fn handle_focus_state_change(window: &Window<Wry>) {
    let w = window.to_owned();
    window.on_window_event(move |event| {
        if let WindowEvent::Focused(false) = event {
            unregister_close_shortcut(&w).unwrap(); // FIXME:
            w.hide().unwrap();
        } else {
            register_close_shortcut(&w).unwrap(); // FIXME:
        }
    });
}
*/

/// Set the behaviors that makes the window appear on all workspaces
fn set_spotlight_window_collection_behavior(window: &Window<Wry>) -> Result<(), Error> {
    let handle: id = window.ns_window().map_err(|_| Error::FailedToGetNSWindow)? as _;
    unsafe {
        handle.setCollectionBehavior_(
            NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenPrimary
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorIgnoresCycle,
        );
    };
    Ok(())
}

fn set_window_level(window: &Window<Wry>, level: i32) -> Result<(), Error> {
    let handle: id = window.ns_window().map_err(|_| Error::FailedToGetNSWindow)? as _;
    unsafe { handle.setLevel_((level).into()) };
    Ok(())
}

pub fn get_frontmost_app_path() -> Option<String> {
    let shared_workspace: id = unsafe { msg_send![class!(NSWorkspace), sharedWorkspace] };
    let frontmost_app: id = unsafe { msg_send![shared_workspace, frontmostApplication] };
    let bundle_url: id = unsafe { msg_send![frontmost_app, bundleURL] };
    let path: id = unsafe { msg_send![bundle_url, path] };
    unsafe { nsstring_to_string!(path) }
}
