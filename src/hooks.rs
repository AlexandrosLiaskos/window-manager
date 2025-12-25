//! Win32 event hooks for window monitoring.

use std::ptr::null_mut;
use std::sync::{Mutex, OnceLock};
use std::sync::atomic::{AtomicPtr, Ordering};
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
use windows_sys::Win32::UI::WindowsAndMessaging::OBJID_WINDOW;

use crate::manager::WindowManager;

static MANAGER: OnceLock<Mutex<WindowManager>> = OnceLock::new();
static HOOK_FOREGROUND: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(null_mut());

const EVENT_SYSTEM_FOREGROUND: u32 = 0x0003;
const WINEVENT_OUTOFCONTEXT: u32 = 0x0000;
const WINEVENT_SKIPOWNPROCESS: u32 = 0x0002;

pub fn init_manager(manager: WindowManager) { 
    let _ = MANAGER.set(Mutex::new(manager)); 
}

pub fn with_manager<F, R>(f: F) -> Option<R> where F: FnOnce(&mut WindowManager) -> R {
    MANAGER.get().and_then(|m| m.lock().ok().map(|mut guard| f(&mut guard)))
}

unsafe extern "system" fn win_event_proc(
    _hook: HWINEVENTHOOK, event: u32, hwnd: HWND, id_object: i32,
    _id_child: i32, _event_thread: u32, _event_time: u32,
) {
    if event != EVENT_SYSTEM_FOREGROUND { return; }
    if id_object != OBJID_WINDOW as i32 || hwnd.is_null() { return; }
    
    with_manager(|m| m.on_focus_change(hwnd));
}

pub fn install_hooks() -> Result<(), &'static str> {
    unsafe {
        let hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND, 
            null_mut(), Some(win_event_proc), 0, 0, 
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS
        );
        
        if hook.is_null() {
            return Err("Failed to install event hook");
        }
        
        HOOK_FOREGROUND.store(hook, Ordering::SeqCst);
        log::info!("Foreground hook installed");
        Ok(())
    }
}

pub fn remove_hooks() {
    unsafe {
        let hook = HOOK_FOREGROUND.swap(null_mut(), Ordering::SeqCst);
        if !hook.is_null() { 
            UnhookWinEvent(hook); 
        }
    }
}
