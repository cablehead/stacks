use std::sync::Mutex;
use tauri::{
    GlobalShortcutManager, Manager, Window, WindowEvent, Wry,
};
use super::{PluginConfig, WindowConfig};
use super::Error;

#[derive(Default, Debug)]
pub struct SpotlightManager {
    pub config: PluginConfig,
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
        let label = window.label().to_string();
        let handle = window.app_handle();
        let state = handle.state::<SpotlightManager>();
        let mut registered_window = state
            .registered_window
            .lock()
            .map_err(|_| Error::FailedToLockMutex)?;
        let registered = registered_window.contains(&label);
        if !registered {
            register_shortcut_for_window(&window, &window_config)?;
            register_close_shortcut(&window)?;
            handle_focus_state_change(&window);
            registered_window.push(label);
        }
        Ok(())
    }

    pub fn show(&self, window: &Window<Wry>) -> Result<(), Error> {
        if !window.is_visible().map_err(|_| Error::FailedToCheckWindowVisibility)? {
            window.show().map_err(|_| Error::FailedToShowWindow)?;
            window.set_focus().map_err(|_| Error::FailedToShowWindow)?;
        }
        Ok(())
    }

    pub fn hide(&self, window: &Window<Wry>) -> Result<(), Error> {
        if window.is_visible().map_err(|_| Error::FailedToCheckWindowVisibility)? {
            window.hide().map_err(|_| Error::FailedToHideWindow)?;
        }
        Ok(())
    }
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
                            window.hide().unwrap();
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
