//! Splash screen with mode selection.

use std::ptr::null_mut;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM, RECT};
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

use crate::mode::WindowMode;

// Constants not in windows-sys
const LWA_ALPHA: u32 = 0x00000002;
const TRANSPARENT: i32 = 1;
const FW_THIN: u32 = 100;
const FW_LIGHT: u32 = 300;
const FW_NORMAL: u32 = 400;
const FW_SEMIBOLD: u32 = 600;
const DEFAULT_CHARSET: u32 = 1;
const CLIP_DEFAULT_PRECIS: u32 = 0;
const CLEARTYPE_NATURAL_QUALITY: u32 = 6;
const OUT_TT_PRECIS: u32 = 4;
const DT_CENTER: u32 = 0x00000001;
const DT_SINGLELINE: u32 = 0x00000020;
const DEFAULT_PITCH: u32 = 0;
const FF_DONTCARE: u32 = 0;

// Virtual key codes
const VK_UP: usize = 0x26;
const VK_DOWN: usize = 0x28;
const VK_RETURN: usize = 0x0D;
const VK_ESCAPE: usize = 0x1B;

const SPLASH_CLASS: &str = "WindowManagerSplash";
const TIMER_ID: usize = 1;
const TIMER_INTERVAL: u32 = 16; // ~60fps

// Animation state
static mut ANIMATION: AnimationState = AnimationState::new();

struct AnimationState {
    phase: Phase,
    opacity: u8,
    selected_index: usize,
    confirmed: bool,
}

fn animation() -> &'static mut AnimationState {
    unsafe { &mut *std::ptr::addr_of_mut!(ANIMATION) }
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    FadeIn,
    Interactive,
    FadeOut,
    Done,
}

impl AnimationState {
    const fn new() -> Self {
        Self {
            phase: Phase::FadeIn,
            opacity: 0,
            selected_index: 0,
            confirmed: false,
        }
    }
    
    fn reset(&mut self) {
        self.phase = Phase::FadeIn;
        self.opacity = 0;
        self.selected_index = 0;
        self.confirmed = false;
    }
    
    fn selected_mode(&self) -> WindowMode {
        WindowMode::all()[self.selected_index]
    }
    
    fn move_selection(&mut self, delta: isize) {
        let modes = WindowMode::all();
        let len = modes.len() as isize;
        let mut new_index = (self.selected_index as isize + delta).rem_euclid(len) as usize;
        
        // Skip unavailable modes when navigating
        let start = new_index;
        loop {
            if modes[new_index].is_available() {
                break;
            }
            new_index = ((new_index as isize + delta.signum()).rem_euclid(len)) as usize;
            if new_index == start {
                break; // No available modes (shouldn't happen)
            }
        }
        
        self.selected_index = new_index;
    }
    
    fn confirm(&mut self) {
        if self.selected_mode().is_available() {
            self.confirmed = true;
            self.phase = Phase::FadeOut;
        }
    }
}

