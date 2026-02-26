pub mod dto;
pub mod entities;
pub mod error;
pub mod services;

pub use services::authz_mode::RbacAuthzMode;
pub use services::permission_authorizer::{
    authorize_all_permissions, authorize_any_permission, authorize_permission,
    AuthorizationDecision,
};
pub use services::permission_evaluator::{
    evaluate_all_permissions, evaluate_any_permission, evaluate_single_permission,
    PermissionEvaluation,
};
pub use services::permission_policy::{
    check_all_permissions, check_any_permission, check_permission, denied_reason_for_denial,
    has_effective_permission_in_set, missing_permissions, DeniedReasonKind, PermissionCheckOutcome,
};

pub use services::permission_resolver::{PermissionResolution, PermissionResolver};
pub use services::relation_permission_resolver::{
    invalidate_cached_permissions, resolve_permissions_from_relations,
    resolve_permissions_with_cache, PermissionCache, RelationPermissionStore,
};
pub use services::runtime_permission_resolver::{RoleAssignmentStore, RuntimePermissionResolver};
pub use services::shadow_decision::{
    compare_all_permissions, compare_any_permissions, compare_single_permission, ShadowDecision,
};

use async_trait::async_trait;
use rustok_core::module::{HealthStatus, MigrationSource, ModuleKind, RusToKModule};
use sea_orm_migration::MigrationTrait;

pub struct RbacModule;

impl MigrationSource for RbacModule {
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        Vec::new()
    }
}

#[async_trait]
impl RusToKModule for RbacModule {
    fn slug(&self) -> &'static str {
        "rbac"
    }

    fn name(&self) -> &'static str {
        "RBAC"
    }

    fn description(&self) -> &'static str {
        "Role-based access control helpers."
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn kind(&self) -> ModuleKind {
        ModuleKind::Core
    }

    async fn health(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
}
