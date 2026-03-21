# RusTok вЂ” РЎРёСЃС‚РµРјР° РјРѕРґСѓР»РµР№: РїРѕР»РЅР°СЏ РєР°СЂС‚Р° Рё РїР»Р°РЅ

> **Р”Р°С‚Р°**: 2026-03-19
> **РќР°Р·РЅР°С‡РµРЅРёРµ**: РїРѕР»РЅР°СЏ РєР°СЂС‚Р° СЂРµР°Р»РёР·Р°С†РёРё вЂ” С‡С‚Рѕ СЃРґРµР»Р°РЅРѕ, РіРґРµ РґРѕРєСѓРјРµРЅС‚РёСЂРѕРІР°РЅРѕ,
> С‡С‚Рѕ РѕСЃС‚Р°Р»РѕСЃСЊ. РЎР»СѓР¶РёС‚ РѕСЃРЅРѕРІРѕР№ РґР»СЏ РїРµСЂРёРѕРґРёС‡РµСЃРєРѕР№ РІРµСЂРёС„РёРєР°С†РёРё РєРѕСЂСЂРµРєС‚РЅРѕСЃС‚Рё.
>
> Р›РµРіРµРЅРґР°: вњ… СЂРµР°Р»РёР·РѕРІР°РЅРѕ В· вљ пёЏ С‡Р°СЃС‚РёС‡РЅРѕ В· в¬њ РЅРµ РЅР°С‡Р°С‚Рѕ

---

## РЎРѕРґРµСЂР¶Р°РЅРёРµ

