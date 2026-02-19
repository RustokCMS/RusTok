//! Build Service for module installation
//!
//! Manages the lifecycle of builds from request to deployment.

use crate::models::build::{
    ActiveModel as BuildActiveModel, BuildStage, BuildStatus, DeploymentProfile,
    Entity as BuildEntity, Model as Build,
};
use crate::models::release::{
    ActiveModel as ReleaseActiveModel, Entity as ReleaseEntity, Model as Release, ReleaseStatus,
};
use async_trait::async_trait;
use chrono::Utc;
use rustok_core::{events::DomainEvent, EventBus};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct BuildRequest {
    pub manifest_ref: String,
    pub requested_by: String,
    pub reason: Option<String>,
    pub modules: HashMap<String, ModuleSpec>,
    pub profile: DeploymentProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSpec {
    pub source: String,
    pub crate_name: String,
    pub version: Option<String>,
    pub git: Option<String>,
    pub rev: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone)]
pub enum BuildEvent {
    BuildRequested {
        build_id: Uuid,
        requested_by: String,
    },
    BuildStarted {
        build_id: Uuid,
    },
    BuildProgress {
        build_id: Uuid,
        stage: BuildStage,
        progress: i32,
    },
    BuildCompleted {
        build_id: Uuid,
        release_id: String,
    },
    BuildFailed {
        build_id: Uuid,
        error: String,
    },
}

#[async_trait]
pub trait BuildEventPublisher: Send + Sync {
    async fn publish(&self, event: BuildEvent) -> anyhow::Result<()>;
}

#[derive(Default)]
pub struct NoopBuildEventPublisher;

#[async_trait]
impl BuildEventPublisher for NoopBuildEventPublisher {
    async fn publish(&self, event: BuildEvent) -> anyhow::Result<()> {
        warn!(?event, "Build event publisher is not configured, skipping event");
        Ok(())
    }
}

pub struct EventBusBuildEventPublisher {
    event_bus: EventBus,
    tenant_id: Uuid,
}

impl EventBusBuildEventPublisher {
    pub fn new(event_bus: EventBus, tenant_id: Uuid) -> Self {
        Self {
            event_bus,
            tenant_id,
        }
    }
}

#[async_trait]
impl BuildEventPublisher for EventBusBuildEventPublisher {
    async fn publish(&self, event: BuildEvent) -> anyhow::Result<()> {
        let domain_event = match event {
            BuildEvent::BuildRequested {
                build_id,
                requested_by,
            } => DomainEvent::BuildRequested {
                build_id,
                requested_by,
            },
            unsupported => {
                warn!(?unsupported, "Build event is not mapped to DomainEvent yet, skipping");
                return Ok(());
            }
        };

        self.event_bus
            .publish(self.tenant_id, None, domain_event)
            .map_err(|error| anyhow::anyhow!("failed to publish build event: {error}"))
    }
}

pub struct BuildService {
    db: DatabaseConnection,
    event_publisher: Arc<dyn BuildEventPublisher>,
}

