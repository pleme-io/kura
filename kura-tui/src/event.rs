use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};
use kura_agent::loop_runner::AgentEvent;
use kura_ghostty::GhosttyCapabilities;
use tokio_stream::StreamExt;

#[derive(Debug, Clone)]
pub enum TuiEvent {
    Key {
        key: KeyCode,
        modifiers: KeyModifiers,
    },
    Mouse(crossterm::event::MouseEvent),
    Resize {
        width: u16,
        height: u16,
    },
    Paste(String),
    Agent(AgentEvent),
    FocusGained,
    FocusLost,
    Tick,
}

pub struct TuiEventStream {
    capabilities: GhosttyCapabilities,
    tick_rate: std::time::Duration,
}

impl TuiEventStream {
    pub fn new(capabilities: GhosttyCapabilities, tick_rate: std::time::Duration) -> Self {
        Self {
            capabilities,
            tick_rate,
        }
    }

    pub async fn next_event(&mut self, event_stream: &mut EventStream) -> Option<TuiEvent> {
        let timeout = tokio::time::sleep(self.tick_rate);
        tokio::pin!(timeout);

        tokio::select! {
            maybe_event = event_stream.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key))) => {
                        if self.capabilities.kitty_keyboard {
                            Some(TuiEvent::Key { key: key.code, modifiers: key.modifiers })
                        } else {
                            Some(TuiEvent::Key { key: key.code, modifiers: key.modifiers })
                        }
                    }
                    Some(Ok(Event::Mouse(mouse))) => Some(TuiEvent::Mouse(mouse)),
                    Some(Ok(Event::Resize(w, h))) => Some(TuiEvent::Resize { width: w, height: h }),
                    Some(Ok(Event::Paste(s))) => Some(TuiEvent::Paste(s)),
                    Some(Ok(Event::FocusGained)) => Some(TuiEvent::FocusGained),
                    Some(Ok(Event::FocusLost)) => Some(TuiEvent::FocusLost),
                    _ => None,
                }
            }
            _ = &mut timeout => Some(TuiEvent::Tick),
        }
    }
}
