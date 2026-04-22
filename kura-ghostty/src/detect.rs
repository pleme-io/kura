use std::env;

pub struct GhosttyDetect;

impl GhosttyDetect {
    pub fn is_ghostty() -> bool {
        env::var("TERM").is_ok_and(|t| t == "xterm-ghostty")
            || env::var("GHOSTTY_RESOURCES_DIR").is_ok()
    }

    pub fn term_var() -> Option<String> {
        env::var("TERM").ok()
    }

    pub fn supports_display_p3() -> bool {
        Self::is_ghostty()
            && env::var("COLORTERM").is_ok_and(|ct| ct == "truecolor" || ct == "24bit")
    }

    pub fn ghostty_version() -> Option<String> {
        env::var("GHOSTTY_VERSION").ok()
    }

    pub fn query_kitty_graphics_support() -> bool {
        if !Self::is_ghostty() {
            return false;
        }
        true
    }
}
