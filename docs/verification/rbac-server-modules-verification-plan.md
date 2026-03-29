# РџР»Р°РЅ rolling-РІРµСЂРёС„РёРєР°С†РёРё RBAC РґР»СЏ server Рё runtime-РјРѕРґСѓР»РµР№

- **РЎС‚Р°С‚СѓСЃ:** РђРєС‚СѓР°Р»РёР·РёСЂРѕРІР°РЅРЅС‹Р№ rolling-С‡РµРєР»РёСЃС‚
- **Р РµР¶РёРј:** РџРѕРІС‚РѕСЂСЏРµРјР°СЏ С‚РѕС‡РµС‡РЅР°СЏ РІРµСЂРёС„РёРєР°С†РёСЏ
- **РџРѕР»РЅР°СЏ С‡Р°СЃС‚РѕС‚Р°:** 1 СЂР°Р· РІ РЅРµРґРµР»СЋ Рё РїРѕСЃР»Рµ Р»СЋР±С‹С… РёР·РјРµРЅРµРЅРёР№ РІ RBAC, server transport, module contracts РёР»Рё capability-boundaries
- **Р¦РµР»СЊ:** Р”РµСЂР¶Р°С‚СЊ РІ СЃРѕРіР»Р°СЃРѕРІР°РЅРЅРѕРј СЃРѕСЃС‚РѕСЏРЅРёРё live RBAC contract РјРµР¶РґСѓ `apps/server`, `rustok-rbac`, runtime-РјРѕРґСѓР»СЏРјРё РёР· `build_registry()` Рё capability-РїРѕРІРµСЂС…РЅРѕСЃС‚СЏРјРё

---

## 1. РћР±СЉРµРєС‚ РїСЂРѕРІРµСЂРєРё

### 1.1 Server surfaces

- `apps/server`
- REST RBAC extractors Рё guards
- GraphQL queries / mutations / subscriptions
- `RbacService`
- permission-aware `SecurityContext`

### 1.2 Runtime modules РёР· `build_registry()`

- `auth`
- `cache`
- `email`
- `index`
- `outbox`
- `tenant`
- `rbac`
- `content`
- `commerce`
- `blog`
- `forum`
- `pages`
- `media`
- `workflow`

### 1.3 Capability surfaces, РїСЂРѕРІРµСЂСЏРµРјС‹Рµ РІРјРµСЃС‚Рµ СЃ RBAC

- `alloy`
- `alloy`
- `flex`
- `rustok-mcp`

---

## 2. РСЃС‚РѕС‡РЅРёРєРё РёСЃС‚РёРЅС‹

РљР°Р¶РґС‹Р№ РїСЂРѕС…РѕРґ РґРѕР»Р¶РµРЅ СЃС‡РёС‚Р°С‚СЊ РєР°РЅРѕРЅРёС‡РµСЃРєРёРјРё СЃР»РµРґСѓСЋС‰РёРµ РєРѕРЅС‚СЂР°РєС‚С‹:

1. `RusToKModule::permissions()`
2. `RusToKModule::dependencies()`
3. `README.md` runtime-РјРѕРґСѓР»СЏ Рё СЂР°Р·РґРµР» `## Interactions`
4. server-side authorization path РІ `apps/server`
5. `rustok-core::Permission`, `Resource`, `Action`

Р”Р»СЏ capability surfaces РґРѕРїРѕР»РЅРёС‚РµР»СЊРЅРѕ:

1. GraphQL/REST guards РІ `alloy`, `flex`, `rustok-mcp`
2. README/docs РґР»СЏ `alloy`, `alloy`, `rustok-mcp`
3. С‚РµРєСѓС‰РёР№ server composition-root РІ `apps/server`

---

## 3. РРЅРІР°СЂРёР°РЅС‚С‹

