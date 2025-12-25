//! Core window management logic.

use std::sync::atomic::{AtomicBool, Ordering};
use windows_sys::Win32::Foundation::HWND;

use crate::config::Config;
use crate::mode::WindowMode;
use crate::positioning::position_window;
use crate::window::{get_foreground_window, get_manageable_windows, Window};

static ENABLED: AtomicBool = AtomicBool::new(true);

pub struct WindowManager {
    windows: Vec<Window>,
    focus_index: Option<usize>,
    config: Config,
    mode: WindowMode,
    switching: bool,
}

impl WindowManager {
    pub fn new(config: Config, mode: WindowMode) -> Self {
        ENABLED.store(config.general.enabled, Ordering::SeqCst);
        Self { 
            windows: Vec::new(), 
            focus_index: None, 
            config, 
            mode, 
            switching: false,
        }
    }
    
    pub fn mode(&self) -> WindowMode { self.mode }
    pub fn is_enabled(&self) -> bool { ENABLED.load(Ordering::SeqCst) }
    pub fn window_count(&self) -> usize { self.windows.len() }

    pub fn toggle_enabled(&mut self) {
        let new_state = !ENABLED.load(Ordering::SeqCst);
        ENABLED.store(new_state, Ordering::SeqCst);
        log::info!("Window manager {}", if new_state { "ENABLED" } else { "DISABLED" });
        
        if !new_state {
            // Disabling: restore all windows
            for window in &self.windows { 
                window.restore(); 
            }
        }
    }

    pub fn initialize(&mut self) {
        // Get currently visible windows
        self.windows = get_manageable_windows(&self.config);
        log::info!("Found {} windows", self.windows.len());
        
        if self.windows.is_empty() { return; }
        
        // Find currently focused window, or use first
        let foreground = get_foreground_window();
        self.focus_index = foreground
            .and_then(|fg| self.windows.iter().position(|w| w.hwnd == fg as isize))
            .or(Some(0));
        
        if self.is_enabled() {
            if let Some(window) = self.focused_window() {
                let target = window.clone();
                self.apply_focus(&target);
            }
        }
    }

    pub fn focused_window(&self) -> Option<&Window> { 
        self.focus_index.and_then(|i| self.windows.get(i)) 
    }
    
    pub fn find_window(&self, hwnd: HWND) -> Option<usize> { 
        self.windows.iter().position(|w| w.hwnd == hwnd as isize) 
    }

    /// Called when user switches to a window (via Alt+Tab, click, etc.)
    pub fn on_focus_change(&mut self, hwnd: HWND) {
        if self.switching { return; }
        if !self.is_enabled() { return; }
        
        // Already focused on this window?
        if let Some(idx) = self.find_window(hwnd) {
            if self.focus_index == Some(idx) { return; }
            self.focus_index = Some(idx);
            if let Some(window) = self.windows.get(idx) { 
                self.apply_focus(&window.clone()); 
            }
        } else {
            // New window - add it to our list
            let window = Window::from_hwnd(hwnd);
            if window.should_manage(&self.config) {
                log::info!("Adding window: {}", window.title);
                self.windows.push(window.clone());
                self.focus_index = Some(self.windows.len() - 1);
                self.apply_focus(&window);
            }
        }
    }

    fn apply_focus(&self, target: &Window) {
        // Minimize all other windows
        for window in &self.windows { 
            if window.hwnd != target.hwnd { 
                window.minimize_no_activate(); 
            } 
        }
        
        // Position and show target
        position_window(target, self.config.general.window_size);
        target.focus();
    }

    /// Restore all windows when shutting down
    pub fn restore_all(&self) {
        log::info!("Restoring {} windows...", self.windows.len());
        for window in &self.windows {
            window.restore();
        }
    }
}
