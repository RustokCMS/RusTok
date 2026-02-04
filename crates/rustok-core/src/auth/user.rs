use std::collections::HashMap;
use std::str::FromStr;

use alloy_scripting::integration::ScriptableEntity;
use chrono::Utc;
use rhai::Dynamic;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::types::{UserRole, UserStatus};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub email: String,
    #[serde(skip)]
    pub password_hash: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: UserRole,
    pub status: UserStatus,
    pub email_verified_at: Option<DateTimeWithTimeZone>,
    pub last_login_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {
    fn before_save(mut self, _insert: bool) -> Result<Self, DbErr> {
        self.updated_at = Set(chrono::DateTime::<chrono::FixedOffset>::from(Utc::now()));
        Ok(self)
    }
}

impl ScriptableEntity for Model {
    fn entity_type(&self) -> &'static str {
        "user"
    }

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn to_dynamic_map(&self) -> HashMap<String, Dynamic> {
        let mut map = HashMap::new();
        map.insert("id".into(), Dynamic::from(self.id.to_string()));
        map.insert("email".into(), Dynamic::from(self.email.clone()));
        if let Some(first_name) = &self.first_name {
            map.insert("first_name".into(), Dynamic::from(first_name.clone()));
        }
        if let Some(last_name) = &self.last_name {
            map.insert("last_name".into(), Dynamic::from(last_name.clone()));
        }
        map.insert("role".into(), Dynamic::from(self.role.to_string()));
        map.insert("status".into(), Dynamic::from(self.status.to_string()));
        map
    }

    fn apply_changes(&mut self, changes: HashMap<String, Dynamic>) {
        for (key, value) in changes {
            match key.as_str() {
                "first_name" => {
                    self.first_name = value.clone().try_cast::<String>();
                }
                "last_name" => {
                    self.last_name = value.clone().try_cast::<String>();
                }
                "role" => {
                    if let Some(role_str) = value.clone().try_cast::<String>() {
                        if let Ok(role) = UserRole::from_str(&role_str) {
                            self.role = role;
                        }
                    }
                }
                "status" => {
                    if let Some(status_str) = value.clone().try_cast::<String>() {
                        self.status = match status_str.as_str() {
                            "inactive" => UserStatus::Inactive,
                            "banned" => UserStatus::Banned,
                            _ => UserStatus::Active,
                        };
                    }
                }
                _ => {}
            }
        }
    }
}

impl Model {
    pub fn full_name(&self) -> String {
        match (&self.first_name, &self.last_name) {
            (Some(first), Some(last)) => format!("{first} {last}"),
            (Some(first), None) => first.clone(),
            (None, Some(last)) => last.clone(),
            (None, None) => self.email.clone(),
        }
    }

    pub fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::Admin | UserRole::SuperAdmin)
    }
}