- [ ] Р’ live server authorization path РЅРµС‚ hardcoded `UserRole::Admin` / `UserRole::SuperAdmin` РєР°Рє Р·Р°РјРµРЅС‹ permission checks.
- [ ] `infer_user_role_from_permissions()` РёСЃРїРѕР»СЊР·СѓРµС‚СЃСЏ С‚РѕР»СЊРєРѕ РґР»СЏ presentation/compatibility/telemetry, РЅРѕ РЅРµ РґР»СЏ СЂРµР°Р»СЊРЅРѕР№ Р°РІС‚РѕСЂРёР·Р°С†РёРё.
- [ ] Runtime-РјРѕРґСѓР»Рё СЃ RBAC-managed behavior РїСѓР±Р»РёРєСѓСЋС‚ Р°РєС‚СѓР°Р»СЊРЅС‹Р№ permission surface С‡РµСЂРµР· `permissions()`.
- [ ] Runtime-РјРѕРґСѓР»Рё РґРѕРєСѓРјРµРЅС‚РёСЂСѓСЋС‚ `## Interactions` РІ root `README.md`.
- [ ] `outbox` СЏРІРЅРѕ Р·Р°РґРѕРєСѓРјРµРЅС‚РёСЂРѕРІР°РЅ РєР°Рє `Core` module Р±РµР· tenant-toggle semantics; РѕС‚СЃСѓС‚СЃС‚РІРёРµ СЃРѕР±СЃС‚РІРµРЅРЅРѕР№ RBAC surface РЅРµ РјР°СЃРєРёСЂСѓРµС‚ СѓСЃС‚Р°СЂРµРІС€СѓСЋ taxonomy.
- [ ] `blog -> content`
- [ ] `forum -> content`
- [ ] `pages -> content`
- [ ] `workflow` РЅРµ РѕРїРёСЃР°РЅ РєР°Рє runtime dependency Alloy, РµСЃР»Рё С‚Р°РєРѕРіРѕ dependency РЅРµС‚ РІ РєРѕРґРµ.
- [ ] Alloy РЅРµ СѓС‡Р°СЃС‚РІСѓРµС‚ РІ tenant module lifecycle РєР°Рє РѕР±С‹С‡РЅС‹Р№ runtime module.
- [ ] `apps/server` СЃРѕР·РґР°С‘С‚ `SecurityContext` РёР· resolved permissions, Р° РЅРµ РёР· role inference.
- [ ] Flex field-definition mutations РёСЃРїРѕР»СЊР·СѓСЋС‚ typed permissions, Р° РЅРµ role shortcuts.

---

## 4. Weekly checklist

### A. Registry Рё module contract

- [ ] Р—Р°РїСѓСЃС‚РёС‚СЊ `cargo test -p rustok-server modules::contract_tests --lib`
- [ ] РџСЂРѕРІРµСЂРёС‚СЊ, С‡С‚Рѕ РєР°Р¶РґС‹Р№ runtime-РјРѕРґСѓР»СЊ РІСЃС‘ РµС‰С‘ СЃРѕРґРµСЂР¶РёС‚ `## Interactions` РІ `README.md`
- [ ] РџСЂРѕРІРµСЂРёС‚СЊ, С‡С‚Рѕ `permissions()` РЅРµРїСѓСЃС‚РѕР№ Сѓ РјРѕРґСѓР»РµР№ СЃ RBAC-managed functionality:
  - `auth`, `tenant`, `rbac`, `content`, `commerce`, `blog`, `forum`, `pages`, `media`, `workflow`
- [ ] РџСЂРѕРІРµСЂРёС‚СЊ, С‡С‚Рѕ dependency edges РІСЃС‘ РµС‰С‘ РєРѕСЂСЂРµРєС‚РЅС‹:
  - `blog -> content`
  - `forum -> content`
  - `pages -> content`
- [ ] РџСЂРѕРІРµСЂРёС‚СЊ, С‡С‚Рѕ capability docs РїРѕ Alloy РІСЃС‘ РµС‰С‘ СЏРІРЅРѕ РіРѕРІРѕСЂСЏС‚, С‡С‚Рѕ Alloy РЅРµ СЏРІР»СЏРµС‚СЃСЏ runtime module

### B. Server authorization path

- [ ] РџРѕРёСЃРєР°С‚СЊ forbidden role-based authorization patterns РІ `apps/server/src`
- [ ] РџСЂРѕРІРµСЂРёС‚СЊ, С‡С‚Рѕ GraphQL Рё REST entry points РёРґСѓС‚ С‡РµСЂРµР· `RbacService`, RBAC extractors РёР»Рё permission-aware guards
- [ ] РџСЂРѕРІРµСЂРёС‚СЊ, С‡С‚Рѕ Alloy/scripts capability path РёСЃРїРѕР»СЊР·СѓРµС‚ `scripts:*`, Р° РЅРµ `tenant_modules.is_enabled("alloy")`
- [ ] РџСЂРѕРІРµСЂРёС‚СЊ, С‡С‚Рѕ MCP/Flex/workflow/media surfaces РЅРµ РІРІРѕРґСЏС‚ Р»РѕРєР°Р»СЊРЅС‹Рµ Р°РІС‚РѕСЂРёР·Р°С†РёРѕРЅРЅС‹Рµ РѕР±С…РѕРґС‹

### C. Typed permission vocabulary

- [ ] РџСЂРѕРІРµСЂРёС‚СЊ, С‡С‚Рѕ server-side RBAC surfaces РёСЃРїРѕР»СЊР·СѓСЋС‚ typed permissions РёР· `rustok-core`
- [ ] РџСЂРѕРІРµСЂРёС‚СЊ, С‡С‚Рѕ РЅРµ РїРѕСЏРІРёР»РёСЃСЊ ad-hoc string permissions РёР»Рё Р»РѕРєР°Р»СЊРЅС‹Рµ role aliases
- [ ] РџСЂРѕРІРµСЂРёС‚СЊ, С‡С‚Рѕ ownership permissions РїРѕ РјРѕРґСѓР»СЏРј СЃРѕРІРїР°РґР°РµС‚ СЃ С‚РµРєСѓС‰РёРјРё server callsites

