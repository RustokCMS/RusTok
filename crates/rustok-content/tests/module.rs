use rustok_content::ContentModule;
use rustok_core::{MigrationSource, RusToKModule};

#[test]
fn module_metadata() {
    let module = ContentModule;
    assert_eq!(module.slug(), "content");
    assert_eq!(module.name(), "Content");
    assert_eq!(
        module.description(),
        "Core CMS Module (Nodes, Bodies, Categories)"
    );
    assert_eq!(module.version(), env!("CARGO_PKG_VERSION"));
}

#[test]
fn module_migrations_empty() {
    let module = ContentModule;
    assert!(module.migrations().is_empty());
}
