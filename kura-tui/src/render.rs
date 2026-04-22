use crate::app::{App, Focus};
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{self, Color, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use kura_ghostty::SyncedOutput;
use std::io::Write;

pub struct Renderer<'a, W: Write> {
    stdout: &'a mut W,
    width: u16,
    height: u16,
}

impl<'a, W: Write> Renderer<'a, W> {
    pub fn new(stdout: &'a mut W, width: u16, height: u16) -> Self {
        Self {
            stdout,
            width,
            height,
        }
    }

    pub fn render(&mut self, app: &App) -> anyhow::Result<()> {
        if app.capabilities.synced_output {
            let _ = SyncedOutput::begin_on_stdout();
        }

        self.clear_screen(app.theme.bg)?;

        let main_height = if self.height > 3 {
            self.height - 2
        } else {
            self.height
        };

        self.render_conversation_pane(app, 0, 0, self.width, main_height)?;
        self.render_input_bar(app, 0, main_height, self.width)?;
        self.render_status_bar(app, 0, main_height + 1, self.width)?;

        if app.focus == Focus::ToolApproval {
            self.render_tool_approval(app)?;
        }

        self.stdout.flush()?;

        if app.capabilities.synced_output {
            let _ = SyncedOutput::end_on_stdout();
        }

        Ok(())
    }

    fn clear_screen(&mut self, bg: Color) -> anyhow::Result<()> {
        queue!(self.stdout, SetBackgroundColor(bg), Clear(ClearType::All))?;
        Ok(())
    }

    fn render_conversation_pane(
        &mut self,
        app: &App,
        x: u16,
        y: u16,
        w: u16,
        h: u16,
    ) -> anyhow::Result<()> {
        let border_color = if app.focus == Focus::Conversation {
            app.theme.border_focused
        } else {
            app.theme.border
        };

        queue!(self.stdout, SetForegroundColor(border_color), MoveTo(x, y),)?;

        let title = format!(
            " {} @ {} [{}] ",
            app.agent_name, app.provider_name, app.model_name
        );
        self.write_line(&title, app.theme.fg, app.theme.bg, w)?;

        for i in 1..h {
            queue!(self.stdout, MoveTo(x, y + i))?;
            self.write_line("", app.theme.fg, app.theme.bg, w)?;
        }

        Ok(())
    }

    fn render_input_bar(&mut self, app: &App, x: u16, y: u16, w: u16) -> anyhow::Result<()> {
        let (bg, fg) = if app.focus == Focus::Input {
            (app.theme.input_bg, app.theme.input_fg)
        } else {
            (app.theme.dim, app.theme.muted)
        };

        queue!(
            self.stdout,
            SetBackgroundColor(bg),
            SetForegroundColor(fg),
            MoveTo(x, y)
        )?;

        let prompt = "> ";
        let input_text = if app.input_buffer.is_empty() {
            "type your message...".to_string()
        } else {
            app.input_buffer.clone()
        };

        let line = format!("{}{}", prompt, input_text);
        self.write_line(&line, fg, bg, w)?;

        if app.focus == Focus::Input {
            let cursor_pos = (prompt.len() + app.input_cursor) as u16;
            if cursor_pos < w {
                queue!(self.stdout, MoveTo(x + cursor_pos, y))?;
            }
        }

        Ok(())
    }

    fn render_status_bar(&mut self, app: &App, x: u16, y: u16, w: u16) -> anyhow::Result<()> {
        queue!(
            self.stdout,
            SetBackgroundColor(app.theme.status_bg),
            SetForegroundColor(app.theme.status_fg),
            MoveTo(x, y),
        )?;

        let ghostty_indicator = if app.capabilities.is_ghostty {
            " G"
        } else {
            ""
        };
        let thinking_indicator = if app.show_thinking { " T" } else { "" };
        let tool_indicator = if app.show_tool_output { " O" } else { "" };

        let left = format!(
            " kura {}:{} turn:{}{}{}{}",
            app.provider_name,
            app.model_name,
            app.turn_count,
            ghostty_indicator,
            thinking_indicator,
            tool_indicator,
        );

        let right = app.status_message.as_deref().unwrap_or("");

        let status = format!(
            "{:<width$}",
            format!("{}{}", left, right),
            width = w as usize
        );
        self.write_line(&status, app.theme.status_fg, app.theme.status_bg, w)?;

        Ok(())
    }

    fn render_tool_approval(&mut self, app: &App) -> anyhow::Result<()> {
        if let Some(tool) = &app.pending_tool {
            let y = self.height / 2;
            let w = (self.width as usize).min(60);
            let x = (self.width as usize - w) / 2;

            queue!(
                self.stdout,
                SetBackgroundColor(app.theme.warn),
                SetForegroundColor(app.theme.bg),
                MoveTo(x as u16, y),
            )?;

            let line1 = format!(" Tool: {} ", tool.name);
            self.write_line(&line1, app.theme.bg, app.theme.warn, w as u16)?;

            queue!(self.stdout, MoveTo(x as u16, y + 1))?;
            let input_str = serde_json::to_string(&tool.input).unwrap_or_default();
            let truncated = &input_str[..input_str.len().min(w as usize - 4)];
            let line2 = format!(" {} ", truncated);
            self.write_line(&line2, app.theme.bg, app.theme.warn, w as u16)?;

            queue!(self.stdout, MoveTo(x as u16, y + 2))?;
            let line3 = " [y] approve  [n] deny ";
            self.write_line(line3, app.theme.bg, app.theme.warn, w as u16)?;
        }
        Ok(())
    }

    fn write_line(&mut self, text: &str, fg: Color, bg: Color, width: u16) -> anyhow::Result<()> {
        let padded = format!("{:<width$}", text, width = width as usize);
        let truncated: String = padded.chars().take(width as usize).collect();
        queue!(
            self.stdout,
            SetForegroundColor(fg),
            SetBackgroundColor(bg),
            style::Print(&truncated),
        )?;
        Ok(())
    }
}
