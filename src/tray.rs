//! System tray icon with menu.

use std::ptr::null_mut;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::*;
use windows_sys::Win32::UI::Shell::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

use crate::hooks::with_manager;

const TRAY_ICON_ID: u32 = 1;
const WM_TRAYICON: u32 = WM_USER + 1;

const IDM_TOGGLE: u32 = 1001;
const IDM_EXIT: u32 = 1002;

// NOTIFYICONDATA version for Windows Vista+
const NOTIFYICON_VERSION_4: u32 = 4;

static mut TRAY_HWND: HWND = null_mut();
static mut NOTIFY_ICON: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };

const TRAY_CLASS: &str = "WindowManagerTray";

pub fn create_tray_icon() -> bool {
    unsafe {
        let hinstance = GetModuleHandleW(null_mut());
        
        // Register window class for tray message handling
        let class_name = to_wstring(TRAY_CLASS);
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(tray_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: null_mut(),
            lpszMenuName: null_mut(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: null_mut(),
        };
        RegisterClassExW(&wc);
        
        // Create hidden window for tray messages
        TRAY_HWND = CreateWindowExW(
            0,
            class_name.as_ptr(),
            null_mut(),
            0,
            0, 0, 0, 0,
            HWND_MESSAGE,  // Message-only window
            null_mut(),
            hinstance,
            null_mut(),
        );
        
        if TRAY_HWND.is_null() {
            log::error!("Failed to create tray window");
            return false;
        }
        
        // Create tray icon
        let tooltip = to_wstring_fixed::<128>("Window Manager");
        
        NOTIFY_ICON = std::mem::zeroed();
        NOTIFY_ICON.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        NOTIFY_ICON.hWnd = TRAY_HWND;
        NOTIFY_ICON.uID = TRAY_ICON_ID;
        NOTIFY_ICON.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP | NIF_SHOWTIP;
        NOTIFY_ICON.uCallbackMessage = WM_TRAYICON;
        NOTIFY_ICON.hIcon = LoadIconW(null_mut(), IDI_APPLICATION);
        NOTIFY_ICON.szTip = tooltip;
        NOTIFY_ICON.Anonymous.uVersion = NOTIFYICON_VERSION_4;
        
        if Shell_NotifyIconW(NIM_ADD, std::ptr::addr_of!(NOTIFY_ICON)) == 0 {
            log::error!("Failed to add tray icon");
            return false;
        }
        
        // Set version for modern behavior
        Shell_NotifyIconW(NIM_SETVERSION, std::ptr::addr_of!(NOTIFY_ICON));
        
        log::info!("Tray icon created");
        true
    }
}

pub fn remove_tray_icon() {
    unsafe {
        if !TRAY_HWND.is_null() {
            Shell_NotifyIconW(NIM_DELETE, std::ptr::addr_of!(NOTIFY_ICON));
            DestroyWindow(TRAY_HWND);
            TRAY_HWND = null_mut();
            log::info!("Tray icon removed");
        }
    }
}

pub fn update_tray_tooltip(enabled: bool) {
    unsafe {
        if TRAY_HWND.is_null() { return; }
        
        let text = if enabled {
            "Window Manager - Running"
        } else {
            "Window Manager - Disabled"
        };
        
        NOTIFY_ICON.szTip = to_wstring_fixed::<128>(text);
        NOTIFY_ICON.uFlags = NIF_TIP;
        Shell_NotifyIconW(NIM_MODIFY, std::ptr::addr_of!(NOTIFY_ICON));
    }
}

unsafe extern "system" fn tray_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TRAYICON => {
            let event = (lparam & 0xFFFF) as u32;
            match event {
                WM_RBUTTONUP => {
                    show_context_menu(hwnd);
                }
                WM_LBUTTONDBLCLK => {
                    // Double-click toggles
                    toggle_manager();
                }
                _ => {}
            }
            0
        }
        WM_COMMAND => {
            let cmd = (wparam & 0xFFFF) as u32;
            match cmd {
                IDM_TOGGLE => {
                    toggle_manager();
                }
                IDM_EXIT => {
                    // Post quit message to main loop
                    PostMessageW(hwnd, WM_CLOSE, 0, 0);
                    // Also signal main app to exit
                    std::process::exit(0);
                }
                _ => {}
            }
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn show_context_menu(hwnd: HWND) {
    let menu = CreatePopupMenu();
    if menu.is_null() { return; }
    
    // Check if enabled
    let enabled = with_manager(|m| m.is_enabled()).unwrap_or(true);
    
    // Toggle item
    let toggle_text = if enabled {
        to_wstring("Disable")
    } else {
        to_wstring("Enable")
    };
    AppendMenuW(menu, MF_STRING, IDM_TOGGLE as usize, toggle_text.as_ptr());
    
    // Separator
    AppendMenuW(menu, MF_SEPARATOR, 0, null_mut());
    
    // Exit item
    let exit_text = to_wstring("Exit");
    AppendMenuW(menu, MF_STRING, IDM_EXIT as usize, exit_text.as_ptr());
    
    // Get cursor position
    let mut pt = POINT { x: 0, y: 0 };
    GetCursorPos(&mut pt);
    
    // Show menu
    SetForegroundWindow(hwnd);
    TrackPopupMenu(
        menu,
        TPM_RIGHTALIGN | TPM_BOTTOMALIGN | TPM_RIGHTBUTTON,
        pt.x, pt.y,
        0,
        hwnd,
        null_mut(),
    );
    PostMessageW(hwnd, WM_NULL, 0, 0);
    
    DestroyMenu(menu);
}

fn toggle_manager() {
    let enabled = with_manager(|m| {
        m.toggle_enabled();
        m.is_enabled()
    }).unwrap_or(true);
    
    update_tray_tooltip(enabled);
}

fn to_wstring(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn to_wstring_fixed<const N: usize>(s: &str) -> [u16; N] {
    let mut arr = [0u16; N];
    for (i, c) in s.encode_utf16().take(N - 1).enumerate() {
        arr[i] = c;
    }
    arr
}

#[repr(C)]
struct POINT {
    x: i32,
    y: i32,
}

#[link(name = "user32")]
extern "system" {
    fn GetCursorPos(lpPoint: *mut POINT) -> i32;
}
