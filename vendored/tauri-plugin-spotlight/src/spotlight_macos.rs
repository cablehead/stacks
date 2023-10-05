use std::sync::Mutex;
use cocoa::{
    appkit::{CGFloat, NSMainMenuWindowLevel, NSWindow, NSWindowCollectionBehavior, NSApplicationActivateIgnoringOtherApps},
    base::{id, nil, BOOL, NO, YES},
    foundation::{NSPoint, NSRect},
};
use objc::{
    class, msg_send, sel, sel_impl,
    runtime::{Class, Object},
};
use tauri::{
    GlobalShortcutManager, Manager, PhysicalPosition, PhysicalSize, Window, WindowEvent, Wry,
};

use super::WindowConfig;
use super::PluginConfig;
use super::Error;

static SELF_KEY_PREFIX: &'static str = "self:";

#[derive(Default, Debug)]
pub struct SpotlightManager {
    pub config: PluginConfig,
    previous_app: Mutex<Option<String>>,
    registered_window: Mutex<Vec<String>>,
}

impl SpotlightManager {
    pub fn new(config: PluginConfig) -> Self {
        let mut manager = Self::default();
        manager.config = config;
        manager
    }

    fn get_window_config(&self, window: &Window<Wry>) -> Option<WindowConfig> {
        if let Some(window_configs) = self.config.windows.clone() {
            for window_config in window_configs {
                if window.label() == window_config.label {
                    return Some(window_config.clone());
                }
            }
        }
        None
    }

    pub fn init_spotlight_window(&self, window: &Window<Wry>) -> Result<(), Error> {
        let window_config = match self.get_window_config(&window) {
            Some(window_config) => window_config,
            None => return Ok(()),
        };
        let registered = set_previous_app(&window, get_frontmost_app_path())?;
        if !registered {
            register_shortcut_for_window(&window, &window_config)?;
            register_close_shortcut(&window)?;
            handle_focus_state_change(&window);
            set_spotlight_window_collection_behavior(&window)?;
            set_window_level(&window, &window_config)?;
        }
        Ok(())
    }

    pub fn show(&self, window: &Window<Wry>) -> Result<(), Error> {
        if !window.is_visible().map_err(|_| Error::FailedToCheckWindowVisibility)? {
            // position_window_at_the_center_of_the_monitor_with_cursor(&window).unwrap();
            set_previous_app(&window, get_frontmost_app_path())?;
            window.set_focus().map_err(|_| Error::FailedToShowWindow)?;
        }
        Ok(())
    }

