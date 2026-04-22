use crate::*;

mod theme {
    use super::*;

    #[test]
    fn nord_theme_has_all_colors() {
        let theme = KuraTheme::nord();
        assert!(matches!(theme.bg, crossterm::style::Color::Rgb { .. }));
        assert!(matches!(theme.fg, crossterm::style::Color::Rgb { .. }));
        assert!(matches!(theme.accent, crossterm::style::Color::Rgb { .. }));
        assert!(matches!(theme.error, crossterm::style::Color::Rgb { .. }));
        assert!(matches!(theme.success, crossterm::style::Color::Rgb { .. }));
        assert!(matches!(theme.warn, crossterm::style::Color::Rgb { .. }));
    }

    #[test]
    fn nord_theme_values() {
        let theme = KuraTheme::nord();
        assert!(matches!(
            theme.accent,
            crossterm::style::Color::Rgb {
                r: 136,
                g: 192,
                b: 208
            }
        ));
        assert!(matches!(
            theme.error,
            crossterm::style::Color::Rgb {
                r: 191,
                g: 97,
                b: 106
            }
        ));
        assert!(matches!(
            theme.success,
            crossterm::style::Color::Rgb {
                r: 163,
                g: 190,
                b: 140
            }
        ));
    }
}

mod layout {
    use super::*;
    use crate::layout::Layout;

    #[test]
    fn compute_layout() {
        let layout = Layout::compute(80, 24);
        assert_eq!(layout.conversation.width, 80);
        assert_eq!(layout.input.height, 1);
        assert_eq!(layout.status.height, 1);
        assert_eq!(layout.conversation.height, 22);
    }

    #[test]
    fn compute_small_terminal() {
        let layout = Layout::compute(40, 5);
        assert_eq!(layout.conversation.height, 3);
    }
}

mod components {
    use super::*;
    use crate::components::{ConversationPane, HelpOverlay, InputBar, StatusBar, ToolApprovalPane};

    #[test]
    fn conversation_format_thinking() {
        let theme = KuraTheme::nord();
        let lines = ConversationPane::format_thinking("thinking step 1", &theme);
        assert!(!lines.is_empty());
        assert!(lines[0].0.contains("thinking step 1"));
    }

    #[test]
    fn conversation_format_tool_call() {
        let theme = KuraTheme::nord();
        let (text, _, _) = ConversationPane::format_tool_call("bash", &theme);
        assert!(text.contains("bash"));
    }

    #[test]
    fn conversation_format_user_message() {
        let theme = KuraTheme::nord();
        let lines = ConversationPane::format_user_message("hello world", &theme);
        assert!(!lines.is_empty());
        assert!(lines[0].0.contains("hello world"));
    }

    #[test]
    fn conversation_format_tool_result() {
        let theme = KuraTheme::nord();
        let lines = ConversationPane::format_tool_result("output", false, &theme);
        assert!(!lines.is_empty());
        let lines_err = ConversationPane::format_tool_result("error msg", true, &theme);
        assert!(!lines_err.is_empty());
    }

    #[test]
    fn status_bar_format() {
        let theme = KuraTheme::nord();
        let status = StatusBar::format("zen", "opus", 5, true, true, false, Some("done"), &theme);
        assert!(status.contains("zen"));
        assert!(status.contains("opus"));
        assert!(status.contains("G"));
    }

    #[test]
    fn help_overlay_format() {
        let theme = KuraTheme::nord();
        let lines = HelpOverlay::format_lines(&theme);
        assert!(!lines.is_empty());
        assert!(lines.iter().any(|(l, _, _)| l.contains("kura")));
    }

    #[test]
    fn tool_approval_format() {
        let theme = KuraTheme::nord();
        let lines =
            ToolApprovalPane::format_lines("bash", &serde_json::json!({"command": "ls"}), &theme);
        assert_eq!(lines.len(), 3);
        assert!(lines[0].0.contains("bash"));
    }
}

