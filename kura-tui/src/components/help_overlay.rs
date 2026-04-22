use crate::theme::KuraTheme;

pub struct HelpOverlay;

impl HelpOverlay {
    pub fn format_lines(
        theme: &KuraTheme,
    ) -> Vec<(String, crossterm::style::Color, crossterm::style::Color)> {
        let lines = vec![
            " kura — help ",
            "",
            " i/Tab     — focus input",
            " Enter     — submit input",
            " Esc       — unfocus / cancel",
            " j/k/↑/↓   — scroll conversation",
            " t         — toggle thinking display",
            " o         — toggle tool output display",
            " n         — new session",
            " Ctrl+Up   — cycle provider",
            " q         — quit",
            "",
            " Ghostty-native: Kitty Graphics, Kitty Keyboard, Synced Output",
            " Config: shikumi (YAML/TOML/Lisp/Nix) + tatara-lisp (defprovider, defagent, defkeymap)",
        ];

        lines
            .into_iter()
            .map(|l| (l.to_string(), theme.fg, theme.bg))
            .collect()
    }
}
