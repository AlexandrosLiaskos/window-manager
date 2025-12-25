//! Configuration management for the window manager.

use serde::Deserialize;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] toml::de::Error),
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub hotkeys: HotkeyConfig,
    #[serde(default)]
    pub exclusions: ExclusionConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GeneralConfig {
    /// Window size as percentage of screen (10-100). Default 90.
    #[serde(default = "default_window_size")]
    pub window_size: u8,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HotkeyConfig {
    #[serde(default = "default_toggle")]
    pub toggle_enabled: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExclusionConfig {
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default)]
    pub titles: Vec<String>,
    // Note: process exclusion not yet implemented
}

fn default_window_size() -> u8 { 95 }
fn default_enabled() -> bool { true }
fn default_toggle() -> String { "Alt+Shift+M".to_string() }

impl Default for GeneralConfig {
    fn default() -> Self {
        Self { window_size: default_window_size(), enabled: default_enabled() }
    }
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            toggle_enabled: default_toggle(),
        }
    }
}

impl Default for ExclusionConfig {
    fn default() -> Self {
        Self {
            classes: vec![
                "Shell_TrayWnd".into(), 
                "Shell_SecondaryTrayWnd".into(), 
                "Progman".into(), 
                "WorkerW".into(),
                "ConsoleWindowClass".into(),  // Console windows (when run from terminal)
            ],
            titles: vec!["Program Manager".into()],
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self { general: GeneralConfig::default(), hotkeys: HotkeyConfig::default(), exclusions: ExclusionConfig::default() }
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config.validate())
    }

    pub fn load_or_default() -> Self {
        for path in ["config.toml", "wm.toml"] {
            if Path::new(path).exists() {
                if let Ok(config) = Self::load(path) {
                    log::info!("Loaded config from {}", path);
                    return config;
                }
            }
        }
        log::info!("Using default configuration");
        Self::default()
    }

    fn validate(mut self) -> Self {
        // Clamp window_size between 10 and 100
        if self.general.window_size < 10 {
            self.general.window_size = 10;
        } else if self.general.window_size > 100 {
            self.general.window_size = 100;
        }
        self
    }

    pub fn is_class_excluded(&self, class: &str) -> bool {
        self.exclusions.classes.iter().any(|c| c == class)
    }

    pub fn is_title_excluded(&self, title: &str) -> bool {
        self.exclusions.titles.iter().any(|t| title.contains(t.as_str()))
    }
}

pub fn parse_hotkey(hotkey: &str) -> Option<(u32, u32)> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

    let parts: Vec<&str> = hotkey.split('+').collect();
    if parts.is_empty() { return None; }

    let mut modifiers: u32 = 0;
    let key_str = parts.last()?;

    for part in &parts[..parts.len() - 1] {
        match part.to_lowercase().as_str() {
            "alt" => modifiers |= MOD_ALT,
            "ctrl" | "control" => modifiers |= MOD_CONTROL,
            "shift" => modifiers |= MOD_SHIFT,
            "win" | "super" => modifiers |= MOD_WIN,
            _ => {}
        }
    }

    let vk: u32 = match key_str.to_uppercase().as_str() {
        "A" => 0x41, "B" => 0x42, "C" => 0x43, "D" => 0x44, "E" => 0x45,
        "F" => 0x46, "G" => 0x47, "H" => 0x48, "I" => 0x49, "J" => 0x4A,
        "K" => 0x4B, "L" => 0x4C, "M" => 0x4D, "N" => 0x4E, "O" => 0x4F,
        "P" => 0x50, "Q" => 0x51, "R" => 0x52, "S" => 0x53, "T" => 0x54,
        "U" => 0x55, "V" => 0x56, "W" => 0x57, "X" => 0x58, "Y" => 0x59, "Z" => 0x5A,
        "0" => 0x30, "1" => 0x31, "2" => 0x32, "3" => 0x33, "4" => 0x34,
        "5" => 0x35, "6" => 0x36, "7" => 0x37, "8" => 0x38, "9" => 0x39,
        "TAB" => VK_TAB as u32, "SPACE" => VK_SPACE as u32,
        "RETURN" | "ENTER" => VK_RETURN as u32, "ESCAPE" | "ESC" => VK_ESCAPE as u32,
        "LEFT" => VK_LEFT as u32, "RIGHT" => VK_RIGHT as u32,
        "UP" => VK_UP as u32, "DOWN" => VK_DOWN as u32,
        _ => return None,
    };

    Some((modifiers, vk))
}
