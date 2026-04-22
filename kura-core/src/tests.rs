use crate::*;

mod provider_spec {
    use super::*;

    #[test]
    fn from_lisp_kwargs() {
        let src = r#"(defprovider :name "zen" :kind zen :api-key-env "OPENCODE_API_KEY" :model "opencode/claude-sonnet-4-20250514" :priority 10)"#;
        let results: Vec<ProviderSpec> = tatara_lisp::compile_typed(src).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "zen");
        assert!(matches!(results[0].kind, ProviderKind::Zen));
        assert_eq!(results[0].api_key_env.as_deref(), Some("OPENCODE_API_KEY"));
        assert_eq!(
            results[0].model.as_deref(),
            Some("opencode/claude-sonnet-4-20250514")
        );
        assert_eq!(results[0].priority, 10);
        assert!(!results[0].disabled);
    }

    #[test]
    fn content_id_stability() {
        let spec = ProviderSpec {
            name: "zen".into(),
            kind: ProviderKind::Zen,
            base_url: Some("https://opencode.ai/zen/v1".into()),
            api_key_env: Some("OPENCODE_API_KEY".into()),
            model: Some("opencode/claude-sonnet-4-20250514".into()),
            priority: 10,
            max_tokens: None,
            temperature: None,
            disabled: false,
        };
        let id1 = spec.content_id();
        let id2 = spec.content_id();
        assert_eq!(id1, id2);
        assert_eq!(id1.len(), 64);
    }

    #[test]
    fn content_id_differs_for_changes() {
        let spec1 = ProviderSpec {
            name: "zen".into(),
            kind: ProviderKind::Zen,
            base_url: None,
            api_key_env: Some("KEY".into()),
            model: Some("a".into()),
            priority: 10,
            max_tokens: None,
            temperature: None,
            disabled: false,
        };
        let mut spec2 = spec1.clone();
        spec2.model = Some("b".into());
        assert_ne!(spec1.content_id(), spec2.content_id());
    }

    #[test]
    fn kind_serde_roundtrip() {
        for kind in [
            ProviderKind::Zen,
            ProviderKind::OpenAi,
            ProviderKind::Anthropic,
            ProviderKind::Ollama,
            ProviderKind::Custom,
        ] {
            let json = serde_json::to_string(&kind).unwrap();
            assert_eq!(kind, serde_json::from_str(&json).unwrap());
        }
    }

    #[test]
    fn disabled_flag() {
        let src = r#"(defprovider :name "off" :kind custom :disabled #t)"#;
        let results: Vec<ProviderSpec> = tatara_lisp::compile_typed(src).unwrap();
        assert!(results[0].disabled);
    }
}

mod agent_spec {
    use super::*;

    #[test]
    fn from_lisp_named() {
        let src = r#"(defagent :name "coder" :kind coder :provider "zen" :max-turns 50 :auto-approve #f)"#;
        let results: Vec<AgentSpec> = tatara_lisp::compile_typed(src).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "coder");
        assert!(matches!(results[0].kind, AgentKind::Coder));
        assert_eq!(results[0].provider.as_deref(), Some("zen"));
        assert_eq!(results[0].max_turns, Some(50));
        assert!(!results[0].auto_approve);
    }

    #[test]
    fn kind_serde_roundtrip() {
        for kind in [
            AgentKind::Coder,
            AgentKind::Reviewer,
            AgentKind::Explorer,
            AgentKind::Planner,
            AgentKind::Custom,
        ] {
            let json = serde_json::to_string(&kind).unwrap();
            assert_eq!(kind, serde_json::from_str(&json).unwrap());
        }
    }

    #[test]
    fn thinking_budget() {
        let src = r#"(defagent :name "deep" :kind coder :thinking-budget "high")"#;
        let results: Vec<AgentSpec> = tatara_lisp::compile_typed(src).unwrap();
        assert_eq!(results[0].thinking_budget.as_deref(), Some("high"));
    }
}

mod tool_spec {
    use super::*;

    #[test]
    fn from_lisp_named() {
        let src = r#"(deftool :name "kubernetes" :kind mcp :mcp-server "kubernetes" :description "K8s operations")"#;
        let results: Vec<ToolSpec> = tatara_lisp::compile_typed(src).unwrap();
        assert_eq!(results[0].name, "kubernetes");
        assert!(matches!(results[0].kind, ToolKind::Mcp));
        assert_eq!(results[0].mcp_server.as_deref(), Some("kubernetes"));
    }

