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
