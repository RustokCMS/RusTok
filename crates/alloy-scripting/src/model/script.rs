use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::trigger::ScriptTrigger;

pub type ScriptId = Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ScriptStatus {
    #[default]
    Draft,
    Active,
    Paused,
    Disabled,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: ScriptId,
    pub name: String,
    pub description: Option<String>,
    pub code: String,
    pub trigger: ScriptTrigger,
    pub status: ScriptStatus,
    pub version: u32,
    pub run_as_system: bool,
    pub permissions: Vec<String>,
    pub author_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub error_count: u32,
    pub last_error_at: Option<DateTime<Utc>>,
}

impl Script {
    pub fn new(name: impl Into<String>, code: impl Into<String>, trigger: ScriptTrigger) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: None,
            code: code.into(),
            trigger,
            status: ScriptStatus::Draft,
            version: 1,
            run_as_system: false,
            permissions: Vec::new(),
            author_id: None,
            created_at: now,
            updated_at: now,
            error_count: 0,
            last_error_at: None,
        }
    }

    pub fn is_executable(&self) -> bool {
        self.status == ScriptStatus::Active
    }

    pub fn register_error(&mut self) -> bool {
        self.error_count += 1;
        self.last_error_at = Some(Utc::now());
        self.error_count >= 3
    }

    pub fn reset_errors(&mut self) {
        self.error_count = 0;
        self.last_error_at = None;
    }

    pub fn activate(&mut self) {
        self.status = ScriptStatus::Active;
        self.reset_errors();
        self.updated_at = Utc::now();
    }

    pub fn disable(&mut self) {
        self.status = ScriptStatus::Disabled;
        self.updated_at = Utc::now();
    }

    pub fn archive(&mut self) {
        self.status = ScriptStatus::Archived;
        self.updated_at = Utc::now();
    }
}

impl ScriptStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Disabled => "disabled",
            Self::Archived => "archived",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "draft" => Some(Self::Draft),
            "active" => Some(Self::Active),
            "paused" => Some(Self::Paused),
            "disabled" => Some(Self::Disabled),
            "archived" => Some(Self::Archived),
            _ => None,
        }
    }
}
