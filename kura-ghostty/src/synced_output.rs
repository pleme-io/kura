use std::io::Write;

pub struct SyncedOutput;

impl SyncedOutput {
    pub fn begin() -> String {
        "\x1b[?2026h".to_string()
    }

    pub fn end() -> String {
        "\x1b[?2026l".to_string()
    }

    pub fn wrap(content: &str) -> String {
        format!("{}{}{}", Self::begin(), content, Self::end())
    }

    pub fn begin_on_stdout() -> anyhow::Result<()> {
        let mut stdout = std::io::stdout();
        stdout.write_all(Self::begin().as_bytes())?;
        stdout.flush()?;
        Ok(())
    }

    pub fn end_on_stdout() -> anyhow::Result<()> {
        let mut stdout = std::io::stdout();
        stdout.write_all(Self::end().as_bytes())?;
        stdout.flush()?;
        Ok(())
    }
}
