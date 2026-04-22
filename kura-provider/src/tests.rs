use crate::*;

mod router {
    use super::*;
    use kura_core::ProviderSpec;

    #[test]
    fn new_with_zen_default() {
        let specs = vec![ProviderSpec {
            name: "zen".into(),
            kind: kura_core::ProviderKind::Zen,
            base_url: Some("https://opencode.ai/zen/v1".into()),
            api_key_env: Some("OPENCODE_API_KEY".into()),
            model: Some("opencode/claude-sonnet-4-20250514".into()),
            priority: 10,
            max_tokens: None,
            temperature: None,
            disabled: false,
        }];
        let router = ProviderRouter::new(&specs);
        assert_eq!(router.default_name(), "zen");
        assert!(router.get("zen").is_some());
    }

    #[test]
    fn highest_priority_is_default() {
        let specs = vec![
            ProviderSpec {
                name: "low".into(),
                kind: kura_core::ProviderKind::Custom,
                base_url: None,
                api_key_env: Some("KEY".into()),
                model: Some("m1".into()),
                priority: 1,
                max_tokens: None,
                temperature: None,
                disabled: false,
            },
            ProviderSpec {
                name: "high".into(),
                kind: kura_core::ProviderKind::Custom,
                base_url: None,
                api_key_env: Some("KEY".into()),
                model: Some("m2".into()),
                priority: 100,
                max_tokens: None,
                temperature: None,
                disabled: false,
            },
        ];
        let router = ProviderRouter::new(&specs);
        assert_eq!(router.default_name(), "high");
    }

    #[test]
    fn disabled_provider_excluded() {
        let specs = vec![ProviderSpec {
            name: "off".into(),
            kind: kura_core::ProviderKind::Custom,
            base_url: None,
            api_key_env: Some("KEY".into()),
            model: Some("m".into()),
            priority: 100,
            max_tokens: None,
            temperature: None,
            disabled: true,
        }];
        let router = ProviderRouter::new(&specs);
        assert_eq!(router.default_name(), "zen");
        assert!(router.get("off").is_none());
    }

    #[test]
    fn providers_iterator() {
        let specs = vec![
            ProviderSpec {
                name: "a".into(),
                kind: kura_core::ProviderKind::Custom,
                base_url: None,
                api_key_env: None,
                model: None,
                priority: 1,
                max_tokens: None,
                temperature: None,
                disabled: false,
            },
            ProviderSpec {
                name: "b".into(),
                kind: kura_core::ProviderKind::Custom,
                base_url: None,
                api_key_env: None,
                model: None,
                priority: 2,
                max_tokens: None,
                temperature: None,
                disabled: false,
            },
        ];
        let router = ProviderRouter::new(&specs);
        let names: Vec<&str> = router.providers().map(|(n, _)| n).collect();
        assert!(names.contains(&"a"));
        assert!(names.contains(&"b"));
    }
}

mod zen_adapter {
    use super::*;
    use kura_core::ProviderSpec;

    #[test]
    fn from_spec_defaults() {
        let spec = ProviderSpec {
            name: "zen".into(),
            kind: kura_core::ProviderKind::Zen,
            base_url: None,
            api_key_env: None,
            model: None,
            priority: 10,
            max_tokens: None,
            temperature: None,
            disabled: false,
        };
        let adapter = ZenAdapter::from_spec(&spec);
        assert_eq!(adapter.name(), "zen");
        assert!(adapter.base_url.contains("opencode.ai"));
    }

    #[test]
    fn model_resolution_with_prefix() {
        let spec = ProviderSpec {
            name: "zen".into(),
            kind: kura_core::ProviderKind::Zen,
            base_url: None,
            api_key_env: None,
            model: Some("opencode/claude-opus-4-6".into()),
            priority: 10,
            max_tokens: None,
            temperature: None,
            disabled: false,
        };
        let adapter = ZenAdapter::from_spec(&spec);
        let request = adapter::CompletionRequest {
            model: "default".into(),
            messages: vec![],
            max_tokens: None,
            temperature: None,
            stream: false,
            tools: vec![],
        };
        let resolved = adapter.resolve_model(&request);
        assert_eq!(resolved, "opencode/claude-opus-4-6");
    }
}

mod openai_compat {
    use super::*;
    use kura_core::ProviderSpec;

    #[test]
    fn from_spec_defaults() {
        let spec = ProviderSpec {
            name: "openai".into(),
            kind: kura_core::ProviderKind::OpenAi,
            base_url: None,
            api_key_env: None,
            model: None,
            priority: 5,
            max_tokens: None,
            temperature: None,
            disabled: false,
        };
        let adapter = OpenAiCompatAdapter::from_spec(&spec);
        assert_eq!(adapter.name(), "openai-compat");
    }

    #[test]
    fn from_spec_with_base() {
        let spec = ProviderSpec {
            name: "ollama".into(),
            kind: kura_core::ProviderKind::Ollama,
            base_url: None,
            api_key_env: None,
            model: None,
            priority: 1,
            max_tokens: None,
            temperature: None,
            disabled: false,
        };
        let adapter = OpenAiCompatAdapter::from_spec_with_base(&spec, "http://localhost:11434/v1");
        assert!(adapter.base_url.contains("localhost"));
    }
}

mod completion_types {
    use super::*;
    use kura_core::ContentBlock;

    #[test]
    fn response_from_openai_json() {
        let json = serde_json::json!({
            "id": "chatcmpl-123",
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "Hello!"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5
            }
        });
        let resp = CompletionResponse::from_openai_response(json, "test-model").unwrap();
        assert_eq!(resp.id, "chatcmpl-123");
        assert_eq!(resp.usage.input_tokens, 10);
        assert_eq!(resp.usage.output_tokens, 5);
    }

    #[test]
    fn response_with_tool_calls() {
        let json = serde_json::json!({
            "id": "chatcmpl-456",
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [{
                        "id": "call_1",
                        "type": "function",
                        "function": {
                            "name": "bash",
                            "arguments": "{\"command\":\"ls\"}"
                        }
                    }]
                }
            }],
            "usage": { "prompt_tokens": 20, "completion_tokens": 10 }
        });
        let resp = CompletionResponse::from_openai_response(json, "test").unwrap();
        assert_eq!(resp.content.len(), 1);
        if let ContentBlock::ToolUse { name, .. } = &resp.content[0] {
            assert_eq!(name, "bash");
        } else {
            panic!("expected ToolUse block");
        }
    }
}