    pub fn hide(&self, window: &Window<Wry>) -> Result<(), Error> {
        if window.is_visible().map_err(|_| Error::FailedToCheckWindowVisibility)? {
            window.hide().map_err(|_| Error::FailedToHideWindow)?;
            if let Ok(Some(prev_frontmost_window_path)) = get_previous_app(&window) {
                if prev_frontmost_window_path.starts_with(SELF_KEY_PREFIX) {
                    if let Some(window_label) = prev_frontmost_window_path.strip_prefix(SELF_KEY_PREFIX) {
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
}

fn set_previous_app(window: &Window<Wry>, value: Option<String>) -> Result<bool, Error> {
    let label = window.label().to_string();
    let handle = window.app_handle();
    let state = handle.state::<SpotlightManager>();
    let mut registered_window = state
        .registered_window
        .lock()
        .map_err(|_| Error::FailedToLockMutex)?;
    let existed = registered_window.contains(&label);
    let mut value = value;
    if let Some(current_app_path) = std::env::current_exe().map_err(|_| Error::FailedToGetExecutablePath)?.to_str() {
        if Some(current_app_path.to_string()) == value {
            let mut activated_non_spotlight_window: Option<String> = None;
            for window in handle.windows().values() {
                if registered_window.contains(&window.label().to_string()) {
                    continue;
                }
                if let Ok(visible) = window.is_visible() {
                    if visible {
                        activated_non_spotlight_window = Some(window.label().to_string());
                        break;
                    }
                }
            }
            if let Some(activated_non_spotlight_window) = activated_non_spotlight_window {
                value = Some(format!("{}{}", SELF_KEY_PREFIX, activated_non_spotlight_window));
            } else {
                return Ok(existed);
            }
        }
    }
    if !existed {
        registered_window.push(label);
    }
    let mut previous_app = state
        .previous_app
        .lock()
        .map_err(|_| Error::FailedToLockMutex)?;
    *previous_app = value;
    Ok(existed)
}

fn get_previous_app(window: &Window<Wry>) -> Result<Option<String>, Error> {
    let handle = window.app_handle();
    let state = handle.state::<SpotlightManager>();
    let previous_app = state
        .previous_app
        .lock()
        .map_err(|_| Error::FailedToLockMutex)?;
    Ok(previous_app.clone())
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

fn register_shortcut_for_window(window: &Window<Wry>, window_config: &WindowConfig) -> Result<(), Error> {
    let window = window.to_owned();
    let mut shortcut_manager = window.app_handle().global_shortcut_manager();
    shortcut_manager.register(&window_config.shortcut, move || {
        let app_handle = window.app_handle();
        let manager = app_handle.state::<SpotlightManager>();
        if window.is_visible().unwrap() {
            manager.hide(&window).unwrap();
        } else {
            manager.show(&window).unwrap();
        }
    }).map_err(|_| Error::FailedToRegisterShortcut)?;
    Ok(())
}

fn register_close_shortcut(window: &Window<Wry>) -> Result<(), Error> {
    let window = window.to_owned();
    let mut shortcut_manager = window.app_handle().global_shortcut_manager();
    let app_handle = window.app_handle();
    let manager = app_handle.state::<SpotlightManager>();
    if let Some(close_shortcut) = manager.config.global_close_shortcut.clone() {
        if let Ok(registered) = shortcut_manager.is_registered(&close_shortcut) {
            if !registered {
                shortcut_manager.register(&close_shortcut, move || {
                    let app_handle = window.app_handle();
                    let state = app_handle.state::<SpotlightManager>();
                    let registered_window = state.registered_window.lock().unwrap();
                    let window_labels = registered_window.clone();
                    std::mem::drop(registered_window);
                    for label in window_labels {
                        if let Some(window) = app_handle.get_window(&label) {
                            state.hide(&window).unwrap();
                        }
                    }
                }).map_err(|_| Error::FailedToRegisterShortcut)?;
            }
        } else {
            return Err(Error::FailedToRegisterShortcut);
        }
    }
    Ok(())
}

fn unregister_close_shortcut(window: &Window<Wry>) -> Result<(), Error> {
    let window = window.to_owned();
    let mut shortcut_manager = window.app_handle().global_shortcut_manager();
    let app_handle = window.app_handle();
    let manager = app_handle.state::<SpotlightManager>();
    if let Some(close_shortcut) = manager.config.global_close_shortcut.clone() {
        if let Ok(registered) = shortcut_manager.is_registered(&close_shortcut) {
            if registered {
                shortcut_manager.unregister(&close_shortcut).map_err(|_| Error::FailedToUnregisterShortcut)?;
            }
        } else {
            return Err(Error::FailedToRegisterShortcut);
        }
    }
    Ok(())
}

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

/// Positions a given window at the center of the monitor with cursor
fn position_window_at_the_center_of_the_monitor_with_cursor(window: &Window<Wry>) -> Result<(), Error> {
    if let Some(monitor) = get_monitor_with_cursor() {
        let display_size = monitor.size.to_logical::<f64>(monitor.scale_factor);
        let display_pos = monitor.position.to_logical::<f64>(monitor.scale_factor);
        let handle: id = window.ns_window().map_err(|_| Error::FailedToGetNSWindow)? as _;
        let win_frame: NSRect = unsafe { handle.frame() };
        let rect = NSRect {
            origin: NSPoint {
                x: (display_pos.x + (display_size.width / 2.0)) - (win_frame.size.width / 2.0),
                y: (display_pos.y + (display_size.height / 2.0)) - (win_frame.size.height / 2.0),
            },
            size: win_frame.size,
        };
        let _: () = unsafe { msg_send![handle, setFrame: rect display: YES] };
    }
    Ok(())
}

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

fn set_window_level(window: &Window<Wry>, window_config: &WindowConfig) -> Result<(), Error> {
    let window_level = match window_config.macos_window_level {
        Some(level) => level,
        None => NSMainMenuWindowLevel,
    };
    let handle: id = window.ns_window().map_err(|_| Error::FailedToGetNSWindow)? as _;
    unsafe { handle.setLevel_((window_level).into()) };
    Ok(())
}

struct Monitor {
    #[allow(dead_code)]
    pub name: Option<String>,
    pub size: PhysicalSize<u32>,
    pub position: PhysicalPosition<i32>,
    pub scale_factor: f64,
}

#[link(name = "Foundation", kind = "framework")]
extern "C" {
    pub fn NSMouseInRect(aPoint: NSPoint, aRect: NSRect, flipped: BOOL) -> BOOL;
}

/// Returns the Monitor with cursor
fn get_monitor_with_cursor() -> Option<Monitor> {
    objc::rc::autoreleasepool(|| {
        let mouse_location: NSPoint = unsafe { msg_send![class!(NSEvent), mouseLocation] };
        let screens: id = unsafe { msg_send![class!(NSScreen), screens] };
        let screens_iter: id = unsafe { msg_send![screens, objectEnumerator] };
        let mut next_screen: id;

        let frame_with_cursor: Option<NSRect> = loop {
            next_screen = unsafe { msg_send![screens_iter, nextObject] };
            if next_screen == nil {
                break None;
            }

            let frame: NSRect = unsafe { msg_send![next_screen, frame] };
            let is_mouse_in_screen_frame: BOOL =
                unsafe { NSMouseInRect(mouse_location, frame, NO) };
            if is_mouse_in_screen_frame == YES {
                break Some(frame);
            }
        };

        if let Some(frame) = frame_with_cursor {
            let name: id = unsafe { msg_send![next_screen, localizedName] };
            let screen_name = unsafe { nsstring_to_string!(name) };
            let scale_factor: CGFloat = unsafe { msg_send![next_screen, backingScaleFactor] };
            let scale_factor: f64 = scale_factor;

            return Some(Monitor {
                name: screen_name,
                position: PhysicalPosition {
                    x: (frame.origin.x * scale_factor) as i32,
                    y: (frame.origin.y * scale_factor) as i32,
                },
                size: PhysicalSize {
                    width: (frame.size.width * scale_factor) as u32,
                    height: (frame.size.height * scale_factor) as u32,
                },
                scale_factor,
            });
        }

        None
    })
}

pub fn get_frontmost_app_path() -> Option<String> {
    let shared_workspace: id = unsafe { msg_send![class!(NSWorkspace), sharedWorkspace] };
    let frontmost_app: id = unsafe { msg_send![shared_workspace, frontmostApplication] };
    let bundle_url: id = unsafe { msg_send![frontmost_app, bundleURL] };
    let path: id = unsafe { msg_send![bundle_url, path] };
    unsafe { nsstring_to_string!(path) }
}
