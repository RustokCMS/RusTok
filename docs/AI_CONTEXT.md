# AI Context для RusToK

Обязательный стартовый контекст для AI-сессий.

## Порядок чтения

1. `docs/index.md`
2. `docs/AI_CONTEXT.md`
3. `README.md` и `CRATE_API.md` целевого компонента
4. При event-изменениях: `crates/rustok-outbox/docs/README.md` и `docs/architecture/event-flow-contract.md`

## Терминология

### Platform modules

Для platform modules существует только два статуса:

- `Core`
- `Optional`

Источник истины по составу модулей — `modules.toml`.

### Crates

`crate` — техническая упаковка в Cargo. Не каждый crate в `crates/` автоматически является platform module.

В `crates/` лежат:

- module-crates
- shared libraries
- infrastructure/support crates

### Важное правило

Не смешивай:

- **статус модуля** (`Core` / `Optional`)
- **способ wiring** (`ModuleRegistry`, bootstrap, codegen, host wiring)
- **форму упаковки** (`crate`)

`rustok-outbox` — `Core` module. То, что event runtime использует его напрямую, не делает его отдельным типом компонента.

## Текущий platform baseline

### Core modules

- `auth`
- `cache`
- `email`
- `index`
- `outbox`
- `tenant`
- `rbac`

### Optional modules

- `content`
- `commerce`
- `blog`
- `forum`
- `pages`
- `media`
- `workflow`

## Общие инварианты

- Platform modules должны оставаться согласованными между `modules.toml`, `build_registry()` и manifest validation.
- Для write-flow с межмодульными событиями используется transactional outbox.
- Tenant isolation и RBAC обязательны в сервисном слое.
- События и обработчики должны оставаться совместимыми по `DomainEvent` / `EventEnvelope`.

## Важные crate'ы

### `crates/rustok-core`

Платформенные контракты: `RusToKModule`, `ModuleRegistry`, permissions, events, health, metrics.

### `crates/rustok-events`

Канонический слой event-контрактов поверх platform event model.

### `crates/rustok-outbox`

`Core` module для transactional event persistence: `TransactionalEventBus`, `OutboxTransport`, `OutboxRelay`, `SysEventsMigration`.

### `crates/rustok-tenant`

`Core` module multi-tenant lifecycle и module enablement.

### `crates/rustok-rbac`

`Core` module authorization, roles, policies и permission resolution.

### `crates/rustok-content` / `commerce` / `blog` / `forum` / `pages` / `media` / `workflow`

Optional domain modules и их transport/UI surfaces.

## Do / Don't

### Do

- Используй только реально существующие API из кода и docs.
- Для доменных write-flow с событиями применяй `publish_in_tx`, когда нужен атомарный publish.
- Проверяй, что docs отражают текущий код, а не старые архитектурные предположения.

### Don't

- Не придумывай третий тип модулей кроме `Core` и `Optional`.
- Не подменяй архитектурный статус модуля способом runtime wiring.
- Не обходи outbox в production event-flow.
