use std::collections::HashSet;

use rustok_core::ModuleRegistry;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::models::_entities::tenant_modules::{
    Column as TenantModulesColumn, Entity as TenantModulesEntity,
};
use crate::services::platform_composition::{PlatformCompositionError, PlatformCompositionService};

pub struct EffectiveModulePolicyService;

impl EffectiveModulePolicyService {
    pub async fn resolve_enabled(
        db: &DatabaseConnection,
        registry: &ModuleRegistry,
        tenant_id: uuid::Uuid,
    ) -> Result<HashSet<String>, PlatformCompositionError> {
        let manifest = PlatformCompositionService::active_manifest(db).await?;
        let mut enabled = registry
            .list()
            .into_iter()
            .filter(|module| registry.is_core(module.slug()))
            .map(|module| module.slug().to_string())
            .collect::<HashSet<_>>();

        for slug in manifest.settings.default_enabled {
            if registry.get(&slug).is_some() {
                enabled.insert(slug);
            }
        }

        let overrides = TenantModulesEntity::find()
            .filter(TenantModulesColumn::TenantId.eq(tenant_id))
            .all(db)
            .await?;

        for module in overrides {
            if module.enabled {
                enabled.insert(module.module_slug);
            } else {
                enabled.remove(&module.module_slug);
            }
        }

        Ok(enabled)
    }

    pub async fn list_enabled(
        db: &DatabaseConnection,
        registry: &ModuleRegistry,
        tenant_id: uuid::Uuid,
    ) -> Result<Vec<String>, PlatformCompositionError> {
        let mut modules = Self::resolve_enabled(db, registry, tenant_id)
            .await?
            .into_iter()
            .collect::<Vec<_>>();
        modules.sort();
        Ok(modules)
    }

    pub async fn is_enabled(
        db: &DatabaseConnection,
        registry: &ModuleRegistry,
        tenant_id: uuid::Uuid,
        module_slug: &str,
    ) -> Result<bool, PlatformCompositionError> {
        if registry.is_core(module_slug) {
            return Ok(true);
        }
        Ok(Self::resolve_enabled(db, registry, tenant_id)
            .await?
            .contains(module_slug))
    }
}
