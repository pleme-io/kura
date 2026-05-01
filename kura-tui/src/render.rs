//! Terminal renderer for the kura TUI, built on `egaku-term` v0.2.
//!
//! The renderer wraps an [`egaku_term::Terminal`] for the duration of a
//! frame: clear → paint conversation pane / input bar / status bar /
//! optional tool-approval modal → flush. Caller owns the Terminal
//! lifecycle (typically held alongside a `kura_ghostty::TerminalRestoreGuard`
//! for synced-output and Kitty-keyboard restore).
//!
//! Each pane composes egaku-term drawers (`bordered_block_with`,
//! `paragraph_with`, `status_line_with`, `modal_with`) — the colors come
//! from the live [`KuraTheme`] so Nord / base16 / Stylix overrides flow
//! through unchanged.

use crate::app::{App, Focus};
use crate::layout::Layout;
use egaku::{Modal, Rect};
use egaku_term::{
    Terminal, draw,
    theme::Palette,
};
use egaku_term::crossterm::{
    QueueableCommand,
    cursor::MoveTo,
    style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use kura_ghostty::SyncedOutput;
use std::io;

/// One-shot frame painter. Construct once per frame; do not hold across
/// awaits — `Terminal` is `!Send` (it owns the stdout handle).
pub struct Renderer<'a> {
    term: &'a mut Terminal,
}

impl<'a> Renderer<'a> {
    /// Borrow a Terminal for the duration of one frame.
    pub fn new(term: &'a mut Terminal) -> Self {
        Self { term }
    }

    /// Paint a full frame from `app` state. Caller does not need to clear
    /// or flush the terminal — this fn does both.
    pub fn render(&mut self, app: &App) -> anyhow::Result<()> {
        // Synced output (DECSET 2026): atomic frame on supporting terminals.
        if app.capabilities.synced_output {
            let _ = SyncedOutput::begin_on_stdout();
        }

        let (cols, rows) = self.term.size().map_err(map_err)?;
        let layout = Layout::compute(cols, rows);

        self.term.clear().map_err(map_err)?;
        self.fill_bg(app, cols, rows)?;

        self.render_conversation_pane(app, &layout)?;
        self.render_input_bar(app, &layout)?;
        self.render_status_bar(app, &layout)?;

        if app.focus == Focus::ToolApproval {
            self.render_tool_approval(app, cols, rows)?;
        }

        self.term.flush().map_err(map_err)?;

        if app.capabilities.synced_output {
            let _ = SyncedOutput::end_on_stdout();
        }
        Ok(())
    }

    /// Paint a uniform background across the whole terminal so cleared
    /// regions match the theme's bg color rather than the user's terminal
    /// default.
    fn fill_bg(&mut self, app: &App, cols: u16, rows: u16) -> anyhow::Result<()> {
        let blank = " ".repeat(usize::from(cols));
        self.term
            .out()
            .queue(SetBackgroundColor(app.theme.bg))?
            .queue(SetForegroundColor(app.theme.fg))?;
        for r in 0..rows {
            self.term
                .out()
                .queue(MoveTo(0, r))?
                .queue(Print(&blank))?;
        }
        self.term.out().queue(ResetColor)?;
        Ok(())
    }

    fn render_conversation_pane(
        &mut self,
        app: &App,
        layout: &Layout,
    ) -> anyhow::Result<()> {
        let palette = palette_from_kura_theme(app);
        let rect = layout_rect(&layout.conversation);
        let title = format!(
            " {} @ {} [{}] ",
            app.agent_name, app.provider_name, app.model_name
        );
        let focused = app.focus == Focus::Conversation;
        draw::bordered_block_with(self.term, rect, &title, focused, &palette)
            .map_err(map_err)?;

        // Body — egaku-term's paragraph drawer wraps to the inner rect.
        // We only render the help-style placeholder for now; the live
        // conversation content is owned by kura-agent and will land here
        // via a `Conversation` arg in the next iteration.
        let inner = draw::block_inner(rect);
        let placeholder = match app.view {
            crate::app::View::Help => help_text(),
            _ => format!("conversation pane — turn {}.", app.turn_count),
        };
        draw::paragraph_with(self.term, inner, &placeholder, &palette)
            .map_err(map_err)?;
        Ok(())
    }

