use crossterm::style::Color;

#[derive(Debug, Clone)]
pub struct KuraTheme {
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub muted: Color,
    pub dim: Color,
    pub success: Color,
    pub warn: Color,
    pub error: Color,
    pub border: Color,
    pub border_focused: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub input_bg: Color,
    pub input_fg: Color,
    pub status_bg: Color,
    pub status_fg: Color,
    pub tool_bg: Color,
    pub thinking_fg: Color,
    pub sigil_check: Color,
    pub sigil_cross: Color,
    pub sigil_arrow: Color,
}

impl KuraTheme {
    pub fn nord() -> Self {
        Self {
            bg: Color::Rgb {
                r: 46,
                g: 52,
                b: 64,
            },
            fg: Color::Rgb {
                r: 216,
                g: 222,
                b: 233,
            },
            accent: Color::Rgb {
                r: 136,
                g: 192,
                b: 208,
            },
            muted: Color::Rgb {
                r: 76,
                g: 86,
                b: 106,
            },
            dim: Color::Rgb {
                r: 59,
                g: 66,
                b: 82,
            },
            success: Color::Rgb {
                r: 163,
                g: 190,
                b: 140,
            },
            warn: Color::Rgb {
                r: 235,
                g: 203,
                b: 139,
            },
            error: Color::Rgb {
                r: 191,
                g: 97,
                b: 106,
            },
            border: Color::Rgb {
                r: 76,
                g: 86,
                b: 106,
            },
            border_focused: Color::Rgb {
                r: 136,
                g: 192,
                b: 208,
            },
            selection_bg: Color::Rgb {
                r: 67,
                g: 76,
                b: 94,
            },
            selection_fg: Color::Rgb {
                r: 216,
                g: 222,
                b: 233,
            },
            input_bg: Color::Rgb {
                r: 59,
                g: 66,
                b: 82,
            },
            input_fg: Color::Rgb {
                r: 216,
                g: 222,
                b: 233,
            },
            status_bg: Color::Rgb {
                r: 59,
                g: 66,
                b: 82,
            },
            status_fg: Color::Rgb {
                r: 136,
                g: 192,
                b: 208,
            },
            tool_bg: Color::Rgb {
                r: 46,
                g: 52,
                b: 64,
            },
            thinking_fg: Color::Rgb {
                r: 180,
                g: 142,
                b: 173,
            },
            sigil_check: Color::Rgb {
                r: 163,
                g: 190,
                b: 140,
            },
            sigil_cross: Color::Rgb {
                r: 191,
                g: 97,
                b: 106,
            },
            sigil_arrow: Color::Rgb {
                r: 136,
                g: 192,
                b: 208,
            },
        }
    }
}