    #[test]
    fn kind_serde_roundtrip() {
        let kinds = [
            ToolKind::Shell,
            ToolKind::FileRead,
            ToolKind::FileWrite,
            ToolKind::FileEdit,
            ToolKind::Glob,
            ToolKind::Grep,
            ToolKind::Git,
            ToolKind::WebFetch,
            ToolKind::WebSearch,
            ToolKind::CodeSearch,
            ToolKind::Mcp,
            ToolKind::Custom,
        ];
        for kind in kinds {
            let json = serde_json::to_string(&kind).unwrap();
            assert_eq!(kind, serde_json::from_str(&json).unwrap());
        }
    }

    #[test]
    fn guardrail_flag() {
        let src = r#"(deftool :name "bash" :kind shell :guardrail #t)"#;
        let results: Vec<ToolSpec> = tatara_lisp::compile_typed(src).unwrap();
        assert!(results[0].guardrail);
    }
}

mod plugin_spec {
    use super::*;

    #[test]
    fn hook_from_lisp() {
        let src = r#"(defplugin :name "guardrail" :kind hook :phase preToolUse :command "guardrail check")"#;
        let results: Vec<PluginSpec> = tatara_lisp::compile_typed(src).unwrap();
        assert_eq!(results[0].name, "guardrail");
        assert!(matches!(results[0].kind, PluginKind::Hook));
        assert!(matches!(results[0].phase, HookPhase::PreToolUse));
    }

    #[test]
    fn transformer_from_lisp() {
        let src = r#"(defplugin :name "nordify" :kind transformer :lisp-transform "(rewrite-theme :style nord)")"#;
        let results: Vec<PluginSpec> = tatara_lisp::compile_typed(src).unwrap();
        assert!(matches!(results[0].kind, PluginKind::Transformer));
        assert!(results[0].lisp_transform.is_some());
    }
}

mod keymap_spec {
    use super::*;

    #[test]
    fn action_serde_roundtrip() {
        let actions = [
            Action::SubmitInput,
            Action::CancelInput,
            Action::NewSession,
            Action::SwitchSession,
            Action::ToggleFocus,
            Action::ScrollUp,
            Action::ScrollDown,
            Action::PageUp,
            Action::PageDown,
            Action::ToggleToolApproval,
            Action::ToggleGhosttyGraphics,
            Action::CycleProvider,
            Action::ToggleThinking,
            Action::OpenCommandPalette,
            Action::Quit,
            Action::Custom("test".into()),
        ];
        for action in actions {
            let json = serde_json::to_string(&action).unwrap();
            assert_eq!(action, serde_json::from_str(&json).unwrap());
        }
    }
}

mod session_content_blocks {
    use super::*;

    #[test]
    fn message_role_serde() {
        for role in [
            MessageRole::System,
            MessageRole::User,
            MessageRole::Assistant,
            MessageRole::Tool,
        ] {
            let json = serde_json::to_string(&role).unwrap();
            assert_eq!(role, serde_json::from_str(&json).unwrap());
        }
    }

    #[test]
    fn content_block_text_roundtrip() {
        let block = ContentBlock::Text {
            text: "hello".into(),
        };
        let json = serde_json::to_string(&block).unwrap();
        assert_eq!(block, serde_json::from_str(&json).unwrap());
    }

    #[test]
    fn content_block_tool_use_roundtrip() {
        let block = ContentBlock::ToolUse {
            id: "t1".into(),
            name: "bash".into(),
            input: serde_json::json!({"command": "ls"}),
        };
        let json = serde_json::to_string(&block).unwrap();
        assert_eq!(block, serde_json::from_str(&json).unwrap());
    }

    #[test]
    fn content_block_tool_result_roundtrip() {
        let block = ContentBlock::ToolResult {
            id: "t1".into(),
            content: "output".into(),
            is_error: false,
        };
        let json = serde_json::to_string(&block).unwrap();
        assert_eq!(block, serde_json::from_str(&json).unwrap());
    }
}

mod domain_registry {
    use super::*;

    #[test]
    fn register_and_lookup() {
        register_all();
        assert!(tatara_lisp::domain::lookup("defprovider").is_some());
        assert!(tatara_lisp::domain::lookup("defagent").is_some());
        assert!(tatara_lisp::domain::lookup("deftool").is_some());
        assert!(tatara_lisp::domain::lookup("defplugin").is_some());
        assert!(tatara_lisp::domain::lookup("defkeymap").is_some());
        assert!(tatara_lisp::domain::lookup("defsession").is_some());
        assert!(tatara_lisp::domain::lookup("nonexistent").is_none());
    }

    #[test]
    fn registry_dispatch_returns_json() {
        register_all();
        let handler = tatara_lisp::domain::lookup("defprovider").unwrap();
        assert_eq!(handler.keyword, "defprovider");
    }
}