mod app {
    use super::*;
    use crate::app::{AppAction, Focus, View};
    use crossterm::event::{KeyCode, KeyModifiers};
    use kura_core::{AgentKind, AgentSpec, ProviderKind, ProviderSpec};
    use kura_provider::ProviderRouter;

    fn test_app() -> App {
        let caps = kura_ghostty::GhosttyCapabilities::detect();
        let spec = AgentSpec {
            name: "coder".into(),
            kind: AgentKind::Coder,
            system_prompt: None,
            tools: vec![],
            plugins: vec![],
            provider: Some("zen".into()),
            model: Some("opus".into()),
            max_turns: Some(50),
            auto_approve: false,
            thinking_budget: None,
        };
        let router = ProviderRouter::new(&[ProviderSpec {
            name: "zen".into(),
            kind: ProviderKind::Zen,
            base_url: None,
            api_key_env: None,
            model: None,
            priority: 10,
            max_tokens: None,
            temperature: None,
            disabled: false,
        }]);
        App::new(KuraTheme::nord(), &spec, router, caps)
    }

    #[test]
    fn initial_state() {
        let app = test_app();
        assert!(app.running);
        assert_eq!(app.focus, Focus::Input);
        assert_eq!(app.view, View::Chat);
    }

    #[test]
    fn ctrl_c_quits() {
        let mut app = test_app();
        let action = app.handle_event(TuiEvent::Key {
            key: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
        });
        assert!(!app.running);
        assert!(matches!(action, Some(AppAction::Quit)));
    }

    #[test]
    fn enter_submits_input() {
        let mut app = test_app();
        app.input_buffer = "hello".into();
        app.input_cursor = 5;
        let action = app.handle_event(TuiEvent::Key {
            key: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
        });
        assert!(matches!(action, Some(AppAction::SubmitInput(ref t)) if t == "hello"));
        assert!(app.input_buffer.is_empty());
    }

    #[test]
    fn typing_in_input() {
        let mut app = test_app();
        app.handle_event(TuiEvent::Key {
            key: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(app.input_buffer, "a");
        app.handle_event(TuiEvent::Key {
            key: KeyCode::Char('b'),
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(app.input_buffer, "ab");
    }

    #[test]
    fn backspace_in_input() {
        let mut app = test_app();
        app.input_buffer = "abc".into();
        app.input_cursor = 3;
        app.handle_event(TuiEvent::Key {
            key: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(app.input_buffer, "ab");
        assert_eq!(app.input_cursor, 2);
    }

    #[test]
    fn esc_switches_focus() {
        let mut app = test_app();
        app.focus = Focus::Input;
        app.handle_event(TuiEvent::Key {
            key: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(app.focus, Focus::Conversation);
    }

    #[test]
    fn tab_switches_to_conversation() {
        let mut app = test_app();
        app.focus = Focus::Input;
        app.handle_event(TuiEvent::Key {
            key: KeyCode::Tab,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(app.focus, Focus::Conversation);
    }

    #[test]
    fn conversation_keys_scroll() {
        let mut app = test_app();
        app.focus = Focus::Conversation;
        app.handle_event(TuiEvent::Key {
            key: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(app.conversation_scroll, 3);
        app.handle_event(TuiEvent::Key {
            key: KeyCode::Char('k'),
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(app.conversation_scroll, 0);
    }

    #[test]
    fn toggle_thinking() {
        let mut app = test_app();
        app.focus = Focus::Conversation;
        let before = app.show_thinking;
        app.handle_event(TuiEvent::Key {
            key: KeyCode::Char('t'),
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(app.show_thinking, !before);
    }

    #[test]
    fn q_quits_from_conversation() {
        let mut app = test_app();
        app.focus = Focus::Conversation;
        let action = app.handle_event(TuiEvent::Key {
            key: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
        });
        assert!(!app.running);
        assert!(matches!(action, Some(AppAction::Quit)));
    }
}
