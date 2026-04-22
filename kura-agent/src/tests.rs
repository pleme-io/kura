use crate::*;

mod conversation {
    use super::*;
    use kura_core::{ContentBlock, MessageRole};

    #[test]
    fn new_with_system_prompt() {
        let conv = Conversation::new(Some("you are helpful".into()));
        assert_eq!(conv.system_prompt(), Some("you are helpful"));
        assert_eq!(conv.messages().len(), 1);
        assert!(matches!(conv.messages()[0].role, MessageRole::System));
    }

    #[test]
    fn new_without_system_prompt() {
        let conv = Conversation::new(None);
        assert!(conv.system_prompt().is_none());
        assert!(conv.messages().is_empty());
    }

    #[test]
    fn add_user_message() {
        let mut conv = Conversation::new(None);
        conv.add_user_message("hello".into());
        assert_eq!(conv.messages().len(), 1);
        assert!(matches!(conv.messages()[0].role, MessageRole::User));
    }

    #[test]
    fn add_assistant_message() {
        let mut conv = Conversation::new(None);
        conv.add_message(
            MessageRole::Assistant,
            vec![ContentBlock::Text { text: "hi".into() }],
        );
        assert_eq!(conv.messages().len(), 1);
    }

    #[test]
    fn last_assistant_text() {
        let mut conv = Conversation::new(None);
        conv.add_message(
            MessageRole::Assistant,
            vec![ContentBlock::Text {
                text: "response".into(),
            }],
        );
        assert_eq!(conv.last_assistant_text(), Some("response"));
    }

    #[test]
    fn last_assistant_text_none_when_no_assistant() {
        let mut conv = Conversation::new(None);
        conv.add_user_message("hello".into());
        assert!(conv.last_assistant_text().is_none());
    }

    #[test]
    fn token_estimate() {
        let mut conv = Conversation::new(None);
        conv.add_user_message("a".repeat(100));
        conv.add_message(
            MessageRole::Assistant,
            vec![ContentBlock::Text {
                text: "b".repeat(200),
            }],
        );
        let estimate = conv.token_estimate();
        assert!(estimate > 0);
        assert!(estimate < 100);
    }

    #[test]
    fn truncate_reduces_size() {
        let mut conv = Conversation::new(None);
        for i in 0..100 {
            conv.add_user_message(format!("message {} with lots of text to fill tokens", i));
        }
        let before = conv.token_estimate();
        conv.truncate_to_tokens(before / 2);
        assert!(conv.token_estimate() <= before / 2 + 50);
    }
}

mod session_store {
    use super::*;

    #[tokio::test]
    async fn save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let store = SessionStore::new(dir.path());
        let mut conv = Conversation::new(Some("test system prompt".into()));
        conv.add_user_message("hello".into());

        store.save("test-session", &conv).await.unwrap();
        let loaded = store.load("test-session").await.unwrap();
        assert!(!loaded.messages().is_empty());
    }

    #[tokio::test]
    async fn list_sessions() {
        let dir = tempfile::tempdir().unwrap();
        let store = SessionStore::new(dir.path());
        let conv = Conversation::new(None);

        store.save("session-a", &conv).await.unwrap();
        store.save("session-b", &conv).await.unwrap();

        let sessions = store.list().await.unwrap();
        assert_eq!(sessions.len(), 2);
        assert!(sessions.contains(&"session-a".to_string()));
        assert!(sessions.contains(&"session-b".to_string()));
    }

    #[tokio::test]
    async fn delete_session() {
        let dir = tempfile::tempdir().unwrap();
        let store = SessionStore::new(dir.path());
        let conv = Conversation::new(None);

        store.save("to-delete", &conv).await.unwrap();
        store.delete("to-delete").await.unwrap();
        assert!(store.load("to-delete").await.is_err());
    }
}