### D. Runtime behavior

- [ ] Р—Р°РїСѓСЃС‚РёС‚СЊ focused RBAC/server slices
- [ ] Р—Р°РїСѓСЃС‚РёС‚СЊ `cargo test -p rustok-server --lib`
- [ ] РљР»Р°СЃСЃРёС„РёС†РёСЂРѕРІР°С‚СЊ РїР°РґРµРЅРёСЏ РєР°Рє RBAC drift, module contract drift, capability drift РёР»Рё unrelated failure

### E. Documentation freshness

- [ ] РџСЂРѕРІРµСЂРёС‚СЊ `docs/modules/registry.md`
- [ ] РџСЂРѕРІРµСЂРёС‚СЊ `docs/modules/crates-registry.md`
- [ ] РџСЂРѕРІРµСЂРёС‚СЊ Р»РѕРєР°Р»СЊРЅС‹Рµ README/docs РґР»СЏ `alloy`, `alloy`, `rustok-mcp`, `rustok-workflow`

---

## 5. РљРѕРјР°РЅРґС‹

### 5.1 Contract Рё grep checks

```powershell
cargo test -p rustok-server modules::contract_tests --lib
git grep -n "infer_user_role_from_permissions" -- apps/server/src
git grep -n "UserRole::Admin\|UserRole::SuperAdmin" -- apps/server/src
git grep -n "tenant_modules.is_enabled(\"alloy\")" -- apps/server/src crates/alloy crates/alloy
```

### 5.2 Focused runtime slices

```powershell
cargo test -p rustok-server rbac --lib
cargo test -p rustok-server metrics --lib
cargo test -p rustok-server flex --lib
```

### 5.3 РџРѕР»РЅС‹Р№ server gate

```powershell
cargo test -p rustok-server --lib
```

### 5.4 Р”РѕРїРѕР»РЅРёС‚РµР»СЊРЅС‹Рµ spot-checks РїРѕСЃР»Рµ RBAC-РёР·РјРµРЅРµРЅРёР№

```powershell
cargo test -p rustok-core --lib
cargo test -p rustok-rbac --lib
cargo test -p rustok-blog --lib
cargo test -p rustok-forum --lib
cargo test -p rustok-pages --lib
cargo test -p rustok-media --lib
cargo test -p rustok-workflow --lib
cargo test -p alloy --lib
```

---

## 6. РђСЂС‚РµС„Р°РєС‚С‹ РїСЂРѕРІРµСЂРєРё

РљР°Р¶РґС‹Р№ РїСЂРѕРіРѕРЅ РґРѕР»Р¶РµРЅ РѕСЃС‚Р°РІР»СЏС‚СЊ РєРѕСЂРѕС‚РєРёР№ evidence bundle:

- РґР°С‚Р°
- branch РёР»Рё commit
- РІС‹РїРѕР»РЅРµРЅРЅС‹Рµ РєРѕРјР°РЅРґС‹
- pass/fail
- СЃРїРёСЃРѕРє RBAC drift findings
- СЃРїРёСЃРѕРє module/capability doc drift findings
- СЃРїРёСЃРѕРє РёСЃРїСЂР°РІР»РµРЅРёР№
- РѕСЃС‚Р°РІС€РёРµСЃСЏ Р±Р»РѕРєРµСЂС‹

РџСЂРµРґРїРѕС‡С‚РёС‚РµР»СЊРЅРѕРµ РјРµСЃС‚Рѕ:

- `artifacts/verification/rbac-server-modules/<yyyy-mm-dd>.md`

---

## 7. Stop-the-line conditions

РЎС‡РёС‚Р°С‚СЊ Р±Р»РѕРєРёСЂСѓСЋС‰РёРј drift Р»СЋР±РѕР№ РёР· СЃР»РµРґСѓСЋС‰РёС… СЃР»СѓС‡Р°РµРІ:

- live server path Р°РІС‚РѕСЂРёР·СѓРµС‚ РїРѕ role shortcut РІРјРµСЃС‚Рѕ explicit permissions
- runtime-РјРѕРґСѓР»СЊ СЃ RBAC-managed behavior РїСѓР±Р»РёРєСѓРµС‚ РїСѓСЃС‚РѕР№ РёР»Рё СѓСЃС‚Р°СЂРµРІС€РёР№ `permissions()`
- runtime-РјРѕРґСѓР»СЊРЅС‹Р№ `README.md` РїРѕС‚РµСЂСЏР» `## Interactions`
- Alloy capability path СЃРЅРѕРІР° Р·Р°РІСЏР·Р°РЅ РЅР° tenant module gating
- `SecurityContext` СЃС‚СЂРѕРёС‚СЃСЏ Р±РµР· resolved permissions РЅР° server path
- `cargo test -p rustok-server --lib` РїР°РґР°РµС‚ РёР·-Р·Р° RBAC РёР»Рё module-contract regressions

