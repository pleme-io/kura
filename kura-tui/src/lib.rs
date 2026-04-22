pub mod app;
pub mod components;
pub mod event;
pub mod layout;
pub mod pane;
pub mod render;
pub mod theme;

pub use app::App;
pub use event::{TuiEvent, TuiEventStream};
pub use theme::KuraTheme;

use kura_ghostty::GhosttyCapabilities;

pub fn detect_capabilities() -> GhosttyCapabilities {
    GhosttyCapabilities::detect()
}

#[cfg(test)]
mod tests;
