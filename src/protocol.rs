use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatMessage {
    pub sender: String,
    pub content: String,
}

impl ChatMessage {
    /// Creates a new ChatMessage from a raw string
    /// (examples: "user:msg" or "sender:content" or "avery:bye")
    pub fn from_raw(raw: &str) -> Result<Self> {
        let parts: Vec<&str> = raw.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid message format: {}", raw));
        }
        Ok(ChatMessage {
            sender: parts[0].trim().to_string(),
            content: parts[1].trim().to_string(),
        })
    }
    /// Serializes the message to JSON
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}
