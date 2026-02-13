//! Build Service for module installation
//!
//! Manages the lifecycle of builds from request to deployment.
//! This is the core service for the WordPress/NodeBB-style module management.

use crate::models::build::{ActiveModel as BuildActiveModel, BuildStatus, BuildStage, DeploymentProfile, Entity as BuildEntity, Model as Build};
use crate::models::release::{ActiveModel as ReleaseActiveModel, ReleaseStatus, Entity as ReleaseEntity, Model as Release};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set};
use std::collections::HashMap;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Build request input
#[derive(Debug, Clone)]
pub struct BuildRequest {
    pub manifest_ref: String,
    pub requested_by: String,
    pub reason: Option<String>,
    pub modules: HashMap<String, ModuleSpec>,
    pub profile: DeploymentProfile,
}

/// Module specification
#[derive(Debug, Clone)]
pub struct ModuleSpec {
    pub source: String,
    pub crate_name: String,
    pub version: Option<String>,
    pub git: Option<String>,
    pub rev: Option<String>,
    pub path: Option<String>,
}

/// Build event for async processing
#[derive(Debug, Clone)]
pub enum BuildEvent {
    BuildRequested { build_id: Uuid },
    BuildStarted { build_id: Uuid },
    BuildProgress { build_id: Uuid, stage: BuildStage, progress: i32 },
    BuildCompleted { build_id: Uuid, release_id: String },
    BuildFailed { build_id: Uuid, error: String },
}

/// Build service
pub struct BuildService {
    db: DatabaseConnection,
}

impl BuildService {
    /// Create new build service
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Request a new build
    ///
    /// Creates a build record and queues it for processing.
    pub async fn request_build(&self, request: BuildRequest) -> anyhow::Result<Build> {
        // Compute manifest hash
        let manifest_hash = compute_manifest_hash(&request.modules);
        
        // Check for existing build with same hash
        if let Some(existing) = self.find_build_by_hash(&manifest_hash).await? {
            if existing.status == BuildStatus::Success {
                info!(
                    build_id = %existing.id,
                    "Build with same manifest already exists, returning existing build"
                );
                return Ok(existing);
            }
        }

        // Create build record
        let build = Build::new(
            request.manifest_ref,
            manifest_hash,
            request.requested_by,
            request.profile,
        );

        let active_model = BuildActiveModel {
            id: Set(build.id),
            status: Set(build.status.clone()),
            stage: Set(build.stage.clone()),
            progress: Set(build.progress),
            profile: Set(build.profile.clone()),
            manifest_ref: Set(build.manifest_ref.clone()),
            manifest_hash: Set(build.manifest_hash.clone()),
            modules_delta: Set(Some(serde_json::to_value(&request.modules)?)),
            requested_by: Set(build.requested_by.clone()),
            reason: Set(request.reason),
            release_id: Set(None),
            logs_url: Set(None),
            error_message: Set(None),
            started_at: Set(None),
            finished_at: Set(None),
            created_at: Set(build.created_at),
            updated_at: Set(build.updated_at),
        };

        active_model.insert(&self.db).await?;

        info!(build_id = %build.id, "Build requested");

        // TODO: Publish BuildRequested event for async processing
        // self.event_bus.publish(...).await?;

        Ok(build)
    }

    /// Get build by ID
    pub async fn get_build(&self, build_id: Uuid) -> anyhow::Result<Option<Build>> {
        Ok(BuildEntity::find_by_id(build_id).one(&self.db).await?)
    }

    /// List recent builds
    pub async fn list_builds(&self, limit: u64) -> anyhow::Result<Vec<Build>> {
        let builds = BuildEntity::find()
            .order_by_desc(crate::models::build::Column::CreatedAt)
            .limit(limit)
            .all(&self.db)
            .await?;
        Ok(builds)
    }

    /// Find build by manifest hash
    async fn find_build_by_hash(&self, hash: &str) -> anyhow::Result<Option<Build>> {
        Ok(BuildEntity::find()
            .filter(crate::models::build::Column::ManifestHash.eq(hash))
            .one(&self.db)
            .await?)
    }

    /// Update build status
    pub async fn update_build_status(
        &self,
        build_id: Uuid,
        status: BuildStatus,
        stage: Option<BuildStage>,
        progress: Option<i32>,
    ) -> anyhow::Result<()> {
        let build = self.get_build(build_id).await?;
        
        if let Some(mut build) = build {
            let mut active_model: BuildActiveModel = build.into();
            active_model.status = Set(status.clone());
            
            if let Some(stage) = stage {
                active_model.stage = Set(stage);
            }
            if let Some(progress) = progress {
                active_model.progress = Set(progress);
            }
            
            active_model.updated_at = Set(Utc::now());
            
            if status == BuildStatus::Running && active_model.started_at.is_none() {
                active_model.started_at = Set(Some(Utc::now()));
            }
            
            if status.is_final() {
                active_model.finished_at = Set(Some(Utc::now()));
            }
            
            active_model.update(&self.db).await?;
        }

        Ok(())
    }

