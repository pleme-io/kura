use crate::theme::KuraTheme;

pub struct StatusBar;

impl StatusBar {
    pub fn format(
        provider: &str,
        model: &str,
        turn: usize,
        is_ghostty: bool,
        show_thinking: bool,
        show_tool_output: bool,
        status_msg: Option<&str>,
        _theme: &KuraTheme,
    ) -> String {
        let ghostty = if is_ghostty { " G" } else { "" };
        let thinking = if show_thinking { " T" } else { "" };
        let tool_out = if show_tool_output { " O" } else { "" };
        let msg = status_msg.unwrap_or("");

        format!(
            " kura {}:{} turn:{}{}{}{} │ {}",
            provider, model, turn, ghostty, thinking, tool_out, msg
        )
    }
}
