//! Window management modes.

/// Available window management modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowMode {
    /// Monocle mode: Single window visible at a time, centered with configurable size.
    /// This is the default and currently only implemented mode.
    #[default]
    Monocle,
    
    /// Tiling mode: Windows arranged in a grid layout.
    /// Not yet implemented.
    Tiling,
    
    /// Floating mode: Traditional overlapping windows.
    /// Not yet implemented.
    Floating,
}

impl WindowMode {
    /// Returns the display name for the mode.
    pub fn name(&self) -> &'static str {
        match self {
            WindowMode::Monocle => "Monocle",
            WindowMode::Tiling => "Tiling",
            WindowMode::Floating => "Floating",
        }
    }
    
    /// Returns a short description of the mode.
    pub fn description(&self) -> &'static str {
        match self {
            WindowMode::Monocle => "Single window, centered",
            WindowMode::Tiling => "Grid layout",
            WindowMode::Floating => "Traditional overlapping",
        }
    }
    
    /// Returns whether this mode is currently implemented.
    pub fn is_available(&self) -> bool {
        matches!(self, WindowMode::Monocle)
    }
    
    /// Returns all modes in display order.
    pub fn all() -> &'static [WindowMode] {
        &[WindowMode::Monocle, WindowMode::Tiling, WindowMode::Floating]
    }
}
