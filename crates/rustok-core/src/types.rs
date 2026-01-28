use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
pub enum UserRoleParseError {
    #[error("Invalid user role: {0}")]
    Invalid(String),
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    EnumIter,
    DeriveActiveEnum,
    Default,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(32))")]
pub enum UserRole {
    #[sea_orm(string_value = "super_admin")]
    SuperAdmin,
    #[sea_orm(string_value = "admin")]
    Admin,
    #[sea_orm(string_value = "manager")]
    Manager,
    #[sea_orm(string_value = "customer")]
    #[default]
    Customer,
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::SuperAdmin => "super_admin",
            Self::Admin => "admin",
            Self::Manager => "manager",
            Self::Customer => "customer",
        };
        write!(f, "{value}")
    }
}

impl FromStr for UserRole {
    type Err = UserRoleParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "super_admin" => Ok(Self::SuperAdmin),
            "admin" => Ok(Self::Admin),
            "manager" => Ok(Self::Manager),
            "customer" => Ok(Self::Customer),
            _ => Err(UserRoleParseError::Invalid(value.to_string())),
        }
    }
}

#[derive(
    Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum, Default,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(32))")]
pub enum UserStatus {
    #[sea_orm(string_value = "active")]
    #[default]
    Active,
    #[sea_orm(string_value = "inactive")]
    Inactive,
    #[sea_orm(string_value = "banned")]
    Banned,
}

impl fmt::Display for UserStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
            Self::Banned => "banned",
        };
        write!(f, "{value}")
    }
}
