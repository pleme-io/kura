use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;

pub struct TerminalGuard;

impl TerminalGuard {
    pub fn init() -> anyhow::Result<()> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
        Ok(())
    }

    pub fn restore() -> anyhow::Result<()> {
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }
}

pub struct TerminalRestoreGuard {
    restored: bool,
}

impl TerminalRestoreGuard {
    pub fn new() -> anyhow::Result<Self> {
        TerminalGuard::init()?;
        Ok(Self { restored: false })
    }

    pub fn restore(&mut self) -> anyhow::Result<()> {
        if !self.restored {
            self.restored = true;
            TerminalGuard::restore()?;
        }
        Ok(())
    }
}

impl Drop for TerminalRestoreGuard {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}
