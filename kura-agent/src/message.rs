use kura_core::ContentBlock;

pub type Conversation = crate::context::Conversation;

pub trait MessageExt {
    fn to_openai_messages(&self) -> Vec<serde_json::Value>;
}

impl MessageExt for Conversation {
    fn to_openai_messages(&self) -> Vec<serde_json::Value> {
        self.messages()
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    kura_core::MessageRole::System => "system",
                    kura_core::MessageRole::User => "user",
                    kura_core::MessageRole::Assistant => "assistant",
                    kura_core::MessageRole::Tool => "tool",
                };
                let content: Vec<serde_json::Value> = msg
                    .content
                    .iter()
                    .map(|block| match block {
                        ContentBlock::Text { text } => serde_json::json!({
                            "type": "text",
                            "text": text,
                        }),
                        ContentBlock::ToolUse { id, name, input } => serde_json::json!({
                            "type": "tool_use",
                            "id": id,
                            "name": name,
                            "input": input,
                        }),
                        ContentBlock::ToolResult {
                            id,
                            content,
                            is_error,
                        } => serde_json::json!({
                            "type": "tool_result",
                            "id": id,
                            "content": content,
                            "is_error": is_error,
                        }),
                    })
                    .collect();
                serde_json::json!({
                    "role": role,
                    "content": content,
                })
            })
            .collect()
    }
}