    /// Mark build as failed
    pub async fn fail_build(&self, build_id: Uuid, error: String) -> anyhow::Result<()> {
        let build = self.get_build(build_id).await?;
        
        if let Some(build) = build {
            let mut active_model: BuildActiveModel = build.into();
            active_model.status = Set(BuildStatus::Failed);
            active_model.error_message = Set(Some(error));
            active_model.finished_at = Set(Some(Utc::now()));
            active_model.updated_at = Set(Utc::now());
            
            active_model.update(&self.db).await?;
            
            error!(build_id = %build_id, "Build failed");
        }

        Ok(())
    }

    /// Create a release for successful build
    pub async fn create_release(
        &self,
        build_id: Uuid,
        environment: String,
        modules: Vec<String>,
    ) -> anyhow::Result<Release> {
        let build = self.get_build(build_id).await?;
        
        let build = build.ok_or_else(|| anyhow::anyhow!("Build not found"))?;
        
        let mut release = Release::new(
            build_id,
            environment,
            build.manifest_hash,
            modules,
        );

        // Link to previous active release for rollback chain
        if let Some(prev) = self.get_active_release().await? {
            release.previous_release_id = Some(prev.id);
        }

        let active_model = ReleaseActiveModel {
            id: Set(release.id.clone()),
            status: Set(release.status.clone()),
            build_id: Set(release.build_id),
            environment: Set(release.environment.clone()),
            container_image: Set(None),
            server_artifact_url: Set(None),
            admin_artifact_url: Set(None),
            storefront_artifact_url: Set(None),
            manifest_hash: Set(release.manifest_hash.clone()),
            modules: Set(release.modules.clone()),
            previous_release_id: Set(release.previous_release_id.clone()),
            deployed_at: Set(None),
            rolled_back_at: Set(None),
            created_at: Set(release.created_at),
            updated_at: Set(release.updated_at),
        };

        active_model.insert(&self.db).await?;

        // Update build with release ID
        let mut build_model: BuildActiveModel = build.into();
        build_model.release_id = Set(Some(release.id.clone()));
        build_model.update(&self.db).await?;

        info!(release_id = %release.id, "Release created");

        Ok(release)
    }

    /// Get release by ID
    pub async fn get_release(&self, release_id: &str) -> anyhow::Result<Option<Release>> {
        Ok(ReleaseEntity::find_by_id(release_id).one(&self.db).await?)
    }

    /// Get active release
    async fn get_active_release(&self) -> anyhow::Result<Option<Release>> {
        Ok(ReleaseEntity::find()
            .filter(crate::models::release::Column::Status.eq(ReleaseStatus::Active))
            .one(&self.db)
            .await?)
    }

    /// List releases
    pub async fn list_releases(&self, limit: u64) -> anyhow::Result<Vec<Release>> {
        let releases = ReleaseEntity::find()
            .order_by_desc(crate::models::release::Column::CreatedAt)
            .limit(limit)
            .all(&self.db)
            .await?;
        Ok(releases)
    }

    /// Rollback to previous release
    pub async fn rollback(&self, release_id: &str) -> anyhow::Result<Release> {
        let current = self.get_release(release_id).await?;
        let current = current.ok_or_else(|| anyhow::anyhow!("Release not found"))?;

        let previous_id = current
            .previous_release_id
            .clone()
            .ok_or_else(|| anyhow::anyhow!("No previous release to rollback to"))?;

        let previous = self.get_release(&previous_id).await?;
        let previous = previous.ok_or_else(|| anyhow::anyhow!("Previous release not found"))?;

        // Mark current as rolled back
        let mut current_model: ReleaseActiveModel = current.into();
        current_model.status = Set(ReleaseStatus::RolledBack);
        current_model.rolled_back_at = Set(Some(Utc::now()));
        current_model.updated_at = Set(Utc::now());
        current_model.update(&self.db).await?;

        // Mark previous as active
        let mut prev_model: ReleaseActiveModel = previous.clone().into();
        prev_model.status = Set(ReleaseStatus::Active);
        prev_model.deployed_at = Set(Some(Utc::now()));
        prev_model.updated_at = Set(Utc::now());
        prev_model.update(&self.db).await?;

        info!(
            from_release = %release_id,
            to_release = %previous_id,
            "Rollback completed"
        );

        Ok(previous)
    }
}

/// Compute hash of module configuration
fn compute_manifest_hash(modules: &HashMap<String, ModuleSpec>) -> String {
    use std::collections::BTreeMap;
    
    // Sort modules for consistent hashing
    let sorted: BTreeMap<_, _> = modules.iter().collect();
    let json = serde_json::to_string(&sorted).unwrap_or_default();
    
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    format!("{:x}", hasher.finalize())[..16].to_string()
}