/// Shows the splash screen and returns the selected window mode.
pub fn show_splash() -> WindowMode {
    unsafe {
        animation().reset();
        
        let hinstance = GetModuleHandleW(null_mut());
        
        // Register window class
        let class_name = to_wstring(SPLASH_CLASS);
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(splash_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: null_mut(),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: null_mut(),
            lpszMenuName: null_mut(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: null_mut(),
        };
        RegisterClassExW(&wc);
        
        // Get screen size
        let screen_width = GetSystemMetrics(SM_CXSCREEN);
        let screen_height = GetSystemMetrics(SM_CYSCREEN);
        
        // Create layered window
        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            class_name.as_ptr(),
            null_mut(),
            WS_POPUP | WS_VISIBLE,
            0, 0, screen_width, screen_height,
            null_mut(),
            null_mut(),
            hinstance,
            null_mut(),
        );
        
        if hwnd.is_null() {
            return WindowMode::default();
        }
        
        // Start with 0 opacity
        SetLayeredWindowAttributes(hwnd, 0, 0, LWA_ALPHA);
        
        // Make sure window can receive keyboard input
        SetForegroundWindow(hwnd);
        
        // Start animation timer
        SetTimer(hwnd, TIMER_ID, TIMER_INTERVAL, None);
        
        // Message loop
        let mut msg: MSG = std::mem::zeroed();
        loop {
            if animation().phase == Phase::Done {
                break;
            }
            
            let result = GetMessageW(&mut msg, hwnd, 0, 0);
            if result == 0 || result == -1 {
                break;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        
        // Cleanup
        let selected = animation().selected_mode();
        KillTimer(hwnd, TIMER_ID);
        DestroyWindow(hwnd);
        UnregisterClassW(class_name.as_ptr(), hinstance);
        
        selected
    }
}

unsafe extern "system" fn splash_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TIMER => {
            if wparam == TIMER_ID {
                update_animation(hwnd);
            }
            0
        }
        WM_KEYDOWN => {
            if animation().phase == Phase::Interactive {
                match wparam {
                    VK_UP => {
                        animation().move_selection(-1);
                        InvalidateRect(hwnd, null_mut(), 1);
                    }
                    VK_DOWN => {
                        animation().move_selection(1);
                        InvalidateRect(hwnd, null_mut(), 1);
                    }
                    VK_RETURN => {
                        animation().confirm();
                    }
                    VK_ESCAPE => {
                        // Exit without selecting (use default)
                        animation().confirmed = true;
                        animation().phase = Phase::FadeOut;
                    }
                    _ => {}
                }
            }
            0
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);
            paint_splash(hwnd, hdc);
            EndPaint(hwnd, &ps);
            0
        }
        WM_ERASEBKGND => 1,
        WM_DESTROY => 0,
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn update_animation(hwnd: HWND) {
    match animation().phase {
        Phase::FadeIn => {
            if animation().opacity < 255 {
                animation().opacity = (animation().opacity as u16 + 12).min(255) as u8;
                SetLayeredWindowAttributes(hwnd, 0, animation().opacity, LWA_ALPHA);
                InvalidateRect(hwnd, null_mut(), 1);
            } else {
                animation().phase = Phase::Interactive;
            }
        }
        Phase::Interactive => {
            // Just wait for user input
        }
        Phase::FadeOut => {
            if animation().opacity > 0 {
                animation().opacity = animation().opacity.saturating_sub(15);
                SetLayeredWindowAttributes(hwnd, 0, animation().opacity, LWA_ALPHA);
            } else {
                animation().phase = Phase::Done;
                PostMessageW(hwnd, WM_CLOSE, 0, 0);
            }
        }
        Phase::Done => {}
    }
}

unsafe fn paint_splash(_hwnd: HWND, hdc: HDC) {
    let screen_width = GetSystemMetrics(SM_CXSCREEN);
    let screen_height = GetSystemMetrics(SM_CYSCREEN);
    
    // Calculate scale factor based on screen height
    // Base design is for 1080p, scale proportionally
    // For 1080p: scale = 1.0, for 1440p: scale = 1.33, for 4K: scale = 2.0
    let scale = screen_height as f32 / 1080.0;
    
    // Helper to scale sizes
    let s = |size: i32| -> i32 { (size as f32 * scale) as i32 };
    
    let rect = RECT {
        left: 0,
        top: 0,
        right: screen_width,
        bottom: screen_height,
    };
    
    // Dark background
    let bg_brush = CreateSolidBrush(0x00121212);
    FillRect(hdc, &rect, bg_brush);
    DeleteObject(bg_brush as *mut _);
    
    SetBkMode(hdc, TRANSPARENT);
    
    let font_name = to_wstring("Segoe UI");
    
    // Font sizes scaled to screen resolution
    // Base sizes designed for 1080p, will scale up for higher res
    let title_size = s(96);      // Large title
    let subtitle_size = s(32);   // Subtitle
    let mode_size = s(42);       // Mode names
    let desc_size = s(24);       // Descriptions
    let hint_size = s(20);       // Navigation hints
    
    // Title font: thin weight for elegance
    let title_font = CreateFontW(
        title_size, 0, 0, 0, FW_THIN as i32, 0, 0, 0,
        DEFAULT_CHARSET, OUT_TT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_NATURAL_QUALITY,
        DEFAULT_PITCH | FF_DONTCARE, font_name.as_ptr(),
    );
    
    // Subtitle font: light weight
    let subtitle_font = CreateFontW(
        subtitle_size, 0, 0, 0, FW_LIGHT as i32, 0, 0, 0,
        DEFAULT_CHARSET, OUT_TT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_NATURAL_QUALITY,
        DEFAULT_PITCH | FF_DONTCARE, font_name.as_ptr(),
    );
    
    // Mode names: semibold for emphasis
    let mode_font = CreateFontW(
        mode_size, 0, 0, 0, FW_SEMIBOLD as i32, 0, 0, 0,
        DEFAULT_CHARSET, OUT_TT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_NATURAL_QUALITY,
        DEFAULT_PITCH | FF_DONTCARE, font_name.as_ptr(),
    );
    
    // Description: normal weight
    let desc_font = CreateFontW(
        desc_size, 0, 0, 0, FW_NORMAL as i32, 0, 0, 0,
        DEFAULT_CHARSET, OUT_TT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_NATURAL_QUALITY,
        DEFAULT_PITCH | FF_DONTCARE, font_name.as_ptr(),
    );
    
    // Hints: normal weight
    let hint_font = CreateFontW(
        hint_size, 0, 0, 0, FW_NORMAL as i32, 0, 0, 0,
        DEFAULT_CHARSET, OUT_TT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_NATURAL_QUALITY,
        DEFAULT_PITCH | FF_DONTCARE, font_name.as_ptr(),
    );
    
    let center_x = screen_width / 2;
    let center_y = screen_height / 2;
    
    // Scaled layout positions
    let title_top = center_y - s(220);
    let subtitle_top = center_y - s(110);
    let mode_start_y = center_y - s(20);
    let mode_height = s(90);
    let mode_desc_offset = s(45);
    let hint_top = center_y + s(260);
    let box_half_width = s(250);
    
    let old_font = SelectObject(hdc, title_font as *mut _);
    
    // Title
    SetTextColor(hdc, 0x00FFFFFF);
    let title = to_wstring("Window Manager");
    let mut title_rect = RECT {
        left: 0,
        top: title_top,
        right: screen_width,
        bottom: title_top + s(100),
    };
    DrawTextW(hdc, title.as_ptr(), -1, &mut title_rect, DT_CENTER | DT_SINGLELINE);
    
    // Subtitle
    SelectObject(hdc, subtitle_font as *mut _);
    SetTextColor(hdc, 0x00808080);
    let subtitle = to_wstring("Select Mode");
    let mut subtitle_rect = RECT {
        left: 0,
        top: subtitle_top,
        right: screen_width,
        bottom: subtitle_top + s(40),
    };
    DrawTextW(hdc, subtitle.as_ptr(), -1, &mut subtitle_rect, DT_CENTER | DT_SINGLELINE);
    
    // Mode list
    let modes = WindowMode::all();
    
    for (i, mode) in modes.iter().enumerate() {
        let y = mode_start_y + (i as i32 * mode_height);
        let is_selected = i == animation().selected_index;
        let is_available = mode.is_available();
        
        // Selection indicator and mode name
        SelectObject(hdc, mode_font as *mut _);
        
        let (name_color, desc_color) = if is_selected && is_available {
            (0x0000D4FF, 0x00909090) // Bright gold/amber for selected
        } else if is_available {
            (0x00A0A0A0, 0x00606060) // Gray for available but not selected
        } else {
            (0x00505050, 0x00383838) // Dim for unavailable
        };
        
        SetTextColor(hdc, name_color);
        
        // Arrow indicator for selected
        let prefix = if is_selected { "▸ " } else { "   " };
        let mode_text = to_wstring(&format!("{}{}", prefix, mode.name()));
        let mut mode_rect = RECT {
            left: center_x - box_half_width,
            top: y,
            right: center_x + box_half_width,
            bottom: y + s(50),
        };
        DrawTextW(hdc, mode_text.as_ptr(), -1, &mut mode_rect, DT_CENTER | DT_SINGLELINE);
        
        // Description
        SelectObject(hdc, desc_font as *mut _);
        SetTextColor(hdc, desc_color);
        
        let desc = if is_available {
            mode.description().to_string()
        } else {
            "Coming soon".to_string()
        };
        let desc_text = to_wstring(&desc);
        let mut desc_rect = RECT {
            left: center_x - box_half_width,
            top: y + mode_desc_offset,
            right: center_x + box_half_width,
            bottom: y + mode_desc_offset + s(30),
        };
        DrawTextW(hdc, desc_text.as_ptr(), -1, &mut desc_rect, DT_CENTER | DT_SINGLELINE);
    }
    
    // Navigation hints at bottom
    if animation().phase == Phase::Interactive {
        SelectObject(hdc, hint_font as *mut _);
        SetTextColor(hdc, 0x00606060);
        let hints = to_wstring("↑↓ Navigate    Enter Confirm    Esc Default");
        let mut hint_rect = RECT {
            left: 0,
            top: hint_top,
            right: screen_width,
            bottom: hint_top + s(30),
        };
        DrawTextW(hdc, hints.as_ptr(), -1, &mut hint_rect, DT_CENTER | DT_SINGLELINE);
    }
    
    // Cleanup
    SelectObject(hdc, old_font);
    DeleteObject(title_font as *mut _);
    DeleteObject(subtitle_font as *mut _);
    DeleteObject(mode_font as *mut _);
    DeleteObject(desc_font as *mut _);
    DeleteObject(hint_font as *mut _);
}

fn to_wstring(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
