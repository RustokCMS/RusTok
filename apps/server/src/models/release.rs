//! Release model for tracking deployed releases
//!
//! Releases are immutable deployment artifacts that can be rolled back to.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Release status
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(None)")]
pub enum ReleaseStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "deploying")]
    Deploying,
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "rolled_back")]
    RolledBack,
    #[sea_orm(string_value = "failed")]
    Failed,
}

/// Release entity
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "releases")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    
    /// Release status
    pub status: ReleaseStatus,
    
    /// Associated build ID
    pub build_id: Uuid,
    
    /// Environment (prod, staging, etc.)
    pub environment: String,
    
    /// Container image URL
    pub container_image: Option<String>,
    
    /// Server binary artifact URL
    pub server_artifact_url: Option<String>,
    
    /// Admin UI artifact URL
    pub admin_artifact_url: Option<String>,
    
    /// Storefront artifact URL
    pub storefront_artifact_url: Option<String>,
    
    /// Manifest hash (for verification)
    pub manifest_hash: String,
    
    /// List of modules in this release
    pub modules: Json,
    
    /// Previous release ID (for rollback chain)
    pub previous_release_id: Option<String>,
    
    /// Deployed at
    pub deployed_at: Option<DateTime<Utc>>,
    
    /// Rolled back at
    pub rolled_back_at: Option<DateTime<Utc>>,
    
    /// Created at
    pub created_at: DateTime<Utc>,
    
    /// Updated at
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

use sea_orm::sea_query::Json;

impl Model {
    /// Generate a release ID
    pub fn generate_id() -> String {
        let now = Utc::now();
        format!(
            "rel_{:04}{:02}{:02}_{:02}{:02}{:02}",
            now.year(),
            now.month(),
            now.day(),
            now.hour(),
            now.minute(),
            now.second()
        )
    }
    
    /// Create a new release
    pub fn new(
        build_id: Uuid,
        environment: String,
        manifest_hash: String,
        modules: Vec<String>,
    ) -> Self {
        Self {
            id: Self::generate_id(),
            status: ReleaseStatus::Pending,
            build_id,
            environment,
            container_image: None,
            server_artifact_url: None,
            admin_artifact_url: None,
            storefront_artifact_url: None,
            manifest_hash,
            modules: serde_json::to_value(modules).unwrap().into(),
            previous_release_id: None,
            deployed_at: None,
            rolled_back_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    /// Mark as deployed
    pub fn mark_deployed(&mut self) {
        self.status = ReleaseStatus::Active;
        self.deployed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
    
    /// Mark as rolled back
    pub fn mark_rolled_back(&mut self) {
        self.status = ReleaseStatus::RolledBack;
        self.rolled_back_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}