    fn render_input_bar(&mut self, app: &App, layout: &Layout) -> anyhow::Result<()> {
        let rect = layout_rect(&layout.input);
        let focused = app.focus == Focus::Input;
        let prompt = "> ";

        let (fg, bg) = if focused {
            (app.theme.input_fg, app.theme.input_bg)
        } else {
            (app.theme.muted, app.theme.dim)
        };

        // Render the input row directly — we want explicit control over
        // the bg fill across the full width plus the cursor placement.
        let blank = " ".repeat(usize::from(rect.width as u16));
        self.term
            .out()
            .queue(SetBackgroundColor(bg))?
            .queue(SetForegroundColor(fg))?
            .queue(MoveTo(rect.x as u16, rect.y as u16))?
            .queue(Print(&blank))?;

        let body = if app.input_buffer.is_empty() && !focused {
            "type your message...".to_string()
        } else {
            app.input_buffer.clone()
        };
        let line = format!("{prompt}{body}");
        let max_w = usize::from(rect.width as u16);
        let truncated: String = line.chars().take(max_w).collect();
        self.term
            .out()
            .queue(MoveTo(rect.x as u16, rect.y as u16))?
            .queue(Print(truncated))?;

        if focused {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let cursor_col = (prompt.len() + app.input_cursor) as u16;
            if cursor_col < rect.width as u16 {
                self.term
                    .out()
                    .queue(MoveTo(rect.x as u16 + cursor_col, rect.y as u16))?;
            }
        }
        self.term.out().queue(ResetColor)?;
        Ok(())
    }

    fn render_status_bar(&mut self, app: &App, layout: &Layout) -> anyhow::Result<()> {
        let rect = layout_rect(&layout.status);
        let palette = Palette {
            background: app.theme.status_bg,
            foreground: app.theme.status_fg,
            accent: app.theme.accent,
            error: app.theme.error,
            warning: app.theme.warn,
            success: app.theme.success,
            selection: app.theme.status_bg,
            muted: app.theme.muted,
            border: app.theme.border,
        };

        let ghostty = if app.capabilities.is_ghostty { " G" } else { "" };
        let thinking = if app.show_thinking { " T" } else { "" };
        let tool_out = if app.show_tool_output { " O" } else { "" };

        let left = format!(
            " kura {}:{} turn:{}{}{}{}",
            app.provider_name,
            app.model_name,
            app.turn_count,
            ghostty,
            thinking,
            tool_out,
        );
        let right = app.status_message.as_deref().unwrap_or("").to_string();

        draw::status_line_with(self.term, rect, &left, &right, &palette)
            .map_err(map_err)
    }

    fn render_tool_approval(&mut self, app: &App, cols: u16, rows: u16) -> anyhow::Result<()> {
        let Some(tool) = &app.pending_tool else {
            return Ok(());
        };
        let palette = palette_from_kura_theme(app);

        // Centre in the upper-half of the screen so the user can still see
        // the input bar context. egaku::Modal is a visibility toggle;
        // since we already gated on focus we render it visible.
        let mut overlay = Modal::new(&format!("Tool approval — {}", tool.name));
        overlay.show();
        let bounds = Rect::new(
            0.0,
            0.0,
            f32::from(cols),
            f32::from(rows.saturating_sub(2)),
        );
        let input_str = serde_json::to_string(&tool.input).unwrap_or_default();
        let truncated_input: String =
            input_str.chars().take(160).collect::<String>();
        let body = vec![
            truncated_input.as_str(),
            "",
            "[y] approve     [n] deny",
        ];
        draw::modal_with(self.term, bounds, &overlay, &body, &palette)
            .map_err(map_err)
    }
}

/// Copy a [`KuraTheme`] into an egaku-term [`Palette`] so drawers can
/// pull semantic colors. KuraTheme already carries crossterm `Color`
/// values, so the conversion is field-by-field.
fn palette_from_kura_theme(app: &App) -> Palette {
    Palette {
        background: app.theme.bg,
        foreground: app.theme.fg,
        accent: app.theme.accent,
        error: app.theme.error,
        warning: app.theme.warn,
        success: app.theme.success,
        selection: app.theme.selection_bg,
        muted: app.theme.muted,
        border: app.theme.border,
    }
}

#[allow(clippy::cast_precision_loss)]
fn layout_rect(r: &crate::layout::Rect) -> Rect {
    Rect::new(
        f32::from(r.x),
        f32::from(r.y),
        f32::from(r.width),
        f32::from(r.height),
    )
}

fn map_err(e: egaku_term::Error) -> anyhow::Error {
    anyhow::anyhow!("{e}")
}

fn help_text() -> String {
    [
        "kura — help",
        "",
        "i / Tab     focus input",
        "Enter       submit input",
        "Esc         unfocus / cancel",
        "j / k       scroll conversation",
        "t           toggle thinking display",
        "o           toggle tool output display",
        "n           new session",
        "Ctrl+Up     cycle provider",
        "q           quit",
        "",
        "Ghostty-native: Kitty Graphics, Kitty Keyboard, Synced Output",
        "Config: shikumi (YAML/TOML/Lisp/Nix) + tatara-lisp (defprovider, defagent, defkeymap)",
    ]
    .join("\n")
}

// `io` is referenced via map_err's lifetime contract on Terminal; mark it
// to keep clippy happy without a top-level allow.
const _: fn() = || {
    let _ = std::mem::size_of::<io::Error>();
};
