use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ScriptTrigger {
    Event {
        entity_type: String,
        event: EventType,
    },
    Cron {
        expression: String,
    },
    Manual,
    Api {
        path: String,
        method: HttpMethod,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    BeforeCreate,
    AfterCreate,
    BeforeUpdate,
    AfterUpdate,
    BeforeDelete,
    AfterDelete,
    OnCommit,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BeforeCreate => "before_create",
            Self::AfterCreate => "after_create",
            Self::BeforeUpdate => "before_update",
            Self::AfterUpdate => "after_update",
            Self::BeforeDelete => "before_delete",
            Self::AfterDelete => "after_delete",
            Self::OnCommit => "on_commit",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "before_create" => Some(Self::BeforeCreate),
            "after_create" => Some(Self::AfterCreate),
            "before_update" => Some(Self::BeforeUpdate),
            "after_update" => Some(Self::AfterUpdate),
            "before_delete" => Some(Self::BeforeDelete),
            "after_delete" => Some(Self::AfterDelete),
            "on_commit" => Some(Self::OnCommit),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    #[default]
    GET,
    POST,
    PUT,
    DELETE,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GET => "GET",
            Self::POST => "POST",
            Self::PUT => "PUT",
            Self::DELETE => "DELETE",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "GET" => Some(Self::GET),
            "POST" => Some(Self::POST),
            "PUT" => Some(Self::PUT),
            "DELETE" => Some(Self::DELETE),
            _ => None,
        }
    }
}
