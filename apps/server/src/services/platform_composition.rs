use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

use rustok_core::ModuleRegistry;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::models::platform_state::{
    ActiveModel as PlatformStateActiveModel, Column as PlatformStateColumn,
    Entity as PlatformStateEntity, Model as PlatformStateModel,
};
use crate::modules::{ManifestError, ManifestManager, ModulesManifest};

pub const ACTIVE_PLATFORM_STATE_ID: &str = "active";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformCompositionSnapshot {
    pub revision: i64,
    pub manifest_hash: String,
    pub manifest: ModulesManifest,
}

#[derive(Debug, Error)]
pub enum PlatformCompositionError {
    #[error(transparent)]
    Database(#[from] DbErr),
    #[error(transparent)]
    Manifest(#[from] ManifestError),
    #[error("Failed to serialize platform manifest: {0}")]
    Serialize(String),
    #[error("Failed to deserialize platform manifest: {0}")]
    Deserialize(String),
    #[error("Platform manifest revision conflict: expected {expected}, current {current}")]
    RevisionConflict { expected: i64, current: i64 },
}

pub struct PlatformCompositionService;

impl PlatformCompositionService {
    pub async fn active_snapshot(
        db: &DatabaseConnection,
    ) -> Result<PlatformCompositionSnapshot, PlatformCompositionError> {
        let state = Self::active_state(db).await?;
        Self::snapshot_from_state(state)
    }

    pub async fn active_manifest(
        db: &DatabaseConnection,
    ) -> Result<ModulesManifest, PlatformCompositionError> {
        Ok(Self::active_snapshot(db).await?.manifest)
    }

    pub async fn active_state(
        db: &DatabaseConnection,
    ) -> Result<PlatformStateModel, PlatformCompositionError> {
        if let Some(state) = PlatformStateEntity::find_by_id(ACTIVE_PLATFORM_STATE_ID.to_string())
            .one(db)
            .await?
        {
            return Ok(state);
        }

        let manifest = Self::bootstrap_manifest()?;
        let manifest_json = serde_json::to_value(&manifest)
            .map_err(|err| PlatformCompositionError::Serialize(err.to_string()))?;
        let manifest_hash = Self::manifest_hash(&manifest);
        let now = chrono::Utc::now().into();
        let active = PlatformStateActiveModel {
            id: Set(ACTIVE_PLATFORM_STATE_ID.to_string()),
            revision: Set(1),
            manifest_json: Set(manifest_json),
            manifest_hash: Set(manifest_hash),
            active_release_id: Set(None),
            updated_by: Set(Some("bootstrap".to_string())),
            created_at: Set(now),
            updated_at: Set(now),
        };

        match active.insert(db).await {
            Ok(state) => Ok(state),
            Err(_) => PlatformStateEntity::find_by_id(ACTIVE_PLATFORM_STATE_ID.to_string())
                .one(db)
                .await?
                .ok_or_else(|| {
                    PlatformCompositionError::Database(DbErr::RecordNotFound(
                        "platform_state.active".to_string(),
                    ))
                }),
        }
    }

    pub async fn update_manifest(
        db: &DatabaseConnection,
        registry: &ModuleRegistry,
        expected_revision: Option<i64>,
        manifest: ModulesManifest,
        updated_by: Option<String>,
    ) -> Result<PlatformCompositionSnapshot, PlatformCompositionError> {
        ManifestManager::validate_with_registry(&manifest, registry)?;

        let current = Self::active_state(db).await?;
        if let Some(expected) = expected_revision {
            if expected != current.revision {
                return Err(PlatformCompositionError::RevisionConflict {
                    expected,
                    current: current.revision,
                });
            }
        }

        let next_revision = current.revision + 1;
        let manifest_json = serde_json::to_value(&manifest)
            .map_err(|err| PlatformCompositionError::Serialize(err.to_string()))?;
        let manifest_hash = Self::manifest_hash(&manifest);
        let result = PlatformStateEntity::update_many()
            .filter(PlatformStateColumn::Id.eq(ACTIVE_PLATFORM_STATE_ID))
            .filter(PlatformStateColumn::Revision.eq(current.revision))
            .col_expr(PlatformStateColumn::Revision, Expr::value(next_revision))
            .col_expr(
                PlatformStateColumn::ManifestJson,
                Expr::value(manifest_json.clone()),
            )
            .col_expr(
                PlatformStateColumn::ManifestHash,
                Expr::value(manifest_hash.clone()),
            )
            .col_expr(
                PlatformStateColumn::UpdatedBy,
                Expr::value(updated_by.clone()),
            )
            .col_expr(
                PlatformStateColumn::UpdatedAt,
                Expr::value(chrono::Utc::now()),
            )
            .exec(db)
            .await?;

        if result.rows_affected != 1 {
            let refreshed = Self::active_state(db).await?;
            return Err(PlatformCompositionError::RevisionConflict {
                expected: current.revision,
                current: refreshed.revision,
            });
        }

        Ok(PlatformCompositionSnapshot {
            revision: next_revision,
            manifest_hash,
            manifest,
        })
    }

    pub fn manifest_hash(manifest: &ModulesManifest) -> String {
        let sorted = manifest
            .modules
            .iter()
            .map(|(slug, spec)| {
                (
                    slug.clone(),
                    (
                        spec.source.clone(),
                        spec.crate_name.clone(),
                        spec.version.clone(),
                        spec.git.clone(),
                        spec.rev.clone(),
                        spec.path.clone(),
                        spec.required,
                        spec.depends_on.clone(),
                    ),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let serialized = serde_json::to_string(&sorted).unwrap_or_default();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        serialized.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    fn snapshot_from_state(
        state: PlatformStateModel,
    ) -> Result<PlatformCompositionSnapshot, PlatformCompositionError> {
        let manifest = serde_json::from_value(state.manifest_json)
            .map_err(|err| PlatformCompositionError::Deserialize(err.to_string()))?;
        Ok(PlatformCompositionSnapshot {
            revision: state.revision,
            manifest_hash: state.manifest_hash,
            manifest,
        })
    }

    fn bootstrap_manifest() -> Result<ModulesManifest, PlatformCompositionError> {
        if let Ok(manifest) = ManifestManager::load() {
            return Ok(manifest);
        }

        let raw = include_str!("../../../../modules.toml");
        toml::from_str(raw).map_err(|err| {
            PlatformCompositionError::Manifest(ManifestError::Parse {
                path: "embedded modules.toml".to_string(),
                error: err.to_string(),
            })
        })
    }
}
