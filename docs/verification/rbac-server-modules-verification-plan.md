# План rolling-верификации RBAC для server и runtime-модулей

- **Статус:** Актуализированный rolling-чеклист
- **Режим:** Повторяемая точечная верификация
- **Полная частота:** 1 раз в неделю и после любых изменений в RBAC, server transport, module contracts или capability-boundaries
- **Цель:** Держать в согласованном состоянии live RBAC contract между `apps/server`, `rustok-rbac`, runtime-модулями из `build_registry()` и capability-поверхностями

---

## 1. Объект проверки

### 1.1 Server surfaces

- `apps/server`
- REST RBAC extractors и guards
- GraphQL queries / mutations / subscriptions
- `RbacService`
- permission-aware `SecurityContext`

### 1.2 Runtime modules из `build_registry()`

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

### 1.3 Capability surfaces, проверяемые вместе с RBAC

- `alloy`
- `alloy-scripting`
- `flex`
- `rustok-mcp`

---

## 2. Источники истины

Каждый проход должен считать каноническими следующие контракты:

1. `RusToKModule::permissions()`
2. `RusToKModule::dependencies()`
3. `README.md` runtime-модуля и раздел `## Interactions`
4. server-side authorization path в `apps/server`
5. `rustok-core::Permission`, `Resource`, `Action`

Для capability surfaces дополнительно:

1. GraphQL/REST guards в `alloy`, `flex`, `rustok-mcp`
2. README/docs для `alloy`, `alloy-scripting`, `rustok-mcp`
3. текущий server composition-root в `apps/server`

---

## 3. Инварианты

- [ ] В live server authorization path нет hardcoded `UserRole::Admin` / `UserRole::SuperAdmin` как замены permission checks.
- [ ] `infer_user_role_from_permissions()` используется только для presentation/compatibility/telemetry, но не для реальной авторизации.
- [ ] Runtime-модули с RBAC-managed behavior публикуют актуальный permission surface через `permissions()`.
- [ ] Runtime-модули документируют `## Interactions` в root `README.md`.
- [ ] `outbox` явно задокументирован как `Core` module без tenant-toggle semantics; отсутствие собственной RBAC surface не маскирует устаревшую taxonomy.
- [ ] `blog -> content`
- [ ] `forum -> content`
- [ ] `pages -> content`
- [ ] `workflow` не описан как runtime dependency Alloy, если такого dependency нет в коде.
- [ ] Alloy не участвует в tenant module lifecycle как обычный runtime module.
- [ ] `apps/server` создаёт `SecurityContext` из resolved permissions, а не из role inference.
- [ ] Flex field-definition mutations используют typed permissions, а не role shortcuts.

---

## 4. Weekly checklist

### A. Registry и module contract

- [ ] Запустить `cargo test -p rustok-server modules::contract_tests --lib`
- [ ] Проверить, что каждый runtime-модуль всё ещё содержит `## Interactions` в `README.md`
- [ ] Проверить, что `permissions()` непустой у модулей с RBAC-managed functionality:
  - `auth`, `tenant`, `rbac`, `content`, `commerce`, `blog`, `forum`, `pages`, `media`, `workflow`
- [ ] Проверить, что dependency edges всё ещё корректны:
  - `blog -> content`
  - `forum -> content`
  - `pages -> content`
- [ ] Проверить, что capability docs по Alloy всё ещё явно говорят, что Alloy не является runtime module

### B. Server authorization path

- [ ] Поискать forbidden role-based authorization patterns в `apps/server/src`
- [ ] Проверить, что GraphQL и REST entry points идут через `RbacService`, RBAC extractors или permission-aware guards
- [ ] Проверить, что Alloy/scripts capability path использует `scripts:*`, а не `tenant_modules.is_enabled("alloy")`
- [ ] Проверить, что MCP/Flex/workflow/media surfaces не вводят локальные авторизационные обходы

### C. Typed permission vocabulary

- [ ] Проверить, что server-side RBAC surfaces используют typed permissions из `rustok-core`
- [ ] Проверить, что не появились ad-hoc string permissions или локальные role aliases
- [ ] Проверить, что ownership permissions по модулям совпадает с текущими server callsites

### D. Runtime behavior

- [ ] Запустить focused RBAC/server slices
- [ ] Запустить `cargo test -p rustok-server --lib`
- [ ] Классифицировать падения как RBAC drift, module contract drift, capability drift или unrelated failure

### E. Documentation freshness

- [ ] Проверить `docs/modules/registry.md`
- [ ] Проверить `docs/modules/crates-registry.md`
- [ ] Проверить локальные README/docs для `alloy`, `alloy-scripting`, `rustok-mcp`, `rustok-workflow`

---

## 5. Команды

### 5.1 Contract и grep checks

```powershell
cargo test -p rustok-server modules::contract_tests --lib
git grep -n "infer_user_role_from_permissions" -- apps/server/src
git grep -n "UserRole::Admin\|UserRole::SuperAdmin" -- apps/server/src
git grep -n "tenant_modules.is_enabled(\"alloy\")" -- apps/server/src crates/alloy crates/alloy-scripting
```

### 5.2 Focused runtime slices

```powershell
cargo test -p rustok-server rbac --lib
cargo test -p rustok-server metrics --lib
cargo test -p rustok-server flex --lib
```

### 5.3 Полный server gate

```powershell
cargo test -p rustok-server --lib
```

### 5.4 Дополнительные spot-checks после RBAC-изменений

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

## 6. Артефакты проверки

Каждый прогон должен оставлять короткий evidence bundle:

- дата
- branch или commit
- выполненные команды
- pass/fail
- список RBAC drift findings
- список module/capability doc drift findings
- список исправлений
- оставшиеся блокеры

Предпочтительное место:

- `artifacts/verification/rbac-server-modules/<yyyy-mm-dd>.md`

---

## 7. Stop-the-line conditions

Считать блокирующим drift любой из следующих случаев:

- live server path авторизует по role shortcut вместо explicit permissions
- runtime-модуль с RBAC-managed behavior публикует пустой или устаревший `permissions()`
- runtime-модульный `README.md` потерял `## Interactions`
- Alloy capability path снова завязан на tenant module gating
- `SecurityContext` строится без resolved permissions на server path
- `cargo test -p rustok-server --lib` падает из-за RBAC или module-contract regressions
