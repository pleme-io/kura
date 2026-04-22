use crate::theme::KuraTheme;
use crossterm::style::Color;

pub struct ConversationPane;

impl ConversationPane {
    pub fn format_thinking(text: &str, theme: &KuraTheme) -> Vec<(String, Color, Color)> {
        text.lines()
            .map(|line| (format!("  │ {}", line), theme.thinking_fg, theme.bg))
            .collect()
    }

    pub fn format_tool_call(name: &str, theme: &KuraTheme) -> (String, Color, Color) {
        (format!("  ⚡ {}", name), theme.accent, theme.bg)
    }

    pub fn format_tool_result(
        output: &str,
        is_error: bool,
        theme: &KuraTheme,
    ) -> Vec<(String, Color, Color)> {
        let color = if is_error { theme.error } else { theme.success };
        output
            .lines()
            .take(20)
            .map(|line| (format!("  │ {}", line), color, theme.bg))
            .collect()
    }

    pub fn format_user_message(text: &str, theme: &KuraTheme) -> Vec<(String, Color, Color)> {
        text.lines()
            .map(|line| (format!("  ▸ {}", line), theme.fg, theme.bg))
            .collect()
    }

    pub fn format_assistant_text(text: &str, theme: &KuraTheme) -> Vec<(String, Color, Color)> {
        text.lines()
            .map(|line| (format!("  {}", line), theme.fg, theme.bg))
            .collect()
    }
}
