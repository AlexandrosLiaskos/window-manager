//! Window Manager - A lightweight window manager for Windows 11
//! 
//! Supports multiple window management modes selected at startup.

// Hide console window - run as a Windows GUI application
#![windows_subsystem = "windows"]

mod config;
mod hooks;
mod hotkeys;
mod manager;
mod mode;
mod positioning;
mod splash;
mod tray;
mod window;

use std::ptr::null_mut;
use std::process;
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
use windows_sys::Win32::UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, TranslateMessage, MSG, WM_HOTKEY};

use config::Config;
use hooks::{init_manager, install_hooks, remove_hooks, with_manager};
use hotkeys::{handle_hotkey, register_hotkeys, unregister_hotkeys};
use manager::WindowManager;

const VERSION: &str = "0.3.0";

// External Windows API declarations
#[link(name = "kernel32")]
extern "system" {
    fn CreateMutexW(lpMutexAttributes: *mut std::ffi::c_void, bInitialOwner: i32, lpName: *const u16) -> HANDLE;
}

#[link(name = "user32")]
extern "system" {
    fn SetProcessDPIAware() -> i32;
}

/// Enable DPI awareness for crisp text rendering on high-DPI displays.
fn enable_dpi_awareness() {
    unsafe {
        SetProcessDPIAware();
    }
}

fn main() {
    // Enable DPI awareness FIRST, before any window creation
    enable_dpi_awareness();
    
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();
    
    log::info!("Window Manager v{} starting...", VERSION);

    // Check single instance
    if is_already_running() { 
        log::error!("Another instance is already running!"); 
        process::exit(1); 
    }

    // Show splash screen with mode selection
    let selected_mode = splash::show_splash();
    log::info!("Selected mode: {:?}", selected_mode);

    // Load config
    let config = Config::load_or_default();
    log::info!("Window size: {}%", config.general.window_size);

    // Set up hooks
    if let Err(e) = install_hooks() { 
        log::error!("Failed to install hooks: {}", e); 
        process::exit(1); 
    }
    
    // Register hotkeys
    if let Err(e) = register_hotkeys(&config) { 
        log::warn!("Failed to register some hotkeys: {}", e); 
    }

    // Initialize window manager with selected mode
    let mut manager = WindowManager::new(config.clone(), selected_mode);
    manager.initialize();
    log::info!("Mode: {:?}, Managing {} windows", selected_mode, manager.window_count());
    init_manager(manager);

    // Create system tray icon
    if !tray::create_tray_icon() {
        log::error!("Failed to create tray icon");
    }
    
    log::info!("Window Manager running");

    // Main message loop
    run_message_loop();
    cleanup();
    
    log::info!("Window Manager stopped");
}

fn is_already_running() -> bool {
    unsafe {
        // Use a unique mutex name for Window Manager
        let mutex_name: Vec<u16> = "Global\\WindowManager_SingleInstance\0".encode_utf16().collect();
        let handle = CreateMutexW(null_mut(), 1, mutex_name.as_ptr());
        if !handle.is_null() && GetLastError() == ERROR_ALREADY_EXISTS { 
            CloseHandle(handle); 
            return true; 
        }
        false
    }
}



fn run_message_loop() {
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        loop {
            let result = GetMessageW(&mut msg, null_mut(), 0, 0);
            if result == 0 || result == -1 { break; }
            match msg.message {
                WM_HOTKEY => { handle_hotkey(msg.wParam); }
                _ => { TranslateMessage(&msg); DispatchMessageW(&msg); }
            }
        }
    }
}

fn cleanup() {
    // Remove tray icon
    tray::remove_tray_icon();
    
    // Restore all windows before exiting
    with_manager(|m| { 
        m.restore_all();
    });
    remove_hooks();
    unregister_hotkeys();
    log::info!("Cleanup complete");
}


