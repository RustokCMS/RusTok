#![allow(elided_lifetimes_in_paths)]

pub use sea_orm_migration::prelude::*;

// Platform-core migrations — tables that are always present regardless of which
// optional modules are installed: tenants, users, sessions, roles, permissions,
// tenant-module registry, tenant locales, builds/releases, platform settings.
mod m20250101_000001_create_tenants;
mod m20250101_000002_create_users;
mod m20250101_000003_create_tenant_modules;
mod m20250101_000004_create_sessions;
mod m20250101_000005_create_roles_and_permissions;
mod m20250101_000006_add_metadata_to_tenants_and_users;
mod m20250130_000004_create_tenant_locales;
mod m20250201_000001_alter_status_to_enums;
mod m20250212_000001_create_builds_and_releases;
mod m20260211_000001_add_event_versioning;
mod m20260211_000002_create_sys_events;
mod m20260315_000001_create_user_field_definitions;
mod m20260316_000001_create_platform_settings;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // Platform-core migrations (always required).
        let mut all: Vec<Box<dyn MigrationTrait>> = vec![];
        all.push(Box::new(m20250101_000001_create_tenants::Migration));
        all.push(Box::new(m20250101_000002_create_users::Migration));
        all.push(Box::new(m20250101_000003_create_tenant_modules::Migration));
        all.push(Box::new(m20250130_000004_create_tenant_locales::Migration));
        all.push(Box::new(m20250101_000004_create_sessions::Migration));
        all.push(Box::new(
            m20250101_000005_create_roles_and_permissions::Migration,
        ));
        all.push(Box::new(
            m20250101_000006_add_metadata_to_tenants_and_users::Migration,
        ));
        all.push(Box::new(m20250201_000001_alter_status_to_enums::Migration));
        all.push(Box::new(
            m20250212_000001_create_builds_and_releases::Migration,
        ));
        all.push(Box::new(m20260211_000001_add_event_versioning::Migration));
        all.push(Box::new(m20260211_000002_create_sys_events::Migration));
        all.push(Box::new(
            m20260315_000001_create_user_field_definitions::Migration,
        ));
        all.push(Box::new(
            m20260316_000001_create_platform_settings::Migration,
        ));

        // Pull module-owned migrations from the domain crates and merge them into
        // the server migrator in chronological order.
        let mut module_migrations: Vec<Box<dyn MigrationTrait>> = Vec::new();
        module_migrations.extend(alloy_scripting::migrations::migrations());
        module_migrations.extend(rustok_auth::migrations::migrations());
        module_migrations.extend(rustok_commerce::migrations::migrations());
        module_migrations.extend(rustok_content::migrations::migrations());
        module_migrations.extend(rustok_forum::migrations::migrations());
        module_migrations.extend(rustok_index::migrations::migrations());
        module_migrations.extend(rustok_workflow::migrations::migrations());
        module_migrations.sort_by(|a, b| a.name().cmp(b.name()));

        all.extend(module_migrations);
        all
    }
}
