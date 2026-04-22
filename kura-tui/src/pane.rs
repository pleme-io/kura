use crate::theme::KuraTheme;

pub struct Pane;

impl Pane {
    pub fn border(top: bool, bottom: bool, focused: bool, theme: &KuraTheme, width: u16) -> String {
        let color = if focused {
            theme.border_focused
        } else {
            theme.border
        };
        let _ = color;
        let horiz = "─".repeat(width as usize);
        let mut lines = vec![];
        if top {
            lines.push(format!("╭{}╮", horiz));
        }
        if bottom {
            lines.push(format!("╰{}╯", horiz));
        }
        lines.join("\n")
    }

    pub fn title_border(title: &str, focused: bool, width: u16) -> String {
        let _ = focused;
        let inner = width.saturating_sub(2);
        let title_str = format!(" {} ", title);
        let padding = inner as usize - title_str.len();
        format!(
            "╭{}╮",
            title_str
                .chars()
                .chain(std::iter::repeat('─').take(padding))
                .collect::<String>()
        )
    }
}
