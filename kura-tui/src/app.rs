use crate::event::TuiEvent;
use crate::theme::KuraTheme;
use crossterm::event::{KeyCode, KeyModifiers};
use kura_agent::loop_runner::AgentEvent;
use kura_core::AgentSpec;
use kura_ghostty::{GhosttyCapabilities, KeyboardMode, KittyKeyboard};
use kura_provider::ProviderRouter;

#[derive(Debug, Clone, PartialEq)]
pub enum Focus {
    Conversation,
    Input,
    ToolApproval,
    SessionList,
    CommandPalette,
}

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Chat,
    Sessions,
    Help,
}

pub struct App {
    pub theme: KuraTheme,
    pub view: View,
    pub focus: Focus,
    pub running: bool,
    pub capabilities: GhosttyCapabilities,
    pub show_thinking: bool,
    pub show_tool_output: bool,
    pub input_buffer: String,
    pub input_cursor: usize,
    pub conversation_scroll: usize,
    pub status_message: Option<String>,
    pub agent_name: String,
    pub model_name: String,
    pub provider_name: String,
    pub session_count: usize,
    pub turn_count: usize,
    pub pending_tool: Option<PendingToolCall>,
    provider_router: ProviderRouter,
}

#[derive(Debug, Clone)]
pub struct PendingToolCall {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

impl App {
    pub fn new(
        theme: KuraTheme,
        spec: &AgentSpec,
        router: ProviderRouter,
        capabilities: GhosttyCapabilities,
    ) -> Self {
        let model_name = spec.model.clone().unwrap_or_default();
        let provider_name = spec
            .provider
            .clone()
            .unwrap_or_else(|| router.default_name().to_string());
        Self {
            theme,
            view: View::Chat,
            focus: Focus::Input,
            running: true,
            capabilities,
            show_thinking: true,
            show_tool_output: true,
            input_buffer: String::new(),
            input_cursor: 0,
            conversation_scroll: 0,
            status_message: None,
            agent_name: spec.name.clone(),
            model_name,
            provider_name,
            session_count: 1,
            turn_count: 0,
            pending_tool: None,
            provider_router: router,
        }
    }

    pub fn handle_event(&mut self, event: TuiEvent) -> Option<AppAction> {
        match event {
            TuiEvent::Key { key, modifiers } => self.handle_key(key, modifiers),
            TuiEvent::Resize { .. } => None,
            TuiEvent::Tick => None,
            TuiEvent::Agent(agent_event) => self.handle_agent_event(agent_event),
            _ => None,
        }
    }

    fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Option<AppAction> {
        if modifiers.contains(KeyModifiers::CONTROL) && key == KeyCode::Char('c') {
            self.running = false;
            return Some(AppAction::Quit);
        }

        match self.focus {
            Focus::Input => self.handle_input_key(key, modifiers),
            Focus::Conversation => self.handle_conversation_key(key, modifiers),
            Focus::ToolApproval => self.handle_approval_key(key),
            Focus::SessionList => self.handle_session_key(key),
            Focus::CommandPalette => None,
        }
    }

    fn handle_input_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Option<AppAction> {
        match key {
            KeyCode::Enter => {
                let text = self.input_buffer.clone();
                if !text.is_empty() {
                    self.input_buffer.clear();
                    self.input_cursor = 0;
                    return Some(AppAction::SubmitInput(text));
                }
                None
            }
            KeyCode::Esc => {
                self.focus = Focus::Conversation;
                None
            }
            KeyCode::Backspace => {
                if self.input_cursor > 0 {
                    self.input_cursor -= 1;
                    self.input_buffer.remove(self.input_cursor);
                }
                None
            }
            KeyCode::Left => {
                if self.input_cursor > 0 {
                    self.input_cursor -= 1;
                }
                None
            }
            KeyCode::Right => {
                if self.input_cursor < self.input_buffer.len() {
                    self.input_cursor += 1;
                }
                None
            }
            KeyCode::Char(c) => {
                self.input_buffer.insert(self.input_cursor, c);
                self.input_cursor += 1;
                None
            }
            KeyCode::Tab => {
                self.focus = Focus::Conversation;
                None
            }
            KeyCode::Up if modifiers.contains(KeyModifiers::CONTROL) => {
                Some(AppAction::CycleProvider)
            }
            _ => None,
        }
    }

    fn handle_conversation_key(
        &mut self,
        key: KeyCode,
        _modifiers: KeyModifiers,
    ) -> Option<AppAction> {
        match key {
            KeyCode::Char('i') | KeyCode::Tab => {
                self.focus = Focus::Input;
                None
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.conversation_scroll = self.conversation_scroll.saturating_add(3);
                None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.conversation_scroll = self.conversation_scroll.saturating_sub(3);
                None
            }
            KeyCode::Char('G') | KeyCode::End => None,
            KeyCode::Char('g') | KeyCode::Home => {
                self.conversation_scroll = 0;
                None
            }
            KeyCode::Char('t') => {
                self.show_thinking = !self.show_thinking;
                None
            }
            KeyCode::Char('o') => {
                self.show_tool_output = !self.show_tool_output;
                None
            }
            KeyCode::Char('n') => Some(AppAction::NewSession),
            KeyCode::Char('?') => {
                self.view = View::Help;
                None
            }
            KeyCode::Char('q') => {
                self.running = false;
                Some(AppAction::Quit)
            }
            _ => None,
        }
    }

    fn handle_approval_key(&mut self, key: KeyCode) -> Option<AppAction> {
        match key {
            KeyCode::Char('y') | KeyCode::Enter => {
                let tool = self.pending_tool.take()?;
                Some(AppAction::ApproveTool(tool.id, tool.name, tool.input))
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                let tool = self.pending_tool.take()?;
                Some(AppAction::DenyTool(tool.id))
            }
            _ => None,
        }
    }

    fn handle_session_key(&mut self, key: KeyCode) -> Option<AppAction> {
        match key {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.focus = Focus::Conversation;
                None
            }
            KeyCode::Enter => Some(AppAction::SwitchSession),
            _ => None,
        }
    }

    fn handle_agent_event(&mut self, event: AgentEvent) -> Option<AppAction> {
        match event {
            AgentEvent::ToolCall { id, name, input } => {
                if !self.pending_tool.is_some() {
                    self.pending_tool = Some(PendingToolCall { id, name, input });
                    self.focus = Focus::ToolApproval;
                }
                None
            }
            AgentEvent::TurnComplete { turn } => {
                self.turn_count = turn;
                None
            }
            AgentEvent::Done { .. } => {
                self.status_message = Some("done".to_string());
                None
            }
            AgentEvent::Error(e) => {
                self.status_message = Some(format!("error: {}", e));
                None
            }
            _ => None,
        }
    }

    pub fn setup_ghostty(&self) {
        if self.capabilities.kitty_keyboard {
            let _ =
                KittyKeyboard::write_to_stdout(&KittyKeyboard::enable(KeyboardMode::Disambiguate));
        }
    }

    pub fn teardown_ghostty(&self) {
        if self.capabilities.kitty_keyboard {
            let _ = KittyKeyboard::write_to_stdout(&KittyKeyboard::disable());
        }
    }
}

#[derive(Debug, Clone)]
pub enum AppAction {
    SubmitInput(String),
    ApproveTool(String, String, serde_json::Value),
    DenyTool(String),
    NewSession,
    SwitchSession,
    CycleProvider,
    ToggleThinking,
    ToggleToolOutput,
    Quit,
}
