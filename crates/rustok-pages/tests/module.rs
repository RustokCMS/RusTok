use rustok_core::{MigrationSource, RusToKModule};
use rustok_pages::PagesModule;

#[test]
fn module_metadata() {
    let module = PagesModule;
    assert_eq!(module.slug(), "pages");
    assert_eq!(module.name(), "Pages");
    assert_eq!(module.description(), "Pages and menus domain logic.");
    assert_eq!(module.version(), env!("CARGO_PKG_VERSION"));
}

#[test]
fn module_migrations_empty() {
    let module = PagesModule;
    assert!(module.migrations().is_empty());
}
