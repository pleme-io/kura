use kura_core::{ContentBlock, MessageRole};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: Vec<ContentBlock>,
}

#[derive(Debug, Clone)]
pub struct Conversation {
    messages: Vec<Message>,
    system_prompt: Option<String>,
}

impl Conversation {
    pub fn new(system_prompt: Option<String>) -> Self {
        let mut messages = vec![];
        if let Some(sp) = &system_prompt {
            messages.push(Message {
                role: MessageRole::System,
                content: vec![ContentBlock::Text { text: sp.clone() }],
            });
        }
        Self {
            messages,
            system_prompt,
        }
    }

    pub fn add_user_message(&mut self, text: String) {
        self.messages.push(Message {
            role: MessageRole::User,
            content: vec![ContentBlock::Text { text }],
        });
    }

    pub fn add_message(&mut self, role: MessageRole, content: Vec<ContentBlock>) {
        self.messages.push(Message { role, content });
    }

    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn system_prompt(&self) -> Option<&str> {
        self.system_prompt.as_deref()
    }

    pub fn last_assistant_text(&self) -> Option<&str> {
        self.messages.iter().rev().find_map(|msg| {
            if matches!(msg.role, MessageRole::Assistant) {
                msg.content.iter().find_map(|block| {
                    if let ContentBlock::Text { text } = block {
                        Some(text.as_str())
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        })
    }

    pub fn token_estimate(&self) -> usize {
        let mut total = 0;
        for msg in &self.messages {
            for block in &msg.content {
                match block {
                    ContentBlock::Text { text } => total += text.len() / 4,
                    ContentBlock::ToolUse { input, .. } => total += input.to_string().len() / 4,
                    ContentBlock::ToolResult { content, .. } => total += content.len() / 4,
                }
            }
        }
        total
    }

    pub fn truncate_to_tokens(&mut self, max_tokens: usize) {
        while self.token_estimate() > max_tokens && self.messages.len() > 2 {
            if let Some(idx) = self
                .messages
                .iter()
                .position(|m| matches!(m.role, MessageRole::User))
            {
                self.messages.remove(idx);
            } else {
                break;
            }
        }
    }
}
