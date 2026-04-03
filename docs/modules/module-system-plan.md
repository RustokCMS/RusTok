# RusTok: план закрытия module-system

> **Дата**: 2026-03-19  
> **Актуализировано**: 2026-04-03  
> **Назначение**: зафиксировать текущее состояние модульной платформы, разделение `Registry V1` и `Registry V2`, а также остаток работ до production-ready контура.

Легенда:
- `✅` реализовано
- `⚠️` частично реализовано
- `⬜` не начато

## Короткий вывод

- `Registry V1` уже существует как постоянный read-only catalog API.
- `Registry V2` уже существует как первый рабочий write/governance контур, но пока без полноценного async validation pipeline и богатой модели ownership/moderation.
- `DeploymentProfile` и `RuntimeHostMode` уже разделены в коде и должны оставаться независимыми осями.
- Manifest-wired Leptos UI уже работает для набора канонических dual-surface модулей; основной остаток здесь в покрытии и дисциплине validation/tooling.

## Архитектурные инварианты

### V1 и V2

- `Registry V1` — это версия read-only API каталога, а не временный этап.
- `Registry V2` — это версия write/governance API, которая не заменяет `V1`, а публикует в него release-проекцию.
- Внешние discovery consumers и operator UI читают каталог через `GET /v1/catalog*`.
- Write-path, publish lifecycle и governance идут через `V2`.

### Deployment profile и runtime host mode

- `DeploymentProfile` описывает build/deploy surface:
  - `monolith`
  - `server-with-admin`
  - `server-with-storefront`
  - `headless-api`
- `RuntimeHostMode` описывает runtime-exposed surface:
  - `full`
  - `registry_only`
- Эти оси независимы.
- `registry_only` остаётся строго read-only режимом поверх `HeadlessApi`.
- `V2` должен быть API-first и одинаково доступным в `Monolith + full` и `HeadlessApi + full`.

## Что уже закрыто

### 1. Manifest contract и module metadata

- ✅ `rustok-module.toml` является каноническим manifest-контрактом для path-модулей.
- ✅ Парсятся и валидируются:
  - `[module]`
  - `[compatibility]`
  - `[dependencies]`
  - `[conflicts]`
  - `[crate]`
  - `[provides]`
  - `[settings]`
  - `[locales]`
  - `[marketplace]`
- ✅ `[settings]` уже поддерживает schema-driven scalar controls, `options`, `min`/`max`, nested `properties`/`items`, inline editor для nested `object`/`array` и schema-aware child editors.
- ✅ `[marketplace]` уже протянут через manifest → catalog → GraphQL → Admin UI:
  - `category`
  - `tags`
  - `icon`
  - `banner`
  - `screenshots`
- ✅ Базовая local validation для marketplace metadata уже включена.

### 2. Build/codegen и host wiring

- ✅ `apps/server/build.rs` генерирует optional module registry, schema fragments и routes из `modules.toml`.
- ✅ `apps/admin/build.rs` уже строит generic module root pages/nav/dashboard и nested route metadata из `[[provides.admin_ui.pages]]`.
- ✅ `apps/storefront/build.rs` поддерживает multi-slot storefront sections и generic route `/modules/{route_segment}`.
- ✅ Build/release pipeline исполняет manifest-derived сборку `server`, `admin` и Leptos `storefront`.
- ✅ `ManifestManager` валидирует semver-диапазоны зависимостей, runtime/product conflicts и schema-driven settings contract.

### 3. Admin `/modules`

- ✅ `updateModuleSettings` закрыт end-to-end.
- ✅ `/modules` умеет:
  - tenant-level toggle
  - platform install/uninstall
  - schema-driven settings editing
  - build progress через GraphQL WS с polling fallback
  - registry readiness summary
  - visual marketplace preview
  - release trail summary
  - publish lifecycle block для `Registry V2`
- ✅ Последние operator-facing строки в module UI выровнены под locale-aware rendering, чтобы модульный UX не разваливался на смешение русского и английского.

### 4. Manifest-wired UI coverage

Текущее состояние path-модулей:

