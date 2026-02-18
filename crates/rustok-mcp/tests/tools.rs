use async_trait::async_trait;
use sea_orm_migration::MigrationTrait;

use rustok_core::module::{MigrationSource, RusToKModule};
use rustok_core::registry::ModuleRegistry;
use rustok_mcp::tools::{
    list_modules, module_details, module_exists, McpState, ModuleLookupRequest,
};

struct DemoModule;

impl MigrationSource for DemoModule {
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        Vec::new()
    }
}

#[async_trait]
impl RusToKModule for DemoModule {
    fn slug(&self) -> &'static str {
        "demo"
    }

    fn name(&self) -> &'static str {
        "Demo"
    }

    fn description(&self) -> &'static str {
        "Demo module"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn dependencies(&self) -> &[&'static str] {
        &["core"]
    }
}

#[tokio::test]
async fn list_modules_returns_registered_metadata() {
    let registry = ModuleRegistry::new().register(DemoModule);
    let state = Box::leak(Box::new(McpState { registry }));

    let response = list_modules(state).await;

    assert_eq!(response.modules.len(), 1);
    assert_eq!(response.modules[0].slug, "demo");
    assert_eq!(response.modules[0].name, "Demo");
    assert_eq!(response.modules[0].description, "Demo module");
    assert_eq!(response.modules[0].version, "0.1.0");
    assert_eq!(response.modules[0].dependencies, vec!["core"]);
}

#[tokio::test]
async fn module_exists_checks_registry() {
    let registry = ModuleRegistry::new().register(DemoModule);
    let state = Box::leak(Box::new(McpState { registry }));

    let response = module_exists(
        state,
        ModuleLookupRequest {
            slug: "demo".to_string(),
        },
    )
    .await;

    assert!(response.exists);
    assert_eq!(response.slug, "demo");
}

#[tokio::test]
async fn module_details_returns_module_data() {
    let registry = ModuleRegistry::new().register(DemoModule);
    let state = Box::leak(Box::new(McpState { registry }));

    let response = module_details(
        state,
        ModuleLookupRequest {
            slug: "demo".to_string(),
        },
    )
    .await;

    let module = response.module.expect("module details");
    assert_eq!(response.slug, "demo");
    assert_eq!(module.slug, "demo");
    assert_eq!(module.name, "Demo");
    assert_eq!(module.description, "Demo module");
    assert_eq!(module.version, "0.1.0");
    assert_eq!(module.dependencies, vec!["core"]);
}

#[tokio::test]
async fn module_details_returns_none_for_unknown_slug() {
    let registry = ModuleRegistry::new().register(DemoModule);
    let state = Box::leak(Box::new(McpState { registry }));

    let response = module_details(
        state,
        ModuleLookupRequest {
            slug: "missing".to_string(),
        },
    )
    .await;

    assert!(response.module.is_none());
    assert_eq!(response.slug, "missing");
}
