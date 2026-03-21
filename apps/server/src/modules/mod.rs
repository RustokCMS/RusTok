mod manifest;

use rustok_auth::AuthModule;
use rustok_blog::BlogModule;
use rustok_cache::CacheModule;
use rustok_commerce::CommerceModule;
use rustok_content::ContentModule;
use rustok_core::ModuleRegistry;
use rustok_email::EmailModule;
use rustok_forum::ForumModule;
use rustok_index::IndexModule;
use rustok_pages::PagesModule;
use rustok_rbac::RbacModule;
use rustok_tenant::TenantModule;

pub use manifest::{
    validate_registry_vs_manifest, BuildExecutionPlan, CatalogManifestModule, CatalogModuleVersion,
    DeploymentSurfaceContract, InstalledManifestModule, ManifestDiff, ManifestError,
    ManifestManager, ManifestModuleSpec, ModulesManifest,
};

pub fn build_registry() -> ModuleRegistry {
    let cache_module = CacheModule::new();
    ModuleRegistry::new()
        .register(AuthModule)
        .register(cache_module)
        .register(EmailModule)
        .register(IndexModule)
        .register(TenantModule)
        .register(RbacModule)
        .register(ContentModule)
        .register(CommerceModule)
        .register(BlogModule)
        .register(ForumModule)
        .register(PagesModule)
        .register(rustok_workflow::WorkflowModule)
}

#[cfg(test)]
mod contract_tests {
    use super::build_registry;
    use rustok_core::Permission;

    const AUTH_README: &str = include_str!("../../../../crates/rustok-auth/README.md");
    const CACHE_README: &str = include_str!("../../../../crates/rustok-cache/README.md");
    const EMAIL_README: &str = include_str!("../../../../crates/rustok-email/README.md");
    const INDEX_README: &str = include_str!("../../../../crates/rustok-index/README.md");
    const TENANT_README: &str = include_str!("../../../../crates/rustok-tenant/README.md");
    const RBAC_README: &str = include_str!("../../../../crates/rustok-rbac/README.md");
    const CONTENT_README: &str = include_str!("../../../../crates/rustok-content/README.md");
    const COMMERCE_README: &str = include_str!("../../../../crates/rustok-commerce/README.md");
    const BLOG_README: &str = include_str!("../../../../crates/rustok-blog/README.md");
    const FORUM_README: &str = include_str!("../../../../crates/rustok-forum/README.md");
    const PAGES_README: &str = include_str!("../../../../crates/rustok-pages/README.md");
    const WORKFLOW_README: &str = include_str!("../../../../crates/rustok-workflow/README.md");
    const FLEX_MUTATION: &str = include_str!("../graphql/flex/mutation.rs");

    #[test]
    fn registry_module_readmes_define_interactions_section() {
        for (slug, readme) in [
            ("auth", AUTH_README),
            ("cache", CACHE_README),
            ("email", EMAIL_README),
            ("index", INDEX_README),
            ("tenant", TENANT_README),
            ("rbac", RBAC_README),
            ("content", CONTENT_README),
            ("commerce", COMMERCE_README),
            ("blog", BLOG_README),
            ("forum", FORUM_README),
            ("pages", PAGES_README),
            ("workflow", WORKFLOW_README),
        ] {
            assert!(
                readme.contains("## Interactions"),
                "module `{slug}` README must contain `## Interactions`"
            );
        }
    }

    #[test]
    fn registry_modules_publish_expected_rbac_surface() {
        let registry = build_registry();
        let auth = registry.get("auth").expect("auth module");
        let tenant = registry.get("tenant").expect("tenant module");
        let rbac = registry.get("rbac").expect("rbac module");
        let blog = registry.get("blog").expect("blog module");
        let forum = registry.get("forum").expect("forum module");
        let pages = registry.get("pages").expect("pages module");
        let workflow = registry.get("workflow").expect("workflow module");

        assert!(auth.permissions().contains(&Permission::USERS_MANAGE));
        assert!(tenant.permissions().contains(&Permission::TENANTS_MANAGE));
        assert!(tenant.permissions().contains(&Permission::MODULES_MANAGE));
        assert!(rbac.permissions().contains(&Permission::SETTINGS_MANAGE));
        assert!(rbac.permissions().contains(&Permission::LOGS_READ));
        assert!(blog.permissions().contains(&Permission::BLOG_POSTS_MANAGE));
        assert!(forum
            .permissions()
            .contains(&Permission::FORUM_TOPICS_MANAGE));
        assert!(pages.permissions().contains(&Permission::PAGES_MANAGE));
        assert!(workflow
            .permissions()
            .contains(&Permission::WORKFLOWS_MANAGE));
    }

    #[test]
    fn registry_dependencies_match_runtime_contract() {
        let registry = build_registry();
        let pages = registry.get("pages").expect("pages module");
        let workflow = registry.get("workflow").expect("workflow module");

        assert_eq!(pages.dependencies(), &["content"]);
        assert!(workflow.dependencies().is_empty());
    }

    #[test]
    fn flex_mutation_uses_explicit_permissions_only() {
        assert!(FLEX_MUTATION.contains("FLEX_SCHEMAS_"));
        assert!(
            !FLEX_MUTATION.contains("infer_user_role_from_permissions"),
            "Flex authorization must not use role inference"
        );
        assert!(
            !FLEX_MUTATION.contains("UserRole::Admin")
                && !FLEX_MUTATION.contains("UserRole::SuperAdmin"),
            "Flex authorization must not hardcode role checks"
        );
    }
}