| Модуль | Статус UI wiring | Комментарий |
|---|---|---|
| `blog` | ✅ dual-surface | admin + storefront |
| `commerce` | ✅ dual-surface | admin + storefront |
| `forum` | ✅ dual-surface | admin + storefront |
| `pages` | ✅ dual-surface | admin + storefront |
| `search` | ✅ dual-surface | admin + storefront |
| `workflow` | ✅ admin-only | осознанный admin-only slice |
| `channel` | ✅ admin-only | осознанный admin-only slice |

Дополнительно:

- ✅ Наличие `admin/Cargo.toml` или `storefront/Cargo.toml` без соответствующего `[provides.*_ui].leptos_crate` теперь считается wiring error.
- ✅ `cargo xtask module validate`, `apps/admin/build.rs` и `apps/storefront/build.rs` используют manifest wiring как источник правды, а не наличие sub-crate на диске.

### 5. Registry V1

`Registry V1` уже реализован как read-model:

- ✅ `GET /v1/catalog`
- ✅ `GET /v1/catalog/{slug}`
- ✅ legacy aliases `GET /catalog` и `GET /catalog/{slug}` сохранены как backward-compatible fallback

Поддерживается:

- ✅ schema-versioned payload
- ✅ filters: `search`, `category`, `tag`
- ✅ paging: `limit`, `offset`
- ✅ `X-Total-Count`
- ✅ stable ordering для paging
- ✅ `ETag`
- ✅ `Cache-Control`
- ✅ `If-None-Match`
- ✅ `304 Not Modified`
- ✅ OpenAPI coverage

Каталожные данные уже включают:

- ✅ `name`
- ✅ `description`
- ✅ `category`
- ✅ `tags`
- ✅ `icon`
- ✅ `banner`
- ✅ `screenshots`
- ✅ release trail normalization
- ✅ checksum/publisher normalization

Consumer path:

- ✅ `RegistryMarketplaceProvider` умеет list/detail-aware lookup
- ✅ GraphQL `marketplace` и `marketplaceModule` используют V1-aware provider path
- ✅ `/modules` уже использует `tag` filter и registry metadata

### 6. `registry_only` host

- ✅ `apps/server` уже умеет работать в `settings.rustok.runtime.host_mode = "registry_only"`.
- ✅ В этом режиме остаются только:
  - `health`
  - `metrics`
  - `swagger`
  - V1 catalog routes
- ✅ GraphQL, auth, MCP и embedded UI surfaces не поднимаются.
- ✅ `OpenAPI` в этом режиме режется до реального reduced surface.
- ✅ `/health/ready`, `/health/runtime` и runtime metrics выровнены под reduced host.
- ✅ `RUSTOK_RUNTIME_HOST_MODE=registry_only` поддерживается через env override.
- ✅ Локальный smoke path уже зафиксирован в `apps/server/docs/health.md` и `scripts/verify/verify-deployment-profiles.sh`.

### 7. Registry V2

Первый рабочий V2 lifecycle уже есть:

- ✅ `POST /v2/catalog/publish`
- ✅ `GET /v2/catalog/publish/{request_id}`
- ✅ `PUT /v2/catalog/publish/{request_id}/artifact`
- ✅ `POST /v2/catalog/publish/{request_id}/validate`
- ✅ `POST /v2/catalog/publish/{request_id}/approve`
- ✅ `POST /v2/catalog/publish/{request_id}/reject`
- ✅ `POST /v2/catalog/yank`

Текущее lifecycle-состояние:

- ✅ request создаётся отдельно от artifact upload
- ✅ upload теперь только сохраняет artifact и ставит request в `submitted`
- ✅ validation вынесена в отдельный lifecycle-step `/validate`
- ✅ `/validate` валидирует bundle against request
- ✅ `/validate` теперь работает как queue boundary и переводит request в background `validating`, а не держит bundle-check inline в HTTP вызове
- ✅ request проходит через `artifact_uploaded -> submitted -> validating -> approved` или `rejected`
- ✅ публикация release происходит отдельным governance action
- ✅ `yank` требует обязательную причину
- ✅ published releases уже проецируются обратно в `V1`

Governance first cut:

- ✅ header-driven actor contract уже работает
- ✅ persisted slug owner binding уже сохраняется отдельно от governance actor; release publisher identity и `/modules` lifecycle UX теперь читают этот binding, а `x-rustok-publisher` отделён от audit actor `x-rustok-actor`
- ✅ persisted audit trail теперь сохраняется в `registry_governance_events` и уже отражается в `/modules` как recent governance events для publish/upload/validate/approve/reject/yank/owner-binding переходов
- ✅ есть first-cut policy для governance actors и slug-scoped publishers
- ✅ `xtask module publish` уже умеет live orchestration
- ✅ `xtask module yank` уже умеет live V2 endpoint
- ✅ dry-run режим сохранён

## Что остаётся сделать

### Блок A. Registry V2 async validation и governance

- ⚠️ Validation pipeline уже отделён от upload path и вынесен за пределы inline HTTP upload/validate round-trip, но compile/test/security/policy checks всё ещё остаются в одном лёгком background validator без отдельного job orchestration слоя.
- ⬜ Нужно вынести compile/test/security/policy checks в асинхронный контур.
- ⬜ Нужен явный request lifecycle для background validation jobs и повторной обработки.
- ⚠️ Базовая persistence-модель для ownership и audit trail уже есть (`registry_module_owners`, `registry_governance_events`), но richer moderation decisions всё ещё не доведены до полного production policy.
- ⬜ Нужен более строгий policy layer для:
  - ownership transfer
  - moderator/admin approve-reject capabilities
  - unpublish/yank governance rules

### Блок B. Отдельный deployment для `modules.rustok.dev`

- ⚠️ `registry_only` runtime уже есть, но отдельный внешний deployment ещё не оформлен до production-ready состояния.
- ⬜ Нужны финальные rollout/runbook шаги для dedicated catalog host.
- ⬜ Нужны явные acceptance-smoke проверки для внешнего deployment surface.

### Блок C. Покрытие UI и operator polish

- ⚠️ Канонические dual-surface proof points уже есть, но покрытие не всех модулей доведено до финального состояния.
- ⬜ Нужно продолжить аудит path-модулей на предмет честной классификации `dual-surface` / `admin-only` / `storefront-only` / `no-ui`.
- ⬜ Нужно расширить operator UX вокруг governance:
  - ownership hints
  - moderation decisions
  - async validation status
  - richer validation errors

### Блок D. Тесты

- ⚠️ Точечные `cargo check` по релевантным пакетам уже проходят для большинства последних шагов.
- ⚠️ Полный workspace/test graph регулярно блокируется незавершённой параллельной разработкой в соседних crate-ах.
- ⬜ Нужны более устойчивые targeted tests для:
  - V2 lifecycle transitions
  - projection V2 → V1
  - `registry_only` reduced surface
  - manifest-wired UI guardrails

## Приоритет выполнения

1. Вынести async validation из sync upload path.
2. Довести ownership/governance persistence и policy model.
3. Закрыть production deployment path для `modules.rustok.dev`.
4. Доработать lifecycle-aware operator UX в `/modules`.
5. Продолжить аудит и доводку manifest-wired UI coverage.
6. Уплотнить targeted test coverage вокруг V1/V2 и reduced host.

## Критерии завершения

План можно считать закрытым, когда одновременно выполняются условия:

- `V1` стабилен как постоянный read-only catalog API.
- `V2` имеет полноценный async publish/governance lifecycle.
- `registry_only` развёртывается как отдельный catalog host без monolith surface.
- publish/yank flow работает и в `Monolith + full`, и в `HeadlessApi + full`.
- `/modules` показывает operator-friendly lifecycle, validation и governance state.
- UI wiring всех path-модулей честно классифицирован и проверяется tooling-guardrails.

## Связанные документы

- [Контракт manifest-файла](./manifest.md)
- [Реестр модулей и владельцев](./registry.md)
- [Обзор модульной платформы](./overview.md)
- [Архитектура модулей](../architecture/modules.md)
- [GraphQL и Leptos server functions](../UI/graphql-architecture.md)
- [Server docs](../../apps/server/docs/README.md)
- [Admin docs](../../apps/admin/docs/README.md)
