//! Window abstraction and utilities.

use std::hash::{Hash, Hasher};
use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM, RECT, TRUE};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

// Make Window Send+Sync safe by wrapping HWND
#[derive(Debug, Clone)]
pub struct Window {
    pub hwnd: isize,  // Store as isize for Send+Sync
    pub title: String,
    pub class_name: String,
}

impl Window {
    pub fn from_hwnd(hwnd: HWND) -> Self {
        Self { hwnd: hwnd as isize, title: get_window_title(hwnd), class_name: get_window_class(hwnd) }
    }

    fn raw_hwnd(&self) -> HWND { self.hwnd as HWND }

    pub fn is_valid(&self) -> bool {
        unsafe { IsWindow(self.raw_hwnd()) != 0 }
    }

    pub fn is_visible(&self) -> bool {
        unsafe { IsWindowVisible(self.raw_hwnd()) != 0 }
    }

    pub fn should_manage(&self, config: &crate::config::Config) -> bool {
        if !self.is_valid() { return false; }
        if config.is_class_excluded(&self.class_name) { return false; }
        if config.is_title_excluded(&self.title) { return false; }

        let style = unsafe { GetWindowLongW(self.raw_hwnd(), GWL_STYLE) as u32 };
        let ex_style = unsafe { GetWindowLongW(self.raw_hwnd(), GWL_EXSTYLE) as u32 };

        if style & WS_CHILD != 0 { return false; }
        if ex_style & WS_EX_TOOLWINDOW != 0 { return false; }

        let has_title = !self.title.is_empty();
        let is_app_window = ex_style & WS_EX_APPWINDOW != 0;
        if !has_title && !is_app_window { return false; }

        let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        if unsafe { GetWindowRect(self.raw_hwnd(), &mut rect) } != 0 {
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;
            if width < 100 || height < 100 { return false; }
        }

        self.is_visible()
    }

    pub fn minimize_no_activate(&self) {
        unsafe { 
            ShowWindow(self.raw_hwnd(), SW_MINIMIZE); 
        }
    }

    /// Restore window to normal state (for when disabling manager)
    pub fn restore(&self) {
        unsafe {
            ShowWindow(self.raw_hwnd(), SW_RESTORE);
        }
    }

    pub fn focus(&self) {
        unsafe {
            SetForegroundWindow(self.raw_hwnd());
        }
    }

    pub fn set_position(&self, x: i32, y: i32, width: i32, height: i32) {
        unsafe {
            // First restore if minimized
            if IsIconic(self.raw_hwnd()) != 0 {
                ShowWindow(self.raw_hwnd(), SW_RESTORE);
            }
            // Then set position and size
            SetWindowPos(
                self.raw_hwnd(), 
                HWND_TOP, 
                x, y, width, height, 
                SWP_SHOWWINDOW
            );
        }
    }
}

impl PartialEq for Window { fn eq(&self, other: &Self) -> bool { self.hwnd == other.hwnd } }
impl Eq for Window {}
impl Hash for Window { fn hash<H: Hasher>(&self, state: &mut H) { self.hwnd.hash(state); } }

// Mark Window as Send+Sync since we're using isize now
unsafe impl Send for Window {}
unsafe impl Sync for Window {}

pub fn get_window_title(hwnd: HWND) -> String {
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        if len == 0 { return String::new(); }
        let mut buffer: Vec<u16> = vec![0; (len + 1) as usize];
        let copied = GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);
        if copied == 0 { return String::new(); }
        String::from_utf16_lossy(&buffer[..copied as usize])
    }
}

pub fn get_window_class(hwnd: HWND) -> String {
    unsafe {
        let mut buffer: Vec<u16> = vec![0; 256];
        let len = GetClassNameW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);
        if len == 0 { return String::new(); }
        String::from_utf16_lossy(&buffer[..len as usize])
    }
}

pub fn enumerate_windows() -> Vec<HWND> {
    let mut windows: Vec<HWND> = Vec::new();
    unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let windows = &mut *(lparam as *mut Vec<HWND>);
        windows.push(hwnd);
        TRUE
    }
    unsafe { EnumWindows(Some(enum_callback), &mut windows as *mut Vec<HWND> as LPARAM); }
    windows
}

pub fn get_manageable_windows(config: &crate::config::Config) -> Vec<Window> {
    enumerate_windows().into_iter().map(Window::from_hwnd).filter(|w| w.should_manage(config)).collect()
}

/// Get the currently focused foreground window
pub fn get_foreground_window() -> Option<HWND> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() { None } else { Some(hwnd) }
    }
}
