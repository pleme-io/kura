use crate::theme::KuraTheme;

pub struct ToolApprovalPane;

impl ToolApprovalPane {
    pub fn format_lines(
        name: &str,
        input: &serde_json::Value,
        theme: &KuraTheme,
    ) -> Vec<(String, crossterm::style::Color, crossterm::style::Color)> {
        let input_str = serde_json::to_string(input).unwrap_or_default();
        vec![
            (format!(" Tool: {} ", name), theme.bg, theme.warn),
            (
                format!(" {} ", &input_str[..input_str.len().min(200)]),
                theme.bg,
                theme.warn,
            ),
            (" [y] approve  [n] deny ".to_string(), theme.bg, theme.warn),
        ]
    }
}
