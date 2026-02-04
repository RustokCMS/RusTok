use rustok_blog::BlogModule;
use rustok_core::{MigrationSource, RusToKModule};

#[test]
fn module_metadata() {
    let module = BlogModule;
    assert_eq!(module.slug(), "blog");
    assert_eq!(module.name(), "Blog");
    assert_eq!(module.description(), "Posts, Pages, Comments");
    assert_eq!(module.version(), env!("CARGO_PKG_VERSION"));
}

#[test]
fn module_migrations_empty() {
    let module = BlogModule;
    assert!(module.migrations().is_empty());
}
