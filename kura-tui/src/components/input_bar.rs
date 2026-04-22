use crate::theme::KuraTheme;

pub struct InputBar;

impl InputBar {
    pub fn format(
        prompt: &str,
        text: &str,
        focused: bool,
        theme: &KuraTheme,
    ) -> (String, crossterm::style::Color, crossterm::style::Color) {
        let (fg, bg) = if focused {
            (theme.input_fg, theme.input_bg)
        } else {
            (theme.muted, theme.dim)
        };
        (format!("{}{}", prompt, text), fg, bg)
    }
}
