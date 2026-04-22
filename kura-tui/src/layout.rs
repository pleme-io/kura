#[derive(Debug, Clone)]
pub struct Layout {
    pub conversation: Rect,
    pub input: Rect,
    pub status: Rect,
}

#[derive(Debug, Clone, Default)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Layout {
    pub fn compute(width: u16, height: u16) -> Self {
        let status_h = 1u16;
        let input_h = 1u16;
        let conversation_h = height.saturating_sub(status_h + input_h);

        Self {
            conversation: Rect {
                x: 0,
                y: 0,
                width,
                height: conversation_h,
            },
            input: Rect {
                x: 0,
                y: conversation_h,
                width,
                height: input_h,
            },
            status: Rect {
                x: 0,
                y: conversation_h + input_h,
                width,
                height: status_h,
            },
        }
    }
}
