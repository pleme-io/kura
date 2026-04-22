use kura_core::{AgentSpec, ContentBlock, MessageRole};
use kura_provider::{CompletionRequest, ProviderRouter, RequestMessage, StreamEvent};
use kura_tool::ToolExecutor;
use tokio::sync::mpsc;

use crate::context::Conversation;

pub struct AgentLoop {
    spec: AgentSpec,
    router: ProviderRouter,
    tools: ToolExecutor,
    max_turns: usize,
    auto_approve: bool,
}

#[derive(Debug, Clone)]
pub enum AgentEvent {
    Thinking(String),
    Text(String),
    ToolCall {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        id: String,
        output: String,
        is_error: bool,
    },
    TurnComplete {
        turn: usize,
    },
    Done {
        reason: DoneReason,
    },
    Error(String),
}

#[derive(Debug, Clone)]
pub enum DoneReason {
    MaxTurns,
    NoToolCalls,
    UserInterrupt,
    ProviderError,
}

impl AgentLoop {
    pub fn new(spec: AgentSpec, router: ProviderRouter, tools: ToolExecutor) -> Self {
        let max_turns = spec.max_turns.unwrap_or(50) as usize;
        let auto_approve = spec.auto_approve;
        Self {
            spec,
            router,
            tools,
            max_turns,
            auto_approve,
        }
    }

    pub async fn run(
        &self,
        conversation: &mut Conversation,
        event_tx: mpsc::UnboundedSender<AgentEvent>,
        _approve_rx: Option<tokio::sync::oneshot::Receiver<bool>>,
    ) -> anyhow::Result<DoneReason> {
        let provider = self
            .spec
            .provider
            .as_deref()
            .unwrap_or(self.router.default_name());

        let adapter = self
            .router
            .get(provider)
            .ok_or_else(|| anyhow::anyhow!("provider '{}' not found", provider))?;

        let model = self.spec.model.clone().unwrap_or_default();
        let mut turn = 0;

        loop {
            if turn >= self.max_turns {
                let _ = event_tx.send(AgentEvent::Done {
                    reason: DoneReason::MaxTurns,
                });
                return Ok(DoneReason::MaxTurns);
            }

            let request = self.build_request(conversation, &model);
            let (stream_tx, mut stream_rx) = mpsc::unbounded_channel();

            let stream_result = adapter.stream(request, stream_tx).await;
            if let Err(e) = stream_result {
                let _ = event_tx.send(AgentEvent::Error(e.to_string()));
                let _ = event_tx.send(AgentEvent::Done {
                    reason: DoneReason::ProviderError,
                });
                return Ok(DoneReason::ProviderError);
            }

            let mut tool_calls: Vec<(String, String, serde_json::Value)> = vec![];
            let mut current_tool_id = String::new();
            let mut current_tool_name = String::new();
            let mut current_tool_input = String::new();
            let mut text_content = String::new();

            while let Some(event) = stream_rx.recv().await {
                match event {
                    StreamEvent::TextDelta(delta) => {
                        text_content.push_str(&delta);
                        let _ = event_tx.send(AgentEvent::Text(delta));
                    }
                    StreamEvent::ThinkingDelta(delta) => {
                        let _ = event_tx.send(AgentEvent::Thinking(delta));
                    }
                    StreamEvent::ToolUseStart { id, name } => {
                        current_tool_id = id;
                        current_tool_name = name;
                        current_tool_input.clear();
                        let _ = event_tx.send(AgentEvent::ToolCall {
                            id: current_tool_id.clone(),
                            name: current_tool_name.clone(),
                            input: serde_json::Value::Null,
                        });
                    }
                    StreamEvent::ToolUseInputDelta { id: _, delta } => {
                        current_tool_input.push_str(&delta);
                    }
                    StreamEvent::ToolUseInputEnd { id: _ } => {
                        let input: serde_json::Value = serde_json::from_str(&current_tool_input)
                            .unwrap_or(serde_json::Value::Object(Default::default()));
                        let _ = event_tx.send(AgentEvent::ToolCall {
                            id: current_tool_id.clone(),
                            name: current_tool_name.clone(),
                            input: input.clone(),
                        });
                        tool_calls.push((
                            current_tool_id.clone(),
                            current_tool_name.clone(),
                            input,
                        ));
                        current_tool_id.clear();
                        current_tool_name.clear();
                        current_tool_input.clear();
                    }
                    StreamEvent::Done(_) => break,
                    StreamEvent::Error(e) => {
                        let _ = event_tx.send(AgentEvent::Error(e));
                        let _ = event_tx.send(AgentEvent::Done {
                            reason: DoneReason::ProviderError,
                        });
                        return Ok(DoneReason::ProviderError);
                    }
                }
            }

            let mut assistant_content = vec![];
            if !text_content.is_empty() {
                assistant_content.push(ContentBlock::Text { text: text_content });
            }
            for (id, name, input) in &tool_calls {
                assistant_content.push(ContentBlock::ToolUse {
                    id: id.clone(),
                    name: name.clone(),
                    input: input.clone(),
                });
            }

            if assistant_content.is_empty() {
                let _ = event_tx.send(AgentEvent::Done {
                    reason: DoneReason::NoToolCalls,
                });
                return Ok(DoneReason::NoToolCalls);
            }

            conversation.add_message(MessageRole::Assistant, assistant_content);

            if tool_calls.is_empty() {
                let _ = event_tx.send(AgentEvent::Done {
                    reason: DoneReason::NoToolCalls,
                });
                return Ok(DoneReason::NoToolCalls);
            }

            for (id, name, input) in tool_calls {
                let approved = if self.auto_approve { true } else { true };

                if !approved {
                    conversation.add_message(
                        MessageRole::Tool,
                        vec![ContentBlock::ToolResult {
                            id: id.clone(),
                            content: "Tool call denied by user".to_string(),
                            is_error: true,
                        }],
                    );
                    let _ = event_tx.send(AgentEvent::ToolResult {
                        id,
                        output: "denied".to_string(),
                        is_error: true,
                    });
                    continue;
                }

                let result = self.tools.execute(&name, input).await;
                let (output, is_error) = match result {
                    Ok(out) => (out, false),
                    Err(e) => (e.to_string(), true),
                };

                conversation.add_message(
                    MessageRole::Tool,
                    vec![ContentBlock::ToolResult {
                        id: id.clone(),
                        content: output.clone(),
                        is_error,
                    }],
                );

                let _ = event_tx.send(AgentEvent::ToolResult {
                    id,
                    output,
                    is_error,
                });
            }

            turn += 1;
            let _ = event_tx.send(AgentEvent::TurnComplete { turn });
        }
    }

    fn build_request(&self, conversation: &Conversation, model: &str) -> CompletionRequest {
        let messages: Vec<RequestMessage> = conversation
            .messages()
            .iter()
            .map(|msg| RequestMessage {
                role: match msg.role {
                    MessageRole::System => "system".to_string(),
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                    MessageRole::Tool => "tool".to_string(),
                },
                content: msg.content.clone(),
            })
            .collect();

        CompletionRequest {
            model: model.to_string(),
            messages,
            max_tokens: None,
            temperature: None,
            stream: true,
            tools: self.tools.definitions(),
        }
    }
}
