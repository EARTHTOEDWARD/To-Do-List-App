use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use ulid::Ulid;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Copy)]
pub enum Kind {
    Task,
    Chat,
    Doc,
    Commit,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ObjectV1 {
    pub id: Ulid,
    pub kind: Kind,
    pub payload: serde_json::Value,
    pub created: DateTime<Utc>,
    pub updated: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EdgeV1 {
    pub id: Ulid,
    pub from: Ulid,
    pub to: Ulid,
    pub typ: String, // e.g. "CREATES", "SPAWNS", "FULFILLS"
    pub created: DateTime<Utc>,
}

/// CRDT operation
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Op {
    AppendObject(ObjectV1),
    AppendEdge(EdgeV1),
    UpdateObject { id: Ulid, patch: serde_json::Value },
}

impl ObjectV1 {
    pub fn new_task(title: &str, description: Option<&str>, priority: &str) -> Self {
        let priority_val = match priority {
            "high" | "medium" | "low" => priority,
            _ => "medium",
        };

        let payload = serde_json::json!({
            "title": title,
            "description": description,
            "completed": false,
            "archived": false,
            "priority": priority_val
        });

        Self {
            id: Ulid::new(),
            kind: Kind::Task,
            payload,
            created: Utc::now(),
            updated: None,
        }
    }

    pub fn new_chat(content: &str, role: &str) -> Self {
        let payload = serde_json::json!({
            "content": content,
            "role": role,
            "model": "gpt-4"
        });

        Self {
            id: Ulid::new(),
            kind: Kind::Chat,
            payload,
            created: Utc::now(),
            updated: None,
        }
    }
}

impl EdgeV1 {
    pub fn new(from: Ulid, to: Ulid, typ: &str) -> Self {
        Self {
            id: Ulid::new(),
            from,
            to,
            typ: typ.to_string(),
            created: Utc::now(),
        }
    }
}
