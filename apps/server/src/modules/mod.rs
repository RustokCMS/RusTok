mod alloy;
mod manifest;

use rustok_blog::BlogModule;
use rustok_commerce::CommerceModule;
use rustok_content::ContentModule;
use rustok_core::ModuleRegistry;
use rustok_forum::ForumModule;
use rustok_index::IndexModule;
use rustok_pages::PagesModule;
use rustok_rbac::RbacModule;
use rustok_tenant::TenantModule;

pub use alloy::AlloyModule;

pub fn build_registry() -> ModuleRegistry {
    ModuleRegistry::new()
        // --- Core modules: always active, cannot be disabled per-tenant ---
        .register(IndexModule)
        .register(TenantModule)
        .register(RbacModule)
        // --- Optional domain modules: per-tenant toggle ---
        .register(ContentModule)
        .register(CommerceModule)
        .register(BlogModule)
        .register(ForumModule)
        .register(PagesModule)
        .register(AlloyModule)
}

pub use manifest::validate_registry_vs_manifest;
