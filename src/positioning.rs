//! Window positioning with size support.

use windows_sys::Win32::Foundation::RECT;
use windows_sys::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTOPRIMARY};
use windows_sys::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, SPI_GETWORKAREA};

use crate::window::Window;

#[derive(Debug, Clone, Copy)]
pub struct ScreenRect { pub x: i32, pub y: i32, pub width: i32, pub height: i32 }

impl ScreenRect {
    pub fn primary_work_area() -> Self {
        unsafe {
            let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            SystemParametersInfoW(SPI_GETWORKAREA, 0, &mut rect as *mut RECT as *mut _, 0);
            Self { x: rect.left, y: rect.top, width: rect.right - rect.left, height: rect.bottom - rect.top }
        }
    }

    pub fn from_window(window: &Window) -> Self {
        unsafe {
            let hwnd = window.hwnd as *mut std::ffi::c_void;
            let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY);
            let mut info = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                rcMonitor: RECT { left: 0, top: 0, right: 0, bottom: 0 },
                rcWork: RECT { left: 0, top: 0, right: 0, bottom: 0 },
                dwFlags: 0,
            };
            if GetMonitorInfoW(monitor, &mut info) != 0 {
                let rect = info.rcWork;
                Self { x: rect.left, y: rect.top, width: rect.right - rect.left, height: rect.bottom - rect.top }
            } else {
                Self::primary_work_area()
            }
        }
    }

    /// Calculate window rect for given window_size percentage (10-100)
    pub fn sized(&self, window_size: u8) -> (i32, i32, i32, i32) {
        let size = window_size.clamp(10, 100) as i32;
        let win_width = (self.width * size) / 100;
        let win_height = (self.height * size) / 100;
        let x = self.x + (self.width - win_width) / 2;
        let y = self.y + (self.height - win_height) / 2;
        (x, y, win_width, win_height)
    }
}

/// Position window centered at given size percentage
pub fn position_window(window: &Window, window_size: u8) {
    let screen = ScreenRect::from_window(window);
    let (x, y, w, h) = screen.sized(window_size);
    log::debug!("Positioning '{}' to ({}, {}) {}x{}", window.title, x, y, w, h);
    window.set_position(x, y, w, h);
}
