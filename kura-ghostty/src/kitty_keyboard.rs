use std::io::Write;

pub struct KittyKeyboard;

#[derive(Debug, Clone)]
pub enum KeyboardMode {
    ReportAll,
    Disambiguate,
    ReportAlternate,
    ReportAllKeys,
}

impl KittyKeyboard {
    pub fn enable(mode: KeyboardMode) -> String {
        let mode_num = match mode {
            KeyboardMode::ReportAll => 1,
            KeyboardMode::Disambiguate => 2,
            KeyboardMode::ReportAlternate => 3,
            KeyboardMode::ReportAllKeys => 4,
        };
        format!("\x1b[>{}u", mode_num)
    }

    pub fn disable() -> String {
        "\x1b[<u".to_string()
    }

    pub fn push_mode(mode: KeyboardMode) -> String {
        let mode_num = match mode {
            KeyboardMode::ReportAll => 1,
            KeyboardMode::Disambiguate => 2,
            KeyboardMode::ReportAlternate => 3,
            KeyboardMode::ReportAllKeys => 4,
        };
        format!("\x1b[>{};1u", mode_num)
    }

    pub fn pop_mode() -> String {
        "\x1b[>1;0u".to_string()
    }

    pub fn write_to_stdout(seq: &str) -> anyhow::Result<()> {
        let mut stdout = std::io::stdout();
        stdout.write_all(seq.as_bytes())?;
        stdout.flush()?;
        Ok(())
    }
}
