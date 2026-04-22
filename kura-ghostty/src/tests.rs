use crate::*;

macro_rules! env_set {
    ($key:expr, $val:expr) => {
        unsafe {
            std::env::set_var($key, $val);
        }
    };
}
macro_rules! env_remove {
    ($key:expr) => {
        unsafe {
            std::env::remove_var($key);
        }
    };
}

mod detect {
    use super::*;

    #[test]
    fn not_ghostty_when_unset() {
        env_remove!("TERM");
        env_remove!("GHOSTTY_RESOURCES_DIR");
        assert!(!GhosttyDetect::is_ghostty());
    }

    #[test]
    fn detect_via_term() {
        env_set!("TERM", "xterm-ghostty");
        assert!(GhosttyDetect::is_ghostty());
        env_remove!("TERM");
    }

    #[test]
    fn detect_via_resources_dir() {
        env_remove!("TERM");
        env_set!("GHOSTTY_RESOURCES_DIR", "/some/path");
        assert!(GhosttyDetect::is_ghostty());
        env_remove!("GHOSTTY_RESOURCES_DIR");
    }

    #[test]
    fn capabilities_default() {
        env_remove!("TERM");
        env_remove!("GHOSTTY_RESOURCES_DIR");
        let caps = GhosttyCapabilities::detect();
        assert!(!caps.is_ghostty);
    }

    #[test]
    fn capabilities_when_ghostty() {
        env_set!("TERM", "xterm-ghostty");
        let caps = GhosttyCapabilities::detect();
        assert!(caps.is_ghostty);
        assert!(caps.kitty_graphics);
        assert!(caps.kitty_keyboard);
        assert!(caps.synced_output);
        env_remove!("TERM");
    }
}

mod kitty_graphics {
    use super::*;

    #[test]
    fn query_support_sequence() {
        let seq = KittyGraphics::query_support();
        assert!(seq.contains("\x1b_G") && seq.contains("a=q") && seq.contains("\x1b\\"));
    }

    #[test]
    fn display_png_sequence() {
        let opts = ImageOptions {
            format: ImageFormat::Png,
            width: 100,
            height: 100,
            x_offset: 0,
            y_offset: 0,
            z_index: None,
            placement: Placement::AtCursor,
        };
        let seq = KittyGraphics::display_png(b"fake-data", &opts).unwrap();
        assert!(seq.contains("f=100") && seq.contains("s=100") && seq.contains("v=100"));
    }

    #[test]
    fn display_png_with_z_index() {
        let opts = ImageOptions {
            format: ImageFormat::Png,
            width: 50,
            height: 50,
            x_offset: 0,
            y_offset: 0,
            z_index: Some(5),
            placement: Placement::AtCursor,
        };
        let seq = KittyGraphics::display_png(b"data", &opts).unwrap();
        assert!(seq.contains("z=5"));
    }

    #[test]
    fn display_png_at_cell() {
        let opts = ImageOptions {
            format: ImageFormat::Png,
            width: 10,
            height: 10,
            x_offset: 0,
            y_offset: 0,
            z_index: None,
            placement: Placement::AtCell { row: 5, col: 10 },
        };
        let seq = KittyGraphics::display_png(b"d", &opts).unwrap();
        assert!(seq.contains("c=10") && seq.contains("r=5"));
    }

    #[test]
    fn delete_image_sequence() {
        let seq = KittyGraphics::delete_image(42);
        assert!(seq.contains("a=d") && seq.contains("i=42"));
    }

    #[test]
    fn display_rgba_sequence() {
        let opts = ImageOptions {
            format: ImageFormat::Rgba,
            width: 8,
            height: 8,
            x_offset: 0,
            y_offset: 0,
            z_index: None,
            placement: Placement::AtCursor,
        };
        let seq = KittyGraphics::display_rgba(&[0u8; 256], 8, 8, &opts).unwrap();
        assert!(seq.contains("f=32"));
    }
}

mod kitty_keyboard {
    use super::*;

    #[test]
    fn enable_disambiguate() {
        assert!(KittyKeyboard::enable(KeyboardMode::Disambiguate).contains("\x1b[>2u"));
    }
    #[test]
    fn enable_report_all() {
        assert!(KittyKeyboard::enable(KeyboardMode::ReportAll).contains("\x1b[>1u"));
    }
    #[test]
    fn disable() {
        assert!(KittyKeyboard::disable().contains("\x1b[<u"));
    }
    #[test]
    fn push_mode() {
        assert!(KittyKeyboard::push_mode(KeyboardMode::Disambiguate).contains("\x1b[>2;1u"));
    }
    #[test]
    fn pop_mode() {
        assert!(KittyKeyboard::pop_mode().contains("\x1b[>1;0u"));
    }
}

mod synced_output {
    use super::*;

    #[test]
    fn begin_sequence() {
        assert_eq!(SyncedOutput::begin(), "\x1b[?2026h");
    }
    #[test]
    fn end_sequence() {
        assert_eq!(SyncedOutput::end(), "\x1b[?2026l");
    }
    #[test]
    fn wrap_content() {
        let wrapped = SyncedOutput::wrap("hello");
        assert!(wrapped.starts_with("\x1b[?2026h") && wrapped.ends_with("\x1b[?2026l"));
    }
}