1. [РЎС‚Р°РЅРґР°СЂС‚ РјРѕРґСѓР»СЏ](#1-СЃС‚Р°РЅРґР°СЂС‚-РјРѕРґСѓР»СЏ)
2. [Tenant-level toggle](#2-tenant-level-toggle)
3. [Platform-level install/uninstall](#3-platform-level-installuninstall)
4. [Build pipeline](#4-build-pipeline)
5. [Marketplace РєР°С‚Р°Р»РѕРі](#5-marketplace-РєР°С‚Р°Р»РѕРі)
6. [Admin UI](#6-admin-ui)
7. [Р’РЅРµС€РЅРёР№ СЂРµРµСЃС‚СЂ Рё РїСѓР±Р»РёРєР°С†РёСЏ](#7-РІРЅРµС€РЅРёР№-СЂРµРµСЃС‚СЂ-Рё-РїСѓР±Р»РёРєР°С†РёСЏ)
8. [РђСЂС…РёС‚РµРєС‚СѓСЂРЅС‹Р№ РґРѕР»Рі](#8-Р°СЂС…РёС‚РµРєС‚СѓСЂРЅС‹Р№-РґРѕР»Рі)
9. [РџСЂРёРѕСЂРёС‚РµС‚ РЅРµР·Р°РІРµСЂС€С‘РЅРЅРѕРіРѕ](#9-РїСЂРёРѕСЂРёС‚РµС‚-РЅРµР·Р°РІРµСЂС€С‘РЅРЅРѕРіРѕ)

---

## 1. РЎС‚Р°РЅРґР°СЂС‚ РјРѕРґСѓР»СЏ

### вњ… `rustok-module.toml` вЂ” РјР°РЅРёС„РµСЃС‚ РјРѕРґСѓР»СЏ

РљР°Р¶РґС‹Р№ path-РјРѕРґСѓР»СЊ РѕР±СЏР·Р°РЅ РёРјРµС‚СЊ `rustok-module.toml` РІ РєРѕСЂРЅРµ crate.
РџР°СЂСЃРёС‚СЃСЏ РІ `ManifestManager::catalog_modules()` Рё `apply_module_package_manifest()`.

| РЎРµРєС†РёСЏ | Р§С‚Рѕ СЃРѕРґРµСЂР¶РёС‚ | РЎС‚Р°С‚СѓСЃ |
|---|---|---|
| `[module]` | slug, name, version, description, authors, license | вњ… РїР°СЂСЃРёС‚СЃСЏ |
| `[marketplace]` | icon, banner, screenshots, category, tags | вњ… РїР°СЂСЃРёС‚СЃСЏ |
| `[compatibility]` | rustok_min, rustok_max | вњ… РїР°СЂСЃРёС‚СЃСЏ |
| `[dependencies]` | depends_on СЃ version_req | вљ пёЏ slug РїСЂРѕРІРµСЂСЏРµС‚СЃСЏ, version_req РёРіРЅРѕСЂРёСЂСѓРµС‚СЃСЏ |
| `[conflicts]` | РЅРµСЃРѕРІРјРµСЃС‚РёРјС‹Рµ РјРѕРґСѓР»Рё | вљ пёЏ РїР°СЂСЃРёС‚СЃСЏ, РЅРѕ РЅРµ РїСЂРѕРІРµСЂСЏРµС‚СЃСЏ |
| `[crate]` | name, entry_type | вњ… РїР°СЂСЃРёС‚СЃСЏ |
| `[provides]` | migrations, permissions, events, admin_nav, storefront_slots, graphql | вњ… РїР°СЂСЃРёС‚СЃСЏ |
| `[settings]` | СЃС…РµРјР° РЅР°СЃС‚СЂРѕРµРє РјРѕРґСѓР»СЏ (type, default, min, max) | вљ пёЏ РїР°СЂСЃРёС‚СЃСЏ, РЅРѕ РЅРµС‚ API РґР»СЏ Р·Р°РїРёСЃРё |
| `[locales]` | supported, default | вњ… РїР°СЂСЃРёС‚СЃСЏ |

**Р¤Р°Р№Р»С‹**:
- `apps/server/src/modules/manifest.rs` вЂ” `apply_module_package_manifest()`

**Р”РѕРєСѓРјРµРЅС‚Р°С†РёСЏ**:
- `docs/modules/manifest.md`

---

### вњ… РЎС‚СЂСѓРєС‚СѓСЂР° С„Р°Р№Р»РѕРІ РјРѕРґСѓР»СЏ

```text
crates/rustok-{slug}/
в”њв”Ђв”Ђ rustok-module.toml       # РѕР±СЏР·Р°С‚РµР»СЊРЅРѕ РґР»СЏ path-РјРѕРґСѓР»РµР№
в”њв”Ђв”Ђ Cargo.toml
в”њв”Ђв”Ђ src/
в”‚   в”њв”Ђв”Ђ lib.rs               # impl RusToKModule (backend)
в”‚   в””в”Ђв”Ђ migrations/          # impl MigrationSource
в”‚       в”њв”Ђв”Ђ mod.rs
в”‚       в””в”Ђв”Ђ m20250101_*.rs
в”њв”Ђв”Ђ admin/                   # [NEW] Leptos-admin sub-crate
в””в”Ђв”Ђ storefront/              # [NEW] Leptos-storefront sub-crate
```

**Р”РѕРєСѓРјРµРЅС‚Р°С†РёСЏ**:
- `docs/architecture/modules.md`

---

### вњ… РљРѕРЅС‚СЂР°РєС‚ `RusToKModule`

```rust
pub trait RusToKModule: Send + Sync {
    fn slug(&self) -> &'static str;
    fn kind(&self) -> ModuleKind;          // Core | Optional
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>>;
    fn on_enable(&self, ctx: &AppContext) -> Result<()>;
    fn on_disable(&self, ctx: &AppContext) -> Result<()>;
    fn health(&self) -> ModuleHealth;
    fn event_listeners(&self) -> Vec<Box<dyn EventListener>>;
}
```

**Р¤Р°Р№Р»С‹**:
- `crates/rustok-core/src/module.rs`
- `crates/rustok-core/src/registry.rs`

**Р”РѕРєСѓРјРµРЅС‚Р°С†РёСЏ**:
- `docs/architecture/modules.md`

---

### вњ… Migration distribution

РљР°Р¶РґС‹Р№ РјРѕРґСѓР»СЊ РЅРµСЃС‘С‚ РјРёРіСЂР°С†РёРё РІРЅСѓС‚СЂРё СЃРІРѕРµРіРѕ crate (`src/migrations/`).
РџСЂРё СЃС‚Р°СЂС‚Рµ Р±РёРЅР°СЂРЅРёРєР° `registry.migrations()` СЃРѕР±РёСЂР°РµС‚ РјРёРіСЂР°С†РёРё РІСЃРµС… РјРѕРґСѓР»РµР№
Рё РїСЂРѕРіРѕРЅСЏРµС‚ РёС… Р°РІС‚РѕРјР°С‚РёС‡РµСЃРєРё. Р’СЂСѓС‡РЅСѓСЋ РґРѕР±Р°РІР»СЏС‚СЊ С„Р°Р№Р»С‹ РІ `apps/server/migration/` РЅРµ РЅСѓР¶РЅРѕ.

**Р¤Р°Р№Р»С‹**:
- `crates/rustok-*/src/migrations/` вЂ” РјРёРіСЂР°С†РёРё РєР°Р¶РґРѕРіРѕ РјРѕРґСѓР»СЏ
- `crates/rustok-core/src/registry.rs` вЂ” `ModuleRegistry::migrations()`

---

## 2. Tenant-level toggle

### вњ… РЎС…РµРјР° `tenant_modules`

```sql
CREATE TABLE tenant_modules (
  id         UUID PRIMARY KEY,
  tenant_id  UUID NOT NULL REFERENCES tenants(id),
  module_slug VARCHAR(64) NOT NULL,
  enabled    BOOLEAN NOT NULL DEFAULT true,
  settings   JSON NOT NULL DEFAULT '{}',
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  UNIQUE(tenant_id, module_slug)
)
```

**Р¤Р°Р№Р»С‹**:
- `apps/server/migration/src/m20250101_000003_create_tenant_modules.rs`
- `crates/rustok-tenant/src/entities/tenant_module.rs`
- `apps/server/src/models/tenant_modules.rs`

---

### вњ… `ModuleLifecycleService::toggle_module`

Flow:
1. slug в€€ `ModuleRegistry` в†’ РёРЅР°С‡Рµ `UnknownModule`
2. РЅРµ `ModuleKind::Core` в†’ РёРЅР°С‡Рµ `CoreModuleCannotBeDisabled`
3. `enabled=true`: РІСЃРµ `depends_on` РІРєР»СЋС‡РµРЅС‹ в†’ РёРЅР°С‡Рµ `MissingDependencies`
4. `enabled=false`: РЅРµС‚ Р·Р°РІРёСЃСЏС‰РёС… РѕС‚ РЅРµРіРѕ в†’ РёРЅР°С‡Рµ `HasDependents`
5. `BEGIN TRANSACTION` в†’ UPDATE `tenant_modules` в†’ `on_enable()` / `on_disable()`
6. РџСЂРё `HookFailed` вЂ” РѕС‚РєР°С‚ СЃРѕСЃС‚РѕСЏРЅРёСЏ РІ С‚СЂР°РЅР·Р°РєС†РёРё

**Р¤Р°Р№Р»С‹**:
- `apps/server/src/services/module_lifecycle.rs`

**РўРµСЃС‚С‹**:
- `apps/server/tests/module_lifecycle.rs`

---

### вњ… GraphQL `toggleModule`

```graphql
mutation {
  toggleModule(moduleSlug: "blog", enabled: true) {
    moduleSlug enabled settings
  }
}
```

**Р¤Р°Р№Р»С‹**:
- `apps/server/src/graphql/mutations.rs` вЂ” `async fn toggle_module`

---

### вњ… `EnabledModulesProvider` + `<ModuleGuard>` (Leptos)

`EnabledModulesProvider` Р·Р°РіСЂСѓР¶Р°РµС‚ РІРєР»СЋС‡С‘РЅРЅС‹Рµ РјРѕРґСѓР»Рё РїСЂРё СЃС‚Р°СЂС‚Рµ Рё РїСЂРµРґРѕСЃС‚Р°РІР»СЏРµС‚
РєРѕРЅС‚РµРєСЃС‚ РІСЃРµРјСѓ РїСЂРёР»РѕР¶РµРЅРёСЋ. `<ModuleGuard slug="blog">` СЂРµРЅРґРµСЂРёС‚ children С‚РѕР»СЊРєРѕ
РµСЃР»Рё РјРѕРґСѓР»СЊ РІРєР»СЋС‡С‘РЅ.

**Р¤Р°Р№Р»С‹**:
- `apps/admin/src/shared/context/enabled_modules.rs`

---

### вњ… Р¤РёР»СЊС‚СЂР°С†РёСЏ slot-РєРѕРјРїРѕРЅРµРЅС‚РѕРІ

`components_for_slot(slot_id, enabled_modules)` С„РёР»СЊС‚СЂСѓРµС‚ РІРёРґР¶РµС‚С‹ РІРёС‚СЂРёРЅС‹
РїРѕ РІРєР»СЋС‡С‘РЅРЅС‹Рј РјРѕРґСѓР»СЏРј С‚РµРЅР°РЅС‚Р° РїРµСЂРµРґ СЂРµРЅРґРµСЂРѕРј.

**Р¤Р°Р№Р»С‹**:
- `apps/storefront/src/modules/registry.rs`

---

### вљ пёЏ РќР°СЃС‚СЂРѕР№РєРё РјРѕРґСѓР»СЏ вЂ” РЅРµС‚ API Р·Р°РїРёСЃРё

РљРѕР»РѕРЅРєР° `settings JSON` РІ `tenant_modules` РµСЃС‚СЊ. `on_enable()` РјРѕР¶РµС‚ Р·Р°РїРёСЃР°С‚СЊ
РґРµС„РѕР»С‚С‹. РќРѕ РЅРµС‚ GraphQL РјСѓС‚Р°С†РёРё РґР»СЏ РѕР±РЅРѕРІР»РµРЅРёСЏ РЅР°СЃС‚СЂРѕРµРє С‡РµСЂРµР· UI.

**Р§С‚Рѕ РЅСѓР¶РЅРѕ**:
```graphql
mutation {
  updateModuleSettings(moduleSlug: "blog", settings: { postsPerPage: 20 }): TenantModule!
}
```

РЎРµСЂРІРµСЂРЅР°СЏ СЃС‚РѕСЂРѕРЅР° (`apps/server/src/graphql/mutations.rs`):
```rust
async fn update_module_settings(
    &self, ctx: &Context<'_>,
    module_slug: String,
    settings: serde_json::Value,
) -> Result<TenantModule> {
    // 1. РџСЂРѕРІРµСЂРёС‚СЊ С‡С‚Рѕ РјРѕРґСѓР»СЊ РІРєР»СЋС‡С‘РЅ РґР»СЏ С‚РµРЅР°РЅС‚Р°
    // 2. Р’Р°Р»РёРґРёСЂРѕРІР°С‚СЊ РїРѕ JSON Schema РёР· [settings] rustok-module.toml
    // 3. UPDATE tenant_modules SET settings = ?
}
```

UI: С„РѕСЂРјР° РёР· `[settings]` СЃРµРєС†РёРё `rustok-module.toml`, РІ РґРµС‚Р°Р»СЊРЅРѕР№ РїР°РЅРµР»Рё `/modules`.

---

## 3. Platform-level install/uninstall

### вњ… `ManifestManager`

```rust
ManifestManager::load()                     // РїР°СЂСЃРёС‚СЊ modules.toml
ManifestManager::save(manifest)             // СЃРѕС…СЂР°РЅРёС‚СЊ modules.toml
ManifestManager::validate(manifest)         // РїСЂРѕРІРµСЂРёС‚СЊ РіСЂР°С„ Р·Р°РІРёСЃРёРјРѕСЃС‚РµР№
ManifestManager::validate_with_registry()   // СЃРІРµСЂРёС‚СЊ СЃ ModuleRegistry
ManifestManager::install_builtin_module()   // РґРѕР±Р°РІРёС‚СЊ РІ modules.toml
ManifestManager::uninstall_module()         // СѓРґР°Р»РёС‚СЊ РёР· modules.toml
ManifestManager::upgrade_module()           // РѕР±РЅРѕРІРёС‚СЊ РІРµСЂСЃРёСЋ
ManifestManager::catalog_modules()          // РґР»СЏ MarketplaceCatalogService
ManifestManager::build_modules()            // РґР»СЏ BuildService
ManifestManager::build_execution_plan()     // РґР»СЏ BuildExecutor
```

**Р¤Р°Р№Р»С‹**:
- `apps/server/src/modules/manifest.rs`

**Р”РѕРєСѓРјРµРЅС‚Р°С†РёСЏ**:
- `docs/modules/manifest.md`
- `docs/architecture/modules.md`

---

### вњ… GraphQL РјСѓС‚Р°С†РёРё install/uninstall/upgrade/rollback

```graphql
installModule(slug: String!, version: String): BuildJob!
uninstallModule(slug: String!): BuildJob!
upgradeModule(slug: String!, version: String!): BuildJob!
rollbackBuild(buildId: ID!): BuildJob!
```

**Р¤Р°Р№Р»С‹**:
- `apps/server/src/graphql/mutations.rs`

---

### вљ пёЏ Semver-РІР°Р»РёРґР°С†РёСЏ Р·Р°РІРёСЃРёРјРѕСЃС‚РµР№ Рё РєРѕРЅС„Р»РёРєС‚РѕРІ

`ManifestManager::validate()` РїСЂРѕРІРµСЂСЏРµС‚ С‚РѕР»СЊРєРѕ С„Р°РєС‚ РЅР°Р»РёС‡РёСЏ slug РІ РјР°РЅРёС„РµСЃС‚Рµ.
Р”РёР°РїР°Р·РѕРЅС‹ РІРµСЂСЃРёР№ РІ `[dependencies]` (`>= 1.0.0`, `~2.0`) РЅРµ РїСЂРѕРІРµСЂСЏСЋС‚СЃСЏ.
РЎРµРєС†РёСЏ `[conflicts]` РїР°СЂСЃРёС‚СЃСЏ, РЅРѕ РЅРёРіРґРµ РЅРµ РїСЂРѕРІРµСЂСЏРµС‚СЃСЏ.

**Р§С‚Рѕ РЅСѓР¶РЅРѕ** РІ `apps/server/src/modules/manifest.rs`:
```rust
// Р”Р»СЏ РєР°Р¶РґРѕР№ Р·Р°РІРёСЃРёРјРѕСЃС‚Рё:
let req = semver::VersionReq::parse(&dep.version_req)?;
let installed = semver::Version::parse(&installed_spec.version)?;
if !req.matches(&installed) {
    return Err(IncompatibleDependencyVersion { ... });
}

// Р”Р»СЏ РєРѕРЅС„Р»РёРєС‚РѕРІ:
if manifest.modules.contains_key(&conflict_slug) {
    return Err(ConflictingModule { slug, conflicts_with: conflict_slug });
}
```

Р”РѕР±Р°РІРёС‚СЊ `semver = "1"` РІ `apps/server/Cargo.toml`.

---

## 4. Build pipeline

### вњ… `BuildService`

```rust
BuildService::request_build(request)   // СЃРѕР·РґР°С‚СЊ Build, С…РµС€РёСЂРѕРІР°С‚СЊ, РґРµРґСѓР±Р»РёСЂРѕРІР°С‚СЊ
BuildService::get_build(build_id)
BuildService::active_build()           // СЃР»РµРґСѓСЋС‰РёР№ queued/running
BuildService::running_build()
```

Р”РµРґСѓРїР»РёРєР°С†РёСЏ: РµСЃР»Рё РІ РѕС‡РµСЂРµРґРё СѓР¶Рµ РµСЃС‚СЊ build СЃ С‚Р°РєРёРј Р¶Рµ SHA-256 `modules_delta` вЂ”
РІРѕР·РІСЂР°С‰Р°РµС‚ СЃСѓС‰РµСЃС‚РІСѓСЋС‰РёР№ РІРјРµСЃС‚Рѕ СЃРѕР·РґР°РЅРёСЏ РЅРѕРІРѕРіРѕ.

**Р¤Р°Р№Р»С‹**:
- `apps/server/src/services/build_service.rs`
- `apps/server/src/models/build.rs` вЂ” `BuildStatus`, `BuildStage`, `DeploymentProfile`
- `apps/server/migration/src/m20250212_000001_create_builds_and_releases.rs`

---

### вњ… `BuildExecutor` вЂ” cargo build

Р’С‹РїРѕР»РЅСЏРµС‚ `cargo build -p rustok-server` СЃ feature flags РёР· СѓСЃС‚Р°РЅРѕРІР»РµРЅРЅС‹С… РјРѕРґСѓР»РµР№.
РћР±РЅРѕРІР»СЏРµС‚ `builds.stage` Рё `builds.progress` РїРѕ С…РѕРґСѓ РІС‹РїРѕР»РЅРµРЅРёСЏ.
РЎРѕР·РґР°С‘С‚ Р·Р°РїРёСЃСЊ РІ `releases` РїСЂРё СѓСЃРїРµС…Рµ.

**Р¤Р°Р№Р»С‹**:
- `apps/server/src/services/build_executor.rs`

**Env vars**:
- `RUSTOK_BUILD_CARGO_BIN` вЂ” РїСѓС‚СЊ Рє cargo (default: `cargo`)

---

### вњ… `buildProgress` GraphQL subscription

РСЃС‚РёРЅРЅС‹Р№ push С‡РµСЂРµР· `tokio::sync::broadcast` РєР°РЅР°Р».
`BuildEventHub` СЂР°СЃСЃС‹Р»Р°РµС‚ СЃРѕР±С‹С‚РёСЏ РїРѕ РјРµСЂРµ РІС‹РїРѕР»РЅРµРЅРёСЏ build executor'Р°.

```graphql
subscription {
  buildProgress(buildId: "...") { status stage progress logsUrl }
}
```

**Р¤Р°Р№Р»С‹**:
- `apps/server/src/graphql/subscriptions.rs`

---

### вњ… GraphQL queries РґР»СЏ builds

```graphql
activeBuild: BuildJob
buildHistory(limit: Int, offset: Int): [BuildJob!]!
```

**Р¤Р°Р№Р»С‹**:
- `apps/server/src/graphql/queries.rs`

---

### вњ… `rollback_build`

РџСЂРѕРІРµСЂСЏРµС‚ С†РµРїРѕС‡РєСѓ СЂРµР»РёР·РѕРІ С‡РµСЂРµР· `releases.previous_release_id`.
РџРѕРІС‚РѕСЂРЅРѕ Р°РєС‚РёРІРёСЂСѓРµС‚ РїСЂРµРґС‹РґСѓС‰РёР№ `Release`. РџРѕР»РЅРѕС†РµРЅРЅС‹Р№ РѕС‚РєР°С‚, РЅРµ РїСЂРѕСЃС‚Рѕ СЃРјРµРЅР° СЃС‚Р°С‚СѓСЃР°.

**Р¤Р°Р№Р»С‹**:
- `apps/server/src/graphql/mutations.rs` вЂ” `async fn rollback_build`
- `apps/server/src/models/release.rs`

---

### вљ пёЏ Docker deploy вЂ” РЅРµ СЂРµР°Р»РёР·РѕРІР°РЅ

`BuildExecutor` РІС‹РїРѕР»РЅСЏРµС‚ С‚РѕР»СЊРєРѕ `cargo build`. РџРѕСЃР»Рµ РєРѕРјРїРёР»СЏС†РёРё СЃРѕР·РґР°С‘С‚СЃСЏ
`Release` Р·Р°РїРёСЃСЊ, РЅРѕ `container_image` Рё `server_artifact_url` РѕСЃС‚Р°СЋС‚СЃСЏ РїСѓСЃС‚С‹РјРё.
`ReleaseStatus::Deploying` / `Active` РЅРµ РёСЃРїРѕР»СЊР·СѓСЋС‚СЃСЏ РїРѕ РЅР°Р·РЅР°С‡РµРЅРёСЋ.

**Р§С‚Рѕ РЅСѓР¶РЅРѕ** РІ `apps/server/src/services/build_executor.rs`:
```
Stage: Deploy (progress 85вЂ“99%)
  1. docker build -t {registry}/rustok-server:{release_id} .
  2. docker push
  3. Р—Р°РїРѕР»РЅРёС‚СЊ releases.container_image
  4. Rolling restart (monolith) | kubectl rollout (K8s)
```

**РќРѕРІС‹Рµ env vars**:
- `RUSTOK_BUILD_DOCKER_BIN` вЂ” РїСѓС‚СЊ Рє docker (default: `docker`)
- `RUSTOK_BUILD_REGISTRY` вЂ” registry URL
- `RUSTOK_DEPLOY_MODE` вЂ” `monolith` | `docker` | `k8s`

---

### вљ пёЏ Build progress UI вЂ” polling РІРјРµСЃС‚Рѕ subscription

РџСЂРѕРіСЂРµСЃСЃ-Р±Р°СЂ РІ `/modules` РѕР±РЅРѕРІР»СЏРµС‚СЃСЏ РѕРїСЂРѕСЃРѕРј РєР°Р¶РґС‹Рµ 5 СЃРµРєСѓРЅРґ
(`use_interval_fn(refresh_live_state, 5000)`).
Р‘СЌРєРµРЅРґ-subscription (`buildProgress`) СЂРµР°Р»РёР·РѕРІР°РЅР°, РЅРѕ UI Рє РЅРµР№ РЅРµ РїРѕРґРєР»СЋС‡С‘РЅ.

**Р§С‚Рѕ РЅСѓР¶РЅРѕ** РІ `apps/admin/src/features/modules/components/modules_list.rs`:
```rust
// Р—Р°РјРµРЅРёС‚СЊ:
use_interval_fn(refresh_live_state, 5000);
// РќР°:
let _sub = use_graphql_subscription::<BuildProgressSubscription>(
    BuildProgressSubscriptionVariables { build_id: active_build_id },
    move |event| set_build.set(Some(event.build_progress)),
);
```

`leptos-graphql` СѓР¶Рµ РїРѕРґРґРµСЂР¶РёРІР°РµС‚ subscriptions.

---

## 5. Marketplace РєР°С‚Р°Р»РѕРі

### вњ… `MarketplaceCatalogService` вЂ” provider chain

```
MarketplaceCatalogService
  в”њв”Ђ LocalManifestMarketplaceProvider   в†’ РІСЃС‚СЂРѕРµРЅРЅС‹Рµ path-РјРѕРґСѓР»Рё РёР· modules.toml
  в””в”Ђ RegistryMarketplaceProvider        в†’ РІРЅРµС€РЅРёР№ СЂРµРµСЃС‚СЂ (RUSTOK_MARKETPLACE_REGISTRY_URL)
       в””в”Ђ moka cache (TTL: RUSTOK_MARKETPLACE_REGISTRY_CACHE_TTL_SECS, default 60s)
```

РџСЂРё РЅРµРґРѕСЃС‚СѓРїРЅРѕСЃС‚Рё СЂРµРµСЃС‚СЂР° вЂ” graceful fallback РЅР° local-manifest.
Р”РµРґСѓРїР»РёРєР°С†РёСЏ: РїРѕР±РµР¶РґР°РµС‚ РїРµСЂРІС‹Р№ РїСЂРѕРІР°Р№РґРµСЂ.

**Р¤Р°Р№Р»С‹**:
- `apps/server/src/services/marketplace_catalog.rs`

**Env vars**:
- `RUSTOK_MARKETPLACE_REGISTRY_URL`
- `RUSTOK_MARKETPLACE_REGISTRY_TIMEOUT_MS` (default: 3000)
- `RUSTOK_MARKETPLACE_REGISTRY_CACHE_TTL_SECS` (default: 60)

---

### вњ… GraphQL `marketplace` + `marketplaceModule`

```graphql
marketplace(
  search: String
  category: String
  source: String          # "local" | "registry"
  installed: Boolean
  trust_level: String     # "first_party" | "third_party" | "community"
  compatible_only: Boolean
): [MarketplaceModule!]!

marketplaceModule(slug: String!): MarketplaceModule
```

**Р¤Р°Р№Р»С‹**:
- `apps/server/src/graphql/queries.rs`

---

### вњ… Deep-link `?module=slug`

Р’С‹Р±СЂР°РЅРЅС‹Р№ РјРѕРґСѓР»СЊ РІ РєР°С‚Р°Р»РѕРіРµ РѕС‚СЂР°Р¶Р°РµС‚СЃСЏ РІ URL (`/modules?module=blog`).
РџСЂСЏРјР°СЏ СЃСЃС‹Р»РєР° РѕС‚РєСЂС‹РІР°РµС‚ РґРµС‚Р°Р»СЊРЅСѓСЋ РїР°РЅРµР»СЊ Р±РµР· РїРµСЂРµС…РѕРґР°.

**Р¤Р°Р№Р»С‹**:
- `apps/admin/src/features/modules/components/modules_list.rs`

---

### в¬њ Р’РЅРµС€РЅРёР№ СЂРµРµСЃС‚СЂ `modules.rustok.dev`

`RegistryMarketplaceProvider` РґРµР»Р°РµС‚ HTTP-Р·Р°РїСЂРѕСЃС‹, РЅРѕ СЃР°Рј СЃРµСЂРІРёСЃ РЅРµ СЃСѓС‰РµСЃС‚РІСѓРµС‚.

**Scope V1** (read-only, first-party РјРѕРґСѓР»Рё):
```
modules.rustok.dev
в””в”Ђв”Ђ GET /v1/catalog в†’ [{ slug, name, version, ... }]
```
РџРѕР·РІРѕР»СЏРµС‚ РїСЂРѕРІРµСЂРёС‚СЊ РІРµСЃСЊ `RegistryMarketplaceProvider` в†’ AdminUI flow.

**Scope V2** (РїРѕР»РЅС‹Р№):
```
modules.rustok.dev
в”њв”Ђв”Ђ GraphQL API (РєР°С‚Р°Р»РѕРі, РІРµСЂСЃРёРё, РїРѕРёСЃРє, publish, yank)
в”њв”Ђв”Ђ Crate Storage (S3: .crate Р°СЂС…РёРІС‹ + checksums)
в””в”Ђв”Ђ Validation Pipeline (static в†’ audit в†’ compile в†’ test в†’ metadata)
```

**РђСѓС‚РµРЅС‚РёС„РёРєР°С†РёСЏ**: `docs/concepts/plan-oauth2-app-connections.md` (РџСЂРёР»РѕР¶РµРЅРёРµ A).

---

## 6. Admin UI

### вњ… РЎС‚СЂР°РЅРёС†Р° `/modules`

| Р­Р»РµРјРµРЅС‚ | РЎС‚Р°С‚СѓСЃ |
|---|---|
| РЎРїРёСЃРѕРє СѓСЃС‚Р°РЅРѕРІР»РµРЅРЅС‹С… РјРѕРґСѓР»РµР№ (`modules` query) | вњ… |
| РљР°С‚Р°Р»РѕРі РјР°СЂРєРµС‚РїР»РµР№СЃР° (`marketplace` query) | вњ… |
| Р¤РёР»СЊС‚СЂС‹: РїРѕРёСЃРє, РєР°С‚РµРіРѕСЂРёСЏ, trust level, compatibility | вњ… |
| Р”РµС‚Р°Р»СЊРЅР°СЏ РїР°РЅРµР»СЊ `marketplaceModule(slug)` | вњ… |
| Deep-link `?module=slug` | вњ… |
| Install / Uninstall РєРЅРѕРїРєРё в†’ `installModule` / `uninstallModule` | вњ… |
| Toggle switch в†’ `toggleModule` | вњ… |
| РЎРµРєС†РёРё: Installed / Marketplace / Updates | вњ… |
| РџСЂРѕРіСЂРµСЃСЃ-Р±Р°СЂ build (polling 5 СЃРµРє) | вњ… (РЅРѕ РЅРµ real-time) |
| РџСЂРѕРіСЂРµСЃСЃ-Р±Р°СЂ build (WebSocket subscription) | вљ пёЏ РЅРµ РїРѕРґРєР»СЋС‡С‘РЅ |
| "Update available" badge + `upgradeModule` РєРЅРѕРїРєР° | в¬њ РЅРµС‚ |
| Р¤РѕСЂРјР° РЅР°СЃС‚СЂРѕРµРє РјРѕРґСѓР»СЏ (`updateModuleSettings`) | в¬њ РЅРµС‚ |

---

## 7. Р’РЅРµС€РЅРёР№ СЂРµРµСЃС‚СЂ Рё РїСѓР±Р»РёРєР°С†РёСЏ

### в¬њ `rustok mod publish` CLI

```bash
rustok mod init          # РЁР°Р±Р»РѕРЅ РјРѕРґСѓР»СЏ СЃ rustok-module.toml
rustok mod validate      # Р›РѕРєР°Р»СЊРЅР°СЏ РїСЂРѕРІРµСЂРєР° РјР°РЅРёС„РµСЃС‚Р°
rustok mod test          # Validation pipeline Р»РѕРєР°Р»СЊРЅРѕ
rustok mod publish       # РћРїСѓР±Р»РёРєРѕРІР°С‚СЊ РІ СЂРµРµСЃС‚СЂ
rustok mod yank 1.2.0    # РћС‚РѕР·РІР°С‚СЊ РІРµСЂСЃРёСЋ
```

Р—Р°РІРёСЃРёС‚ РѕС‚ Рї. [РІРЅРµС€РЅРёР№ СЂРµРµСЃС‚СЂ](#-РІРЅРµС€РЅРёР№-СЂРµРµСЃС‚СЂ-modulesrustokdev).

---

### в¬њ Validation pipeline РґР»СЏ РїСѓР±Р»РёРєР°С†РёРё

| РЎС‚Р°РґРёСЏ | РџСЂРѕРІРµСЂРєРё |
|---|---|
| 1. Static | РјР°РЅРёС„РµСЃС‚ РІР°Р»РёРґРµРЅ, slug СѓРЅРёРєР°Р»РµРЅ, semver, license, locales/en.json |
| 2. Security | cargo-audit, РѕС‚СЃСѓС‚СЃС‚РІРёРµ unsafe Р±РµР· РѕР±РѕСЃРЅРѕРІР°РЅРёСЏ, РЅРµС‚ std::process::Command |
| 3. Compilation | РєРѕРјРїРёР»РёСЂСѓРµС‚СЃСЏ СЃ rustok_min..rustok_max |
| 4. Runtime | cargo test, РјРёРіСЂР°С†РёРё up/down РёРґРµРјРїРѕС‚РµРЅС‚РЅС‹, on_enable/on_disable |
| 5. Metadata | icon.svg РІР°Р»РёРґРµРЅ, description >= 20 СЃРёРјРІРѕР»РѕРІ, screenshots |

---

## 8. РђСЂС…РёС‚РµРєС‚СѓСЂРЅС‹Р№ РґРѕР»Рі

### вљ пёЏ GraphQL Рё REST РјРѕРґСѓР»РµР№ Р¶РёРІСѓС‚ РІ СЃРµСЂРІРµСЂРµ, Р° РЅРµ РІ РјРѕРґСѓР»СЊРЅС‹С… РєСЂРµР№С‚Р°С…

**РСЃС‚РѕСЂРёС‡РµСЃРєРѕРµ СЃРѕСЃС‚РѕСЏРЅРёРµ, СЃ РєРѕС‚РѕСЂРѕРіРѕ СЃС‚Р°СЂС‚РѕРІР°Р» РІС‹РЅРѕСЃ Р°РґР°РїС‚РµСЂРѕРІ**:

РР·РЅР°С‡Р°Р»СЊРЅРѕ GraphQL Рё REST Р°РґР°РїС‚РµСЂС‹ РґР»СЏ РєР°Р¶РґРѕРіРѕ РјРѕРґСѓР»СЏ Р¶РёР»Рё РІ `apps/server/`:

```
apps/server/src/
в”њв”Ђв”Ђ graphql/
в”‚   в”њв”Ђв”Ђ blog/        (~535 СЃС‚СЂРѕРє)   в†ђ Р·РЅР°Р» Рѕ rustok_blog::PostService
в”‚   в”њв”Ђв”Ђ content/     (~723 СЃС‚СЂРѕРє)   в†ђ Р·РЅР°Р» Рѕ rustok_content::NodeService
в”‚   в”њв”Ђв”Ђ commerce/    (~682 СЃС‚СЂРѕРє)   в†ђ Р·РЅР°Р» Рѕ rustok_commerce::CatalogService
в”‚   в”њв”Ђв”Ђ forum/       (~740 СЃС‚СЂРѕРє)   в†ђ Р·РЅР°Р» Рѕ rustok_forum::TopicService
в”‚   в”њв”Ђв”Ђ pages/       (~823 СЃС‚СЂРѕРє)   в†ђ Р·РЅР°Р» Рѕ rustok_pages::PageService
в”‚   в”њв”Ђв”Ђ workflow/    (~1071 СЃС‚СЂРѕРє)  в†ђ Р·РЅР°РµС‚ Рѕ rustok_workflow::WorkflowService
в”‚   в”њв”Ђв”Ђ alloy/       (~799 СЃС‚СЂРѕРє)   в†ђ Р·РЅР°РµС‚ Рѕ alloy_scripting::ScriptRegistry
в”‚   в””в”Ђв”Ђ media/       (~233 СЃС‚СЂРѕРє)   в†ђ Р·РЅР°РµС‚ Рѕ rustok_media::MediaService
в””в”Ђв”Ђ controllers/
    в”њв”Ђв”Ђ blog/        (~271 СЃС‚СЂРѕРє)   в†ђ С‚Рѕ Р¶Рµ СЃР°РјРѕРµ РґР»СЏ REST
    в”њв”Ђв”Ђ content/     (~199 СЃС‚СЂРѕРє)
    в”њв”Ђв”Ђ commerce/    (~1149 СЃС‚СЂРѕРє)
    в”њв”Ђв”Ђ forum/       (~638 СЃС‚СЂРѕРє)
    в”њв”Ђв”Ђ pages/       (~297 СЃС‚СЂРѕРє)
    в”њв”Ђв”Ђ workflow/    (~272 СЃС‚СЂРѕРє)
    в””в”Ђв”Ђ media/       (~191 СЃС‚СЂРѕРє)
```

**РџРѕС‡РµРјСѓ СЌС‚Рѕ РїСЂРѕР±Р»РµРјР°**:
РЎС‚РѕСЂРѕРЅРЅРёР№ РјРѕРґСѓР»СЊ РёР· РјР°СЂРєРµС‚РїР»РµР№СЃР° РЅРµ РјРѕР¶РµС‚ РґРѕР±Р°РІРёС‚СЊ СЃРІРѕР№ GraphQL/REST Р±РµР· РїСЂР°РІРєРё
`apps/server/`. Р­С‚Рѕ РЅР°СЂСѓС€Р°РµС‚ РєРѕРЅС†РµРїС†РёСЋ СЃР°РјРѕРґРѕСЃС‚Р°С‚РѕС‡РЅРѕРіРѕ РјРѕРґСѓР»СЏ.

**РџРѕС‡РµРјСѓ РЅРµ РІ `rustok-core`**:
`async-graphql`, `axum`, `loco_rs` вЂ” С‚СЏР¶С‘Р»С‹Рµ web-Р·Р°РІРёСЃРёРјРѕСЃС‚Рё. РћРЅРё РЅРµ РґРѕР»Р¶РЅС‹
РїРѕРїР°РґР°С‚СЊ РІ РґРѕРјРµРЅРЅРѕРµ СЏРґСЂРѕ. РњРѕРґСѓР»СЊРЅС‹Р№ РєСЂРµР№С‚ РґРѕР»Р¶РµРЅ РѕСЃС‚Р°РІР°С‚СЊСЃСЏ framework-agnostic.

**РЎС‚Р°С‚СѓСЃ РЅР° 2026-03-19**:

- вњ… РЎРѕР·РґР°РЅ РЅРѕРІС‹Р№ crate `crates/rustok-api/` РєР°Рє РѕР±С‰РёР№ API-СЃР»РѕР№ РјРµР¶РґСѓ `apps/server` Рё Р±СѓРґСѓС‰РёРјРё РјРѕРґСѓР»СЊРЅС‹РјРё web-Р°РґР°РїС‚РµСЂР°РјРё.
- вњ… Р’ `rustok-api` РІС‹РЅРµСЃРµРЅС‹ РѕР±С‰РёРµ РїСЂРёРјРёС‚РёРІС‹: `AuthContext`, `TenantContext`, `RequestContext`, `scope_matches`, `PageInfo`, `PaginationInput`, `GraphQLError`, `require_module_enabled`, `resolve_graphql_locale`.
- вњ… `apps/server` РїРµСЂРµРІРµРґС‘РЅ РЅР° СЃРѕРІРјРµСЃС‚РёРјС‹Рµ re-export/shim-С‚РѕС‡РєРё, С‡С‚РѕР±С‹ СЃСѓС‰РµСЃС‚РІСѓСЋС‰РёР№ РєРѕРґ РїСЂРѕРґРѕР»Р¶Р°Р» СЃРѕР±РёСЂР°С‚СЊСЃСЏ Р±РµР· РјР°СЃСЃРѕРІРѕР№ РїСЂР°РІРєРё РёРјРїРѕСЂС‚РѕРІ.
- вњ… РџРёР»РѕС‚РЅС‹Р№ РјРѕРґСѓР»СЊ `pages` РїРµСЂРµРЅРµСЃС‘РЅ: GraphQL Рё REST Р°РґР°РїС‚РµСЂС‹ С‚РµРїРµСЂСЊ Р¶РёРІСѓС‚ РІ `crates/rustok-pages`, Р° `apps/server` РґРµСЂР¶РёС‚ С‚РѕР»СЊРєРѕ С‚РѕРЅРєРёРµ re-export shim-С„Р°Р№Р»С‹.
- вњ… РЎР»РµРґРѕРј РїРµСЂРµРЅРµСЃС‘РЅ `blog`: РµРіРѕ GraphQL/REST Р°РґР°РїС‚РµСЂС‹ С‚РµРїРµСЂСЊ Р¶РёРІСѓС‚ РІ `crates/rustok-blog`, Р° СЃРµСЂРІРµСЂ РѕСЃС‚Р°РІР»СЏРµС‚ С‚РѕР»СЊРєРѕ shim/composition-root СЃР»РѕР№ Рё РјР°СЂС€СЂСѓС‚ health-check.
- вњ… Р—Р°С‚РµРј РїРµСЂРµРЅРµСЃС‘РЅ `forum`: GraphQL/REST Р°РґР°РїС‚РµСЂС‹ Рё connection helper С‚РµРїРµСЂСЊ Р¶РёРІСѓС‚ РІ `crates/rustok-forum`, Р° СЃРµСЂРІРµСЂ РѕСЃС‚Р°РІР»СЏРµС‚ С‚РѕР»СЊРєРѕ shim/composition-root СЃР»РѕР№ Рё РјР°СЂС€СЂСѓС‚ health-check.
- вњ… Р—Р°С‚РµРј РїРµСЂРµРЅРµСЃС‘РЅ `commerce`: GraphQL/REST Р°РґР°РїС‚РµСЂС‹ С‚РµРїРµСЂСЊ Р¶РёРІСѓС‚ РІ `crates/rustok-commerce`, permission-check РїРµСЂРµРІРµРґС‘РЅ РЅР° `AuthContext.permissions`, Р° `apps/server` РѕСЃС‚Р°РІР»СЏРµС‚ С‚РѕР»СЊРєРѕ shim/composition-root СЃР»РѕР№.
- вњ… Р—Р°С‚РµРј РїРµСЂРµРЅРµСЃС‘РЅ `content`: GraphQL/REST Р°РґР°РїС‚РµСЂС‹ С‚РµРїРµСЂСЊ Р¶РёРІСѓС‚ РІ `crates/rustok-content`, permission-check РІ REST/GraphQL mutation РїРµСЂРµРІРµРґС‘РЅ РЅР° `AuthContext.permissions`, Р° `apps/server` РѕСЃС‚Р°РІР»СЏРµС‚ С‚РѕР»СЊРєРѕ shim/composition-root СЃР»РѕР№ Рё РјР°СЂС€СЂСѓС‚ health-check.
- вњ… Р—Р°С‚РµРј РїРµСЂРµРЅРµСЃС‘РЅ `workflow`: GraphQL/REST Р°РґР°РїС‚РµСЂС‹ Рё webhook ingress С‚РµРїРµСЂСЊ Р¶РёРІСѓС‚ РІ `crates/rustok-workflow`, permission-check РїРµСЂРµРІРµРґС‘РЅ РЅР° `AuthContext.permissions`, Р° `apps/server` РѕСЃС‚Р°РІР»СЏРµС‚ С‚РѕР»СЊРєРѕ shim/composition-root СЃР»РѕР№.
- вњ… Р—Р°С‚РµРј РїРµСЂРµРЅРµСЃС‘РЅ `media`: GraphQL/REST Р°РґР°РїС‚РµСЂС‹ С‚РµРїРµСЂСЊ Р¶РёРІСѓС‚ РІ `crates/rustok-media`, REST РёСЃРїРѕР»СЊР·СѓРµС‚ РѕР±С‰РёР№ `AuthContext`, РјРµС‚СЂРёРєРё РѕСЃС‚Р°Р»РёСЃСЊ СЂСЏРґРѕРј СЃ transport-СЃР»РѕРµРј РјРѕРґСѓР»СЏ, Р° `apps/server` РѕСЃС‚Р°РІР»СЏРµС‚ С‚РѕР»СЊРєРѕ re-export shim.
- ? Затем исправлена позиция `alloy`: transport-слой вынесен в `crates/alloy`, а сам Alloy зафиксирован как module-agnostic capability вне runtime module registry; `apps/server` оставляет только composition-root shim.
- вњ… Server-only transport-С…РІРѕСЃС‚ Р°СЂС…РёС‚РµРєС‚СѓСЂРЅРѕРіРѕ РґРѕР»РіР° РґР»СЏ РјРѕРґСѓР»СЊРЅС‹С… GraphQL/REST Р°РґР°РїС‚РµСЂРѕРІ Р·Р°РєСЂС‹С‚.

**РџСЂР°РІРёР»СЊРЅРѕРµ СЂРµС€РµРЅРёРµ вЂ” РЅРѕРІС‹Р№ РєСЂРµР№С‚ `rustok-api`**:

```
crates/rustok-api/
  в””в”Ђв”Ђ src/
      в”њв”Ђв”Ђ context.rs       в†ђ TenantContext, AuthContext (РёР· apps/server/src/context/)
      в”њв”Ђв”Ђ graphql/
      в”‚   в”њв”Ђв”Ђ common.rs    в†ђ require_module_enabled, resolve_graphql_locale
      в”‚   в””в”Ђв”Ђ errors.rs    в†ђ GraphQLError
      в””в”Ђв”Ђ extractors/
          в””в”Ђв”Ђ rbac.rs      в†ђ Р±Р°Р·РѕРІС‹Рµ RBAC extractor С‚СЂРµР№С‚С‹
  # Р·Р°РІРёСЃРёС‚ РѕС‚: async-graphql, axum, loco_rs, rustok-core
```

РџРѕСЃР»Рµ СЌС‚РѕРіРѕ РєР°Р¶РґС‹Р№ РјРѕРґСѓР»СЊ РґРµСЂР¶РёС‚ GraphQL + REST Сѓ СЃРµР±СЏ:

```
crates/rustok-blog/src/
в”њв”Ђв”Ђ graphql/      в†ђ РїРµСЂРµРµС…Р°Р»Рѕ РёР· apps/server/src/graphql/blog/
в”‚   в”њв”Ђв”Ђ mod.rs    в†ђ pub struct BlogQuery; pub struct BlogMutation;
в”‚   в”њв”Ђв”Ђ query.rs
в”‚   в”њв”Ђв”Ђ mutation.rs
в”‚   в””в”Ђв”Ђ types.rs
в””в”Ђв”Ђ controllers/  в†ђ РїРµСЂРµРµС…Р°Р»Рѕ РёР· apps/server/src/controllers/blog/
    в”њв”Ђв”Ђ mod.rs    в†ђ pub fn routes() -> Routes
    в””в”Ђв”Ђ posts.rs
```

РЎРµСЂРІРµСЂ вЂ” С‚РѕР»СЊРєРѕ composition root:
```rust
// apps/server/src/graphql/schema.rs
#[cfg(feature = "mod-blog")]
use rustok_blog::graphql::{BlogMutation, BlogQuery};
```

**Р§С‚Рѕ РЅСѓР¶РЅРѕ СЃРґРµР»Р°С‚СЊ РґР°Р»СЊС€Рµ**:
1. Р—Р°РІРµСЂС€РёС‚СЊ С„Р°Р·Сѓ foundation: Р·Р°С„РёРєСЃРёСЂРѕРІР°С‚СЊ СЃРѕСЃС‚Р°РІ `rustok-api` РєР°Рє РµРґРёРЅСЃС‚РІРµРЅРЅРѕР№ С‚РѕС‡РєРё РґР»СЏ РѕР±С‰РёС… GraphQL/HTTP helper-РѕРІ Рё РЅРµ РІРѕР·РІСЂР°С‰Р°С‚СЊ СЌС‚Рё С‚РёРїС‹ РѕР±СЂР°С‚РЅРѕ РІ `apps/server`.
2. Зафиксировать split `alloy` + `alloy-scripting` как шаблон для module-agnostic capabilities, которые не должны попадать в runtime module registry даже при наличии собственного transport-слоя.
3. РћР±РЅРѕРІРёС‚СЊ `apps/server/src/graphql/schema.rs` Рё `apps/server/src/app.rs`, С‡С‚РѕР±С‹ РѕРЅРё Рё РґР°Р»СЊС€Рµ РѕСЃС‚Р°РІР°Р»РёСЃСЊ С‚РѕР»СЊРєРѕ composition-root СЃР»РѕРµРј РґР»СЏ РјРѕРґСѓР»СЊРЅС‹С… entry points.
4. РџРµСЂРµР№С‚Рё Рє СЃР»РµРґСѓСЋС‰РµРјСѓ РїР»Р°СЃС‚Сѓ Р°СЂС…РёС‚РµРєС‚СѓСЂРЅРѕРіРѕ РґРѕР»РіР°: `build.rs`-РєРѕРґРѕРіРµРЅРµСЂР°С†РёСЏ module entry points Рё РІС‹РЅРѕСЃ UI-СЃР»РѕСЏ РІ publishable module packages.

**РћС‚РєСЂС‹С‚С‹Р№ РІРѕРїСЂРѕСЃ**: СЃР»РµРґСѓСЋС‰РёР№ СЃСЂРµР· РґР»СЏ `rustok-api` вЂ” abstraction around `CurrentUser`/RBAC extractor-С‹ Рё РјРёРЅРёРјР°Р»СЊРЅС‹Р№ runtime-contract, РЅСѓР¶РЅС‹Р№ РјРѕРґСѓР»СЊРЅС‹Рј HTTP-РєРѕРЅС‚СЂРѕР»Р»РµСЂР°Рј Р±РµР· РїСЂРѕС‚Р°СЃРєРёРІР°РЅРёСЏ РІСЃРµРіРѕ `AppContext`.

---

### в¬њ РљРѕРґРѕРіРµРЅРµСЂР°С†РёСЏ СЂРµРіРёСЃС‚СЂР°С†РёРё РјРѕРґСѓР»РµР№ (`build.rs`)

**РўРµРєСѓС‰РµРµ СЃРѕСЃС‚РѕСЏРЅРёРµ** вЂ” С‚СЂРё РјРµСЃС‚Р° РїСЂРѕС€РёС‚С‹ РІСЂСѓС‡РЅСѓСЋ:

```rust
// 1. apps/server/src/modules/mod.rs вЂ” build_registry()
registry.register(BlogModule);     // в†ђ РєР°Р¶РґС‹Р№ РјРѕРґСѓР»СЊ СЏРІРЅРѕ
registry.register(CommerceModule); // в†ђ СЃС‚РѕСЂРѕРЅРЅРёР№ СЃСЋРґР° РЅРµ РїРѕРїР°РґС‘С‚

// 2. apps/server/src/graphql/schema.rs вЂ” СЃС‚Р°С‚РёС‡РµСЃРєРёР№ MergedObject
#[derive(MergedObject, Default)]
pub struct Query(
    #[cfg(feature = "mod-blog")] BlogQuery,   // в†ђ compile-time С‚РёРїС‹
    // PodcastQuery СЃС‚РѕСЂРѕРЅРЅРµРіРѕ РјРѕРґСѓР»СЏ СЃСЋРґР° РЅРµ РїРѕРїР°РґС‘С‚
);

// 3. apps/server/src/app.rs вЂ” РјР°СЂС€СЂСѓС‚С‹
.add_route(controllers::blog::routes()) // в†ђ СЃС‚РѕСЂРѕРЅРЅРёР№ РЅРµ РґРѕР±Р°РІРёС‚СЃСЏ
```

Р”Р»СЏ СѓСЃС‚Р°РЅРѕРІРєРё СЃС‚РѕСЂРѕРЅРЅРµРіРѕ РјРѕРґСѓР»СЏ РЅСѓР¶РЅРѕ РІСЂСѓС‡РЅСѓСЋ РјРµРЅСЏС‚СЊ РІСЃРµ С‚СЂРё С„Р°Р№Р»Р°.
Р­С‚Рѕ РЅРµ РјР°СЂРєРµС‚РїР»РµР№СЃ вЂ” СЌС‚Рѕ СЂСѓС‡РЅР°СЏ РёРЅС‚РµРіСЂР°С†РёСЏ.

**Р РµС€РµРЅРёРµ вЂ” `apps/server/build.rs`**:

`build.rs` С‡РёС‚Р°РµС‚ `modules.toml` Рё РіРµРЅРµСЂРёСЂСѓРµС‚ С‚СЂРё С„Р°Р№Р»Р°:

```
apps/server/src/generated/
в”њв”Ђв”Ђ registry.rs   в†ђ РІС‹Р·РѕРІС‹ register() РґР»СЏ РІСЃРµС… СѓСЃС‚Р°РЅРѕРІР»РµРЅРЅС‹С… РјРѕРґСѓР»РµР№
в”њв”Ђв”Ђ schema.rs     в†ђ MergedObject СЃ Query/Mutation РІСЃРµС… РјРѕРґСѓР»РµР№
в””в”Ђв”Ђ routes.rs     в†ђ add_route() РґР»СЏ РІСЃРµС… РјРѕРґСѓР»РµР№
```

РљР°Р¶РґС‹Р№ РјРѕРґСѓР»СЊ СЌРєСЃРїРѕСЂС‚РёСЂСѓРµС‚ СЃС‚Р°РЅРґР°СЂС‚РЅС‹Рµ С‚РѕС‡РєРё РІС…РѕРґР° (РїРѕСЃР»Рµ РїРµСЂРµРЅРѕСЃР° РІ РєСЂРµР№С‚):
```rust
// crates/rustok-podcast/src/lib.rs
pub mod graphql { pub struct PodcastQuery; pub struct PodcastMutation; }
pub mod controllers { pub fn routes() -> Routes { ... } }
```

`build.rs` Р·РЅР°РµС‚ С‡С‚Рѕ РіРµРЅРµСЂРёСЂРѕРІР°С‚СЊ РёР· `rustok-module.toml`:
```toml
[provides.graphql]
query_type = "PodcastQuery"
mutation_type = "PodcastMutation"

[provides.http]
routes_fn = "controllers::routes"
```

**РС‚РѕРі**: СЃРµСЂРІРµСЂ Р±РѕР»СЊС€Рµ РЅРёРєРѕРіРґР° РЅРµ С‚СЂРѕРіР°РµС‚СЃСЏ РІСЂСѓС‡РЅСѓСЋ РїСЂРё СѓСЃС‚Р°РЅРѕРІРєРµ РјРѕРґСѓР»СЏ.
`modules.toml` в†’ РєРѕРґРѕРіРµРЅРµСЂР°С†РёСЏ в†’ `cargo build` в†’ Р±РёРЅР°СЂРЅРёРє СЃ РЅРѕРІС‹Рј РјРѕРґСѓР»РµРј.

**Р—Р°РІРёСЃРёС‚ РѕС‚**: `rustok-api` + РїРµСЂРµРЅРѕСЃ GraphQL/REST РІ РјРѕРґСѓР»СЊРЅС‹Рµ РєСЂРµР№С‚С‹.

---

### в¬њ UI С‚РѕР¶Рµ РґРѕР»Р¶РµРЅ РїРµСЂРµСЃРѕР±РёСЂР°С‚СЊСЃСЏ вЂ” admin WASM Рё storefront WASM (Leptos)

**РљР»СЋС‡РµРІРѕР№ С„Р°РєС‚**: Leptos РєРѕРјРїРёР»РёСЂСѓРµС‚СЃСЏ РІ WASM. РљР°Рє СЃРµСЂРІРµСЂ в†’ Р±РёРЅР°СЂРЅРёРє,
С‚Р°Рє admin Рё storefront в†’ `.wasm`. Р”РёРЅР°РјРёС‡РµСЃРєРё РїРѕРґРіСЂСѓР·РёС‚СЊ РЅРѕРІС‹Р№ Rust-РєРѕРґ
РІ runtime РЅРµРІРѕР·РјРѕР¶РЅРѕ. Р›СЋР±РѕР№ РЅРѕРІС‹Р№ РјРѕРґСѓР»СЊ = РїРµСЂРµСЃР±РѕСЂРєР° WASM.

> [!IMPORTANT]
> **Next.js** (`apps/next-admin`, `apps/next-frontend`) **РЅРµ РІС…РѕРґРёС‚** РІ build pipeline
> РїСЂРё install/uninstall РјРѕРґСѓР»СЏ. РџРµСЂРµСЃР±РѕСЂРєР° Next.js вЂ” С‚РѕР»СЊРєРѕ РІСЂСѓС‡РЅСѓСЋ.
> РђРІС‚Рѕ-СѓСЃС‚Р°РЅРѕРІРєР° С‡РµСЂРµР· marketplace СЂР°Р±РѕС‚Р°РµС‚ РёСЃРєР»СЋС‡РёС‚РµР»СЊРЅРѕ РґР»СЏ **Leptos**-СЃС‚РµРєР°.

**Р§С‚Рѕ РїСЂРѕС€РёС‚Рѕ РІСЂСѓС‡РЅСѓСЋ РІ admin** (Leptos):

```
apps/admin/src/
в”њв”Ђв”Ђ pages/mod.rs         в†ђ mod workflows; mod workflow_detail;  (СЏРІРЅС‹Рµ РѕР±СЉСЏРІР»РµРЅРёСЏ)
в”њв”Ђв”Ђ pages/workflows.rs   в†ђ СЃС‚СЂР°РЅРёС†Р° Workflows
в”њв”Ђв”Ђ pages/workflow_detail.rs
в”њв”Ђв”Ђ features/workflow/   в†ђ РєРѕРјРїРѕРЅРµРЅС‚С‹ workflow (400+ СЃС‚СЂРѕРє)
в””в”Ђв”Ђ app/router.rs        в†ђ Route path="/workflows" view=Workflows
```

Р”Р»СЏ СЃС‚РѕСЂРѕРЅРЅРµРіРѕ `rustok-podcast`: РЅРµС‚ РЅРё `/podcasts` РјР°СЂС€СЂСѓС‚Р°,
РЅРё `PodcastsPage`, РЅРё `features/podcast/`.

**Р§С‚Рѕ РґРёРЅР°РјРёС‡РЅРѕ** (СЃР»РѕС‚-СЃРёСЃС‚РµРјР°):
- `AdminSlot::NavItem` вЂ” nav items СЂРµРіРёСЃС‚СЂРёСЂСѓСЋС‚СЃСЏ С‡РµСЂРµР· `register_component()` вњ…
- `AdminSlot::DashboardSection` вЂ” РІРёРґР¶РµС‚С‹ РґР°С€Р±РѕСЂРґР° вњ…
- `StorefrontSlot::*` вЂ” СЃР»РѕС‚С‹ РІРёС‚СЂРёРЅС‹ вњ…

РќРѕ РґР°Р¶Рµ РґР»СЏ СЃР»РѕС‚РѕРІ: С„СѓРЅРєС†РёСЏ `render: fn() -> AnyView` РґРѕР»Р¶РЅР° Р±С‹С‚СЊ
**СЃРєРѕРјРїРёР»РёСЂРѕРІР°РЅР° РІ WASM Р·Р°СЂР°РЅРµРµ**. РЎР»РѕС‚-СЃРёСЃС‚РµРјР° СѓРїСЂР°РІР»СЏРµС‚ РІРёРґРёРјРѕСЃС‚СЊСЋ,
Р° РЅРµ Р·Р°РіСЂСѓР·РєРѕР№ РєРѕРґР°.

**Р§С‚Рѕ РЅСѓР¶РЅРѕ СЃРґРµР»Р°С‚СЊ**:

1. **UI РІ РїРѕРґРїР°РїРєР°С… РјРѕРґСѓР»СЏ** (РЅР°РїСЂ. `crates/rustok-workflow/admin/`):

```text
crates/rustok-workflow/
в”њв”Ђв”Ђ Cargo.toml          # rustok-workflow (backend)
в”њв”Ђв”Ђ src/                # backend logic
в”њв”Ђв”Ђ admin/
в”‚   в”њв”Ђв”Ђ Cargo.toml      # rustok-workflow-admin (publishable)
в”‚   в””в”Ђв”Ђ src/            # Leptos components & register_routes()
в””в”Ђв”Ђ storefront/
    в”њв”Ђв”Ђ Cargo.toml      # rustok-workflow-storefront (publishable)
    в””в”Ђв”Ђ src/            # Leptos SSR components
```

2. **`apps/admin/build.rs`** РіРµРЅРµСЂРёСЂСѓРµС‚ РёР· `modules.toml`:
```rust
// generated/routes.rs
<Route path=path!("/workflows") view=rustok_workflow::ui::admin::WorkflowsPage />
<Route path=path!("/workflows/:id") view=rustok_workflow::ui::admin::WorkflowDetailPage />
```

3. **`apps/storefront/build.rs`** РіРµРЅРµСЂРёСЂСѓРµС‚ РІС‹Р·РѕРІС‹ `register_component()`:
```rust
// generated/registrations.rs
rustok_workflow::storefront::register_slot_components();
```

4. **`BuildExecutor`** СЃРѕР±РёСЂР°РµС‚ С‚СЂРё Р°СЂС‚РµС„Р°РєС‚Р°:
```
cargo build -p rustok-server          // Р±РёРЅР°СЂРЅРёРє СЃРµСЂРІРµСЂР° (СЃРµР№С‡Р°СЃ вњ…)
wasm-pack build apps/admin            // admin WASM         (в¬њ РЅРµ СЂРµР°Р»РёР·РѕРІР°РЅРѕ)
cargo build -p rustok-storefront      // storefront         (в¬њ РЅРµ СЂРµР°Р»РёР·РѕРІР°РЅРѕ)
```

**`rustok-module.toml`** РѕР±СЉСЏРІР»СЏРµС‚ UI С‚РѕС‡РєРё РІС…РѕРґР°:
```toml
[provides.admin_ui]
leptos_crate  = "rustok-workflow-admin"
routes_fn     = "register_routes"
components_fn = "register_components"

[provides.storefront_ui]
leptos_crate  = "rustok-workflow-storefront"
components_fn = "register_slot_components"
```

**Р—Р°РІРёСЃРёС‚ РѕС‚**: РєРѕРґРѕРіРµРЅРµСЂР°С†РёРё `build.rs` (Рї.6 РІС‹С€Рµ).

---

## 9. РџСЂРёРѕСЂРёС‚РµС‚ РЅРµР·Р°РІРµСЂС€С‘РЅРЅРѕРіРѕ

| # | Р—Р°РґР°С‡Р° | РЎР»РѕР¶РЅРѕСЃС‚СЊ | Р¦РµРЅРЅРѕСЃС‚СЊ |
|---|---|---|---|
| **1** | **Audit РґРѕРєСѓРјРµРЅС‚Р°С†РёРё** вЂ” РїСЂРёРІРµСЃС‚Рё РІ СЃРѕРѕС‚РІРµС‚СЃС‚РІРёРµ СЃ СЂРµС€РµРЅРёСЏРјРё РѕС‚ 2026-03-17 | РЎСЂРµРґРЅСЏСЏ | РљСЂРёС‚РёС‡РµСЃРєР°СЏ вЂ” РґРѕРєСѓРјРµРЅС‚Р°С†РёСЏ РЅР°РїСЂСЏРјСѓСЋ РѕРїСЂРµРґРµР»СЏРµС‚ РєР°Рє СЂР°Р·СЂР°Р±Р°С‚С‹РІР°СЋС‚СЃСЏ РјРѕРґСѓР»Рё Рё РјР°СЂРєРµС‚РїР»РµР№СЃ |
| 2 | Semver + conflict РІР°Р»РёРґР°С†РёСЏ | РњР°Р»Р°СЏ | Р’С‹СЃРѕРєР°СЏ вЂ” Р·Р°С‰РёС‚Р° РѕС‚ broken installs |
| 3 | `updateModuleSettings` РјСѓС‚Р°С†РёСЏ | РњР°Р»Р°СЏ | Р’С‹СЃРѕРєР°СЏ вЂ” `[settings]` СѓР¶Рµ РІРµР·РґРµ РµСЃС‚СЊ |
| 4 | Build progress в†’ subscription | РњР°Р»Р°СЏ | РЎСЂРµРґРЅСЏСЏ вЂ” UX, РёРЅС„СЂР°СЃС‚СЂСѓРєС‚СѓСЂР° СѓР¶Рµ РµСЃС‚СЊ |
| 5 | Docker deploy РІ BuildExecutor | РЎСЂРµРґРЅСЏСЏ | Р’С‹СЃРѕРєР°СЏ вЂ” Р±РµР· СЌС‚РѕРіРѕ install РЅРµ prod-ready |
| 6 | `rustok-api` foundation + РїРµСЂРµРЅРѕСЃ GraphQL/REST РІ РєСЂРµР№С‚С‹ | Р‘РѕР»СЊС€Р°СЏ | РљСЂРёС‚РёС‡РµСЃРєР°СЏ вЂ” Р±Р»РѕРєРёСЂСѓРµС‚ 3rd party |
| 7 | РџРµСЂРµРЅРѕСЃ UI (admin/storefront) РІ РјРѕРґСѓР»СЊРЅС‹Рµ РєСЂРµР№С‚С‹ | Р‘РѕР»СЊС€Р°СЏ | РљСЂРёС‚РёС‡РµСЃРєР°СЏ вЂ” Р±Р»РѕРєРёСЂСѓРµС‚ 3rd party |
| 8 | `build.rs` РєРѕРґРѕРіРµРЅРµСЂР°С†РёСЏ (СЃРµСЂРІРµСЂ + admin + storefront) | Р‘РѕР»СЊС€Р°СЏ | РљСЂРёС‚РёС‡РµСЃРєР°СЏ вЂ” Р±Р»РѕРєРёСЂСѓРµС‚ 3rd party |
| 9 | `BuildExecutor`: СЃР±РѕСЂРєР° admin WASM + storefront | РЎСЂРµРґРЅСЏСЏ | РљСЂРёС‚РёС‡РµСЃРєР°СЏ вЂ” Р±РµР· СЌС‚РѕРіРѕ install РЅРµРїРѕР»РЅС‹Р№ |
| 10 | Р’РЅРµС€РЅРёР№ СЂРµРµСЃС‚СЂ V1 (read-only) | Р‘РѕР»СЊС€Р°СЏ | Р’С‹СЃРѕРєР°СЏ вЂ” РѕСЃРЅРѕРІР° marketplace |
| 11 | Р’РЅРµС€РЅРёР№ СЂРµРµСЃС‚СЂ V2 + publish | РћС‡РµРЅСЊ Р±РѕР»СЊС€Р°СЏ | РЎСЂРµРґРЅСЏСЏ вЂ” РЅСѓР¶РµРЅ С‚РѕР»СЊРєРѕ РґР»СЏ 3rd party |

> РџРї. 6, 7, 8, 9 вЂ” РµРґРёРЅС‹Р№ Р±Р»РѕРє. Р’СЃРµ С‡РµС‚С‹СЂРµ РЅСѓР¶РЅС‹ РІРјРµСЃС‚Рµ, С‡С‚РѕР±С‹
> СЃС‚РѕСЂРѕРЅРЅРёР№ РјРѕРґСѓР»СЊ РїРѕР»РЅРѕС†РµРЅРЅРѕ Р·Р°СЂР°Р±РѕС‚Р°Р» (СЃРµСЂРІРµСЂ + UI).

### Р§С‚Рѕ РёР·РјРµРЅРёР»РѕСЃСЊ (2026-03-18) вЂ” С„РёРЅР°Р»СЊРЅС‹Р№ РѕСЂРёРµРЅС‚РёСЂ

РџСЂРёРЅСЏС‚С‹Рµ СЂРµС€РµРЅРёСЏ РїРѕ СЃС‚СЂСѓРєС‚СѓСЂРµ UI:

1. **Leptos UI** вЂ” РІС‹РЅРµСЃРµРЅ РІ РѕС‚РґРµР»СЊРЅС‹Рµ publishable СЃСѓР±-РєСЂРµР№С‚С‹ `admin/` Рё `storefront/` РІРЅСѓС‚СЂРё РїР°РїРєРё РјРѕРґСѓР»СЏ. Р­С‚Рѕ РїРѕР·РІРѕР»СЏРµС‚ РїСѓР±Р»РёРєРѕРІР°С‚СЊ РёС… РІ crates.io Рё РјРёРЅРёРјРёР·РёСЂРѕРІР°С‚СЊ Р·Р°РІРёСЃРёРјРѕСЃС‚Рё РѕСЃРЅРѕРІРЅРѕРіРѕ Р±РµРєРµРЅРґ-РєСЂРµР№С‚Р°.
2. **Next.js UI** вЂ” РїРµСЂРµРЅС‘СЃС‘РЅ РІ `apps/*/packages/<module>/` РІ РІРёРґРµ Р»РѕРєР°Р»СЊРЅС‹С… npm-РїР°РєРµС‚РѕРІ. Р­С‚Рѕ РѕР±РµСЃРїРµС‡РёРІР°РµС‚ РёР·РѕР»СЏС†РёСЋ РєРѕРґР° РјРѕРґСѓР»РµР№ РѕС‚ РѕСЃРЅРѕРІРЅРѕРіРѕ РїСЂРёР»РѕР¶РµРЅРёСЏ РїСЂРё СЃРѕС…СЂР°РЅРµРЅРёРё РІРѕР·РјРѕР¶РЅРѕСЃС‚Рё РїСѓР±Р»РёРєР°С†РёРё РІ npm.
3. **РђРІС‚Рѕ-РґРµРїР»РѕР№** вЂ” СЂР°Р±РѕС‚Р°РµС‚ **С‚РѕР»СЊРєРѕ РґР»СЏ Leptos** С‡РµСЂРµР· BuildExecutor. Next.js РїСЂРёР»РѕР¶РµРЅРёСЏ С‚СЂРµР±СѓСЋС‚ СЂСѓС‡РЅРѕР№ СЃР±РѕСЂРєРё/РѕР±РЅРѕРІР»РµРЅРёСЏ `package.json`.
4. **РњР°РЅРёС„РµСЃС‚** вЂ” `rustok-module.toml` С‚РµРїРµСЂСЊ РґРѕР»Р¶РµРЅ СЏРІРЅРѕ СѓРєР°Р·С‹РІР°С‚СЊ РёРјРµРЅР° UI-РїР°РєРµС‚РѕРІ (`leptos_crate`, `next_package`).

РЎРІСЏР·Р°РЅРЅС‹Р№ ADR: `DECISIONS/2026-03-17-dual-ui-strategy-next-batteries-included.md` (РѕР±РЅРѕРІР»РµРЅ 2026-03-18).



