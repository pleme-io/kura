use std::io::Write;

pub struct KittyGraphics;

#[derive(Debug, Clone)]
pub struct ImageOptions {
    pub format: ImageFormat,
    pub width: u32,
    pub height: u32,
    pub x_offset: u32,
    pub y_offset: u32,
    pub z_index: Option<i32>,
    pub placement: Placement,
}

#[derive(Debug, Clone)]
pub enum ImageFormat {
    Rgba,
    Png,
}

#[derive(Debug, Clone)]
pub enum Placement {
    AtCursor,
    AtCell { row: u32, col: u32 },
    AtPixel { x: u32, y: u32 },
}

impl KittyGraphics {
    pub fn display_png(png_data: &[u8], opts: &ImageOptions) -> anyhow::Result<String> {
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, png_data);
        let mut control = format!("a=T,f=100,s={},v={}", opts.width, opts.height);

        if let Some(z) = opts.z_index {
            control.push_str(&format!(",z={}", z));
        }

        match opts.placement {
            Placement::AtCursor => {}
            Placement::AtCell { row, col } => {
                control.push_str(&format!(",c={},r={}", col, row));
            }
            Placement::AtPixel { x, y } => {
                control.push_str(&format!(",x={},y={}", x, y));
            }
        }

        Ok(format!("\x1b_G{};{}\x1b\\", control, encoded))
    }

    pub fn display_rgba(
        rgba_data: &[u8],
        width: u32,
        height: u32,
        opts: &ImageOptions,
    ) -> anyhow::Result<String> {
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, rgba_data);
        let mut control = format!("a=T,f=32,s={},v={}", width, height);

        if let Some(z) = opts.z_index {
            control.push_str(&format!(",z={}", z));
        }

        Ok(format!("\x1b_G{};{}\x1b\\", control, encoded))
    }

    pub fn delete_image(image_id: u32) -> String {
        format!("\x1b_Ga=d,d=I,i={}\x1b\\", image_id)
    }

    pub fn query_support() -> String {
        "\x1b_Ga=q,i=1;\x1b\\".to_string()
    }

    pub fn write_to_stdout(escape_seq: &str) -> anyhow::Result<()> {
        let mut stdout = std::io::stdout();
        stdout.write_all(escape_seq.as_bytes())?;
        stdout.flush()?;
        Ok(())
    }
}
