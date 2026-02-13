//! Build model for tracking module installation builds
//!
//! Tracks the lifecycle of a build from request to deployment.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Build status
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(None)")]
pub enum BuildStatus {
    #[sea_orm(string_value = "queued")]
    Queued,
    #[sea_orm(string_value = "running")]
    Running,
    #[sea_orm(string_value = "success")]
    Success,
    #[sea_orm(string_value = "failed")]
    Failed,
    #[sea_orm(string_value = "cancelled")]
    Cancelled,
}

/// Build stage
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(None)")]
pub enum BuildStage {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "checkout")]
    Checkout,
    #[sea_orm(string_value = "build")]
    Build,
    #[sea_orm(string_value = "test")]
    Test,
    #[sea_orm(string_value = "deploy")]
    Deploy,
    #[sea_orm(string_value = "complete")]
    Complete,
}

/// Deployment profile
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(None)")]
pub enum DeploymentProfile {
    #[sea_orm(string_value = "monolith")]
    Monolith,
    #[sea_orm(string_value = "headless")]
    Headless,
}

/// Build entity
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "builds")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    
    /// Build status
    pub status: BuildStatus,
    
    /// Current build stage
    pub stage: BuildStage,
    
    /// Progress percentage (0-100)
    pub progress: i32,
    
    /// Deployment profile
    pub profile: DeploymentProfile,
    
    /// Git reference (branch/tag/commit)
    pub manifest_ref: String,
    
    /// Hash of modules.toml content
    pub manifest_hash: String,
    
    /// Delta of modules (added/removed/changed)
    pub modules_delta: Option<Json>,
    
    /// Who requested the build
    pub requested_by: String,
    
    /// Reason for build
    pub reason: Option<String>,
    
    /// Associated release ID (if deployed)
    pub release_id: Option<String>,
    
    /// Logs URL
    pub logs_url: Option<String>,
    
    /// Error message (if failed)
    pub error_message: Option<String>,
    
    /// Build started at
    pub started_at: Option<DateTime<Utc>>,
    
    /// Build finished at
    pub finished_at: Option<DateTime<Utc>>,
    
    /// Created at
    pub created_at: DateTime<Utc>,
    
    /// Updated at
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Create a new build request
    pub fn new(
        manifest_ref: String,
        manifest_hash: String,
        requested_by: String,
        profile: DeploymentProfile,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            status: BuildStatus::Queued,
            stage: BuildStage::Pending,
            progress: 0,
            profile,
            manifest_ref,
            manifest_hash,
            modules_delta: None,
            requested_by,
            reason: None,
            release_id: None,
            logs_url: None,
            error_message: None,
            started_at: None,
            finished_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    /// Check if build is in a final state
    pub fn is_final(&self) -> bool {
        matches!(
            self.status,
            BuildStatus::Success | BuildStatus::Failed | BuildStatus::Cancelled
        )
    }
    
    /// Get duration of build (if finished)
    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.started_at, self.finished_at) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }
}

use sea_orm::sea_query::Json;
