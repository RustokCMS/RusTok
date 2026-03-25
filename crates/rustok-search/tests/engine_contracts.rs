use rustok_search::{SearchConnectorDescriptor, SearchEngineKind, SearchSettingsRecord};
use uuid::Uuid;

#[test]
fn postgres_is_default_connector() {
    let descriptor = SearchConnectorDescriptor::postgres_default();
    assert_eq!(descriptor.kind, SearchEngineKind::Postgres);
    assert!(descriptor.enabled);
    assert!(descriptor.default_engine);
}

#[test]
fn default_settings_fallback_to_postgres() {
    let tenant_id = Uuid::new_v4();
    let settings = SearchSettingsRecord::default_for_tenant(Some(tenant_id));

    assert_eq!(settings.tenant_id, Some(tenant_id));
    assert_eq!(settings.active_engine, SearchEngineKind::Postgres);
    assert_eq!(settings.fallback_engine, SearchEngineKind::Postgres);
}
