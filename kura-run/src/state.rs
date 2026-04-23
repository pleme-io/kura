//! Execution state management for DAG nodes

use crate::dag::{NodeId, StateId};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct NodeSnapshot {
    pub node_id: NodeId,
    pub state: ExecutionState,
    pub output: Option<String>,
    pub error: Option<String>,
    pub attempts: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionState {
    Pending,
    Waiting,
    Running,
    Verifying,
    Retrying,
    Success,
    Failed,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct NodeMessage {
    pub from: NodeId,
    pub to: NodeId,
    pub payload: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl NodeMessage {
    pub fn new(from: NodeId, to: NodeId, payload: impl Into<String>) -> Self {
        Self {
            from,
            to,
            payload: payload.into(),
            timestamp: chrono::Utc::now(),
        }
    }
}

pub struct ExecutionContext {
    state: Arc<RwLock<ExecutionStateInner>>,
}

struct ExecutionStateInner {
    node_states: HashMap<NodeId, ExecutionState>,
    node_outputs: HashMap<NodeId, String>,
    node_errors: HashMap<NodeId, String>,
    node_attempts: HashMap<NodeId, u32>,
    messages: Vec<NodeMessage>,
    custom_context: HashMap<String, String>,
    custom_state: HashMap<String, StateId>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(ExecutionStateInner {
                node_states: HashMap::new(),
                node_outputs: HashMap::new(),
                node_errors: HashMap::new(),
                node_attempts: HashMap::new(),
                messages: Vec::new(),
                custom_context: HashMap::new(),
                custom_state: HashMap::new(),
            })),
        }
    }

    pub fn init_node(&self, node_id: NodeId) {
        let mut state = self.state.write();
        let id = node_id.clone();
        state.node_states.insert(node_id, ExecutionState::Pending);
        state.node_attempts.insert(id, 0);
    }

    pub fn set_state(&self, node_id: &NodeId, state: ExecutionState) {
        let mut inner = self.state.write();
        inner.node_states.insert(node_id.clone(), state);
    }

    pub fn get_state(&self, node_id: &NodeId) -> ExecutionState {
        let inner = self.state.read();
        inner.node_states.get(node_id).copied().unwrap_or(ExecutionState::Pending)
    }

    pub fn set_output(&self, node_id: &NodeId, output: String) {
        let mut inner = self.state.write();
        inner.node_outputs.insert(node_id.clone(), output);
    }

    pub fn get_output(&self, node_id: &NodeId) -> Option<String> {
        let inner = self.state.read();
        inner.node_outputs.get(node_id).cloned()
    }

    pub fn set_error(&self, node_id: &NodeId, error: String) {
        let mut inner = self.state.write();
        inner.node_errors.insert(node_id.clone(), error);
    }

    pub fn get_error(&self, node_id: &NodeId) -> Option<String> {
        let inner = self.state.read();
        inner.node_errors.get(node_id).cloned()
    }

    pub fn increment_attempts(&self, node_id: &NodeId) -> u32 {
        let mut inner = self.state.write();
        let count = inner.node_attempts.entry(node_id.clone()).or_insert(0);
        *count += 1;
        *count
    }

    pub fn get_attempts(&self, node_id: &NodeId) -> u32 {
        let inner = self.state.read();
        *inner.node_attempts.get(node_id).unwrap_or(&0)
    }

    pub fn send_message(&self, from: NodeId, to: NodeId, payload: String) {
        let mut inner = self.state.write();
        inner.messages.push(NodeMessage::new(from, to, payload));
    }

    pub fn get_messages_for(&self, node_id: &NodeId) -> Vec<NodeMessage> {
        let inner = self.state.read();
        inner.messages.iter()
            .filter(|m| &m.to == node_id)
            .cloned()
            .collect()
    }

    pub fn set_context(&self, key: impl Into<String>, value: impl Into<String>) {
        let mut inner = self.state.write();
        inner.custom_context.insert(key.into(), value.into());
    }

    pub fn get_context(&self, key: &str) -> Option<String> {
        let inner = self.state.read();
        inner.custom_context.get(key).cloned()
    }

    pub fn set_state_machine_state(&self, key: impl Into<String>, state: StateId) {
        let mut inner = self.state.write();
        inner.custom_state.insert(key.into(), state);
    }

    pub fn get_state_machine_state(&self, key: &str) -> Option<StateId> {
        let inner = self.state.read();
        inner.custom_state.get(key).cloned()
    }

    pub fn get_all_outputs(&self) -> HashMap<NodeId, String> {
        let inner = self.state.read();
        inner.node_outputs.clone()
    }

    pub fn snapshot(&self, node_id: &NodeId) -> NodeSnapshot {
        let inner = self.state.read();
        NodeSnapshot {
            node_id: node_id.clone(),
            state: *inner.node_states.get(node_id).unwrap_or(&ExecutionState::Pending),
            output: inner.node_outputs.get(node_id).cloned(),
            error: inner.node_errors.get(node_id).cloned(),
            attempts: *inner.node_attempts.get(node_id).unwrap_or(&0),
        }
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_state_transitions() {
        let ctx = ExecutionContext::new();
        let node_id = NodeId::new("test-node");

        ctx.init_node(node_id.clone());
        assert_eq!(ctx.get_state(&node_id), ExecutionState::Pending);

        ctx.set_state(&node_id, ExecutionState::Running);
        assert_eq!(ctx.get_state(&node_id), ExecutionState::Running);

        ctx.set_output(&node_id, "result".to_string());
        assert_eq!(ctx.get_output(&node_id), Some("result".to_string()));
    }

    #[test]
    fn test_retry_counting() {
        let ctx = ExecutionContext::new();
        let node_id = NodeId::new("retry-test");

        ctx.init_node(node_id.clone());
        assert_eq!(ctx.get_attempts(&node_id), 0);

        ctx.increment_attempts(&node_id);
        ctx.increment_attempts(&node_id);
        ctx.increment_attempts(&node_id);

        assert_eq!(ctx.get_attempts(&node_id), 3);
    }

    #[test]
    fn test_message_passing() {
        let ctx = ExecutionContext::new();
        let from = NodeId::new("node-a");
        let to = NodeId::new("node-b");

        ctx.send_message(from.clone(), to.clone(), "hello".to_string());

        let messages = ctx.get_messages_for(&to);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].from, from);
        assert_eq!(messages[0].payload, "hello");
    }

    #[test]
    fn test_context_variables() {
        let ctx = ExecutionContext::new();

        ctx.set_context("key1", "value1");
        ctx.set_context("key2", "value2");

        assert_eq!(ctx.get_context("key1"), Some("value1".to_string()));
        assert_eq!(ctx.get_context("key2"), Some("value2".to_string()));
        assert_eq!(ctx.get_context("key3"), None);
    }
}