use codex_protocol::protocol::{Event, EventMsg};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// HTTP message wrapper for Codex events
/// This directly uses the EventMsg from codex_protocol
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct HttpMessage {
    /// The event ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Working directory for command execution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_dir: Option<String>,

    /// The actual event message from Codex protocol
    #[serde(flatten)]
    pub event: EventMsg,
}

impl HttpMessage {
    /// Create a new HTTP message from an Event
    pub fn from_event(event: Event) -> Self {
        Self {
            id: Some(event.id),
            work_dir: None,
            event: event.msg,
        }
    }

    /// Create a new HTTP message from an EventMsg
    pub fn new(event: EventMsg) -> Self {
        Self {
            id: None,
            work_dir: None,
            event,
        }
    }

    /// Create an HTTP message with an ID
    pub fn with_id(event: EventMsg, id: String) -> Self {
        Self {
            id: Some(id),
            work_dir: None,
            event,
        }
    }

    /// Convert the message to a JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Parse an HTTP message from a JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Convert to a full Event struct
    pub fn to_event(&self) -> Event {
        Event {
            id: self.id.clone().unwrap_or_default(),
            msg: self.event.clone(),
        }
    }
}
