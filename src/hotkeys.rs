//! Global hotkey registration and handling.

use std::ptr::null_mut;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey};
use crate::config::{parse_hotkey, Config};
use crate::hooks::with_manager;

pub const HOTKEY_TOGGLE: i32 = 1;

pub fn register_hotkeys(config: &Config) -> Result<(), &'static str> {
    unsafe {
        if let Some((mods, vk)) = parse_hotkey(&config.hotkeys.toggle_enabled) {
            if RegisterHotKey(null_mut(), HOTKEY_TOGGLE, mods, vk) == 0 {
                log::warn!("Failed to register hotkey '{}' for toggle", config.hotkeys.toggle_enabled);
            } else { 
                log::info!("Registered hotkey: {} (toggle)", config.hotkeys.toggle_enabled); 
            }
        }
        Ok(())
    }
}

pub fn unregister_hotkeys() {
    unsafe {
        UnregisterHotKey(null_mut(), HOTKEY_TOGGLE);
    }
}

pub fn handle_hotkey(wparam: usize) {
    if wparam as i32 == HOTKEY_TOGGLE {
        with_manager(|m| m.toggle_enabled());
    }
}
