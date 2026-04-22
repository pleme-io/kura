pub mod detect;
pub mod kitty_graphics;
pub mod kitty_keyboard;
pub mod synced_output;
pub mod terminal;

pub use detect::GhosttyDetect;
pub use kitty_graphics::{ImageFormat, ImageOptions, KittyGraphics, Placement};
pub use kitty_keyboard::{KeyboardMode, KittyKeyboard};
pub use synced_output::SyncedOutput;
pub use terminal::{TerminalGuard, TerminalRestoreGuard};

#[derive(Debug, Clone)]
pub struct GhosttyCapabilities {
    pub is_ghostty: bool,
    pub kitty_graphics: bool,
    pub kitty_keyboard: bool,
    pub synced_output: bool,
    pub osc_8_hyperlinks: bool,
    pub osc_52_clipboard: bool,
    pub display_p3: bool,
}

impl GhosttyCapabilities {
    pub fn detect() -> Self {
        let is_ghostty = GhosttyDetect::is_ghostty();
        Self {
            is_ghostty,
            kitty_graphics: is_ghostty,
            kitty_keyboard: is_ghostty,
            synced_output: is_ghostty,
            osc_8_hyperlinks: is_ghostty,
            osc_52_clipboard: is_ghostty,
            display_p3: GhosttyDetect::supports_display_p3(),
        }
    }
}

#[cfg(test)]
mod tests;