impl BuildService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            event_publisher: Arc::new(NoopBuildEventPublisher),
        }
    }

    pub fn with_event_publisher(
        db: DatabaseConnection,
        event_publisher: Arc<dyn BuildEventPublisher>,
    ) -> Self {
        Self { db, event_publisher }
    }

    pub async fn request_build(&self, request: BuildRequest) -> anyhow::Result<Build> {
        let manifest_hash = compute_manifest_hash(&request.modules);

        if let Some(existing) = self.find_build_by_hash(&manifest_hash).await? {
            if existing.status == BuildStatus::Success {
                info!(
                    build_id = %existing.id,
                    "Build with same manifest already exists, returning existing build"
                );
                return Ok(existing);
            }
        }

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

        self.event_publisher
            .publish(BuildEvent::BuildRequested {
                build_id: build.id,
                requested_by: build.requested_by.clone(),
            })
            .await?;

        Ok(build)
    }

    pub async fn get_build(&self, build_id: Uuid) -> anyhow::Result<Option<Build>> {
        Ok(BuildEntity::find_by_id(build_id).one(&self.db).await?)
    }

    pub async fn list_builds(&self, limit: u64) -> anyhow::Result<Vec<Build>> {
        let builds = BuildEntity::find()
            .order_by_desc(crate::models::build::Column::CreatedAt)
            .limit(limit)
            .all(&self.db)
            .await?;
        Ok(builds)
    }

    async fn find_build_by_hash(&self, hash: &str) -> anyhow::Result<Option<Build>> {
        Ok(BuildEntity::find()
            .filter(crate::models::build::Column::ManifestHash.eq(hash))
            .one(&self.db)
            .await?)
    }

    /// Update build status atomically inside a transaction to prevent TOCTOU races.
    pub async fn update_build_status(
        &self,
        build_id: Uuid,
        status: BuildStatus,
        stage: Option<BuildStage>,
        progress: Option<i32>,
    ) -> anyhow::Result<()> {
        self.db
            .transaction::<_, (), sea_orm::DbErr>(|txn| {
                let status = status.clone();
                let stage = stage.clone();
                Box::pin(async move {
                    let build = BuildEntity::find_by_id(build_id).one(txn).await?;
                    let Some(build) = build else {
                        return Ok(());
                    };

                    if build.is_final() {
                        return Ok(());
                    }

                    let now = Utc::now();
                    let started_at_is_none = build.started_at.is_none();
                    let mut active_model: BuildActiveModel = build.into();
                    active_model.status = Set(status.clone());

                    if let Some(stage) = stage {
                        active_model.stage = Set(stage);
                    }
                    if let Some(progress) = progress {
                        active_model.progress = Set(progress);
                    }

                    active_model.updated_at = Set(now);

                    if status == BuildStatus::Running && started_at_is_none {
                        active_model.started_at = Set(Some(now));
                    }

                    if status.is_final() {
                        active_model.finished_at = Set(Some(now));
                    }

                    active_model.update(txn).await?;
                    Ok(())
                })
            })
            .await
            .map_err(|e| anyhow::anyhow!("Failed to update build status: {e}"))
    }

    /// Mark build as failed atomically inside a transaction.
    pub async fn fail_build(&self, build_id: Uuid, err_msg: String) -> anyhow::Result<()> {
        self.db
            .transaction::<_, (), sea_orm::DbErr>(|txn| {
                let err_msg = err_msg.clone();
                Box::pin(async move {
                    let build = BuildEntity::find_by_id(build_id).one(txn).await?;
                    let Some(build) = build else {
                        return Ok(());
                    };

                    if build.is_final() {
                        return Ok(());
                    }

                    let now = Utc::now();
                    let mut active_model: BuildActiveModel = build.into();
                    active_model.status = Set(BuildStatus::Failed);
                    active_model.error_message = Set(Some(err_msg));
                    active_model.finished_at = Set(Some(now));
                    active_model.updated_at = Set(now);
                    active_model.update(txn).await?;
                    Ok(())
                })
            })
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fail build: {e}"))?;

        error!(build_id = %build_id, "Build failed");
        Ok(())
    }

    pub async fn create_release(
        &self,
        build_id: Uuid,
        environment: String,
        modules: Vec<String>,
    ) -> anyhow::Result<Release> {
        let build = self
            .get_build(build_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Build not found"))?;

        let mut release = Release::new(build_id, environment, build.manifest_hash, modules);

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

        let mut build_model: BuildActiveModel = build.into();
        build_model.release_id = Set(Some(release.id.clone()));
        build_model.update(&self.db).await?;

        info!(release_id = %release.id, "Release created");

        Ok(release)
    }

    pub async fn get_release(&self, release_id: &str) -> anyhow::Result<Option<Release>> {
        Ok(ReleaseEntity::find_by_id(release_id).one(&self.db).await?)
    }

    async fn get_active_release(&self) -> anyhow::Result<Option<Release>> {
        Ok(ReleaseEntity::find()
            .filter(crate::models::release::Column::Status.eq(ReleaseStatus::Active))
            .one(&self.db)
            .await?)
    }

    pub async fn list_releases(&self, limit: u64) -> anyhow::Result<Vec<Release>> {
        let releases = ReleaseEntity::find()
            .order_by_desc(crate::models::release::Column::CreatedAt)
            .limit(limit)
            .all(&self.db)
            .await?;
        Ok(releases)
    }

    pub async fn rollback(&self, release_id: &str) -> anyhow::Result<Release> {
        let current = self
            .get_release(release_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Release not found"))?;

        let previous_id = current
            .previous_release_id
            .clone()
            .ok_or_else(|| anyhow::anyhow!("No previous release to rollback to"))?;

        let previous = self
            .get_release(&previous_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Previous release not found"))?;

        let mut current_model: ReleaseActiveModel = current.into();
        current_model.status = Set(ReleaseStatus::RolledBack);
        current_model.rolled_back_at = Set(Some(Utc::now()));
        current_model.updated_at = Set(Utc::now());
        current_model.update(&self.db).await?;

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

/// Compute a full SHA-256 hex digest of the module configuration.
/// Modules are sorted by key to ensure deterministic output.
fn compute_manifest_hash(modules: &HashMap<String, ModuleSpec>) -> String {
    use sha2::{Digest, Sha256};
    use std::collections::BTreeMap;

    let sorted: BTreeMap<_, _> = modules.iter().collect();
    let json = serde_json::to_string(&sorted).unwrap_or_default();

    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    format!("{:x}", hasher.finalize())
}
