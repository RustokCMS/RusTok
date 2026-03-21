# Реестр crate-модулей `crates/rustok-*`

Документ фиксирует:

- зону ответственности crate'ов;
- их публичные entry points;
- недопустимые обходы модульного слоя;
- разделение между runtime-модулями и platform capabilities.

## Важная граница

RusToK использует смешанную архитектуру:

- runtime-модули живут в `crates/rustok-*` и регистрируются через `ModuleRegistry`;
- platform/core функциональность остаётся в `apps/server` и shared crate'ах;
- capability-слои вроде Alloy не обязаны быть runtime-модулями.

## Единый реестр

| Crate | Ответственность | Публичные entry points | Нельзя делать |
|---|---|---|---|
| `rustok-core` | Базовые платформенные контракты: модульная модель, события, RBAC-примитивы, shared types. | `RusToKModule`, `ModuleRegistry`, `Permission`, `Resource`, `Action`, `DomainEvent`, `EventEnvelope`. | Дублировать core-контракты в приложениях и модулях. |
| `rustok-api` | Общий web/API слой для transport-адаптеров: tenant/auth/request context, GraphQL helper'ы, extractor-friendly types. | `AuthContext`, `TenantContext`, `RequestContext`, `PageInfo`, `PaginationInput`, `GraphQLError`, `scope_matches`. | Возвращать общие HTTP/GraphQL helper'ы обратно в `apps/server` или в `rustok-core`. |
| `rustok-content` | Контентный домен и его GraphQL/REST adapters. | `ContentModule`, `NodeService`, `graphql::ContentQuery`, `graphql::ContentMutation`, `controllers::routes`. | Писать content transport-логику в `apps/server` или обходить сервисы direct SQL-операциями. |
| `rustok-commerce` | Commerce/catalog/inventory и transport-адаптеры. | `CommerceModule`, `CatalogService`, `PricingService`, `InventoryService`, `graphql::*`, `controllers::*`. | Реализовывать commerce transport/API поверх `apps/server` мимо crate. |
| `rustok-blog` | Blog-домен поверх content и transport-адаптеры. | `BlogModule`, `PostService`, `CommentService`, `graphql::*`, `controllers::*`. | Обходить blog-правила напрямую через `rustok-content` или SQL. |
| `rustok-forum` | Forum-домен и transport-адаптеры. | `ForumModule`, `TopicService`, `ReplyService`, `graphql::*`, `controllers::*`. | Обходить forum-сервисы через server-only handlers. |
| `rustok-pages` | Pages/menus/blocks и transport-адаптеры. | `PagesModule`, `PageService`, `graphql::*`, `controllers::*`. | Оставлять pages GraphQL/REST в `apps/server`. |
| `rustok-workflow` | Workflow automation, execution history, webhook ingress и transport-адаптеры. | `WorkflowModule`, `WorkflowService`, `WorkflowEngine`, `graphql::*`, `controllers::*`. | Считать Alloy runtime-зависимостью workflow registry-графа. |
| `rustok-media` | Media lifecycle, storage-facing services и transport-адаптеры. | `MediaService`, `graphql::*`, `controllers::*`. | Держать media transport/API слой в `apps/server`. |
| `alloy` | Transport-shell Alloy: GraphQL/REST adapters поверх runtime crate `alloy-scripting`. | `graphql::AlloyQuery`, `graphql::AlloyMutation`, `graphql::AlloyState`, `controllers::router`. | Трактовать Alloy как tenant-toggle модуль или возвращать его transport-логику в `apps/server`. |
| `rustok-index` | Индексация и search contracts. | `IndexModule`, `Indexer`, `LocaleIndexer`. | Строить ad-hoc индексацию мимо index contracts. |
| `rustok-rbac` | Authorization contracts и Casbin-backed runtime. | `RbacModule`, `PermissionResolver`, `PermissionAuthorizer`, `AuthzEngine`. | Возвращаться к hardcoded role checks в server-коде. |
| `rustok-tenant` | Tenant lifecycle и module enablement. | `TenantModule`, `TenantService`, tenant DTOs. | Менять tenant/module configuration напрямую в приложениях или SQL. |
| `rustok-outbox` | Transactional outbox и relay contracts. | `TransactionalEventBus`, `OutboxRelay`, `OutboxTransport`. | Публиковать критичные межмодульные события мимо outbox. |
| `rustok-iggy` | Event streaming transport runtime. | `IggyTransport`, topology/DLQ/replay managers. | Писать parallel transport-runtime для тех же потоков в сервисах. |
| `rustok-iggy-connector` | Подключение к Iggy и message I/O abstractions. | `IggyConnector`, `MessageSubscriber`, connector configs. | Обходить connector-абстракцию прямыми ad-hoc подключениями. |
| `rustok-telemetry` | Observability bootstrap. | `init`, `TelemetryConfig`, `render_metrics`, `current_trace_id`. | Настраивать разрозненные telemetry pipelines в разных модулях. |
| `rustok-mcp` | MCP adapter/server tool surface, Alloy-related integrations, identity/policy foundation, session-start runtime binding hooks, pluggable scaffold draft store и первый real Alloy scaffold flow через официальный SDK `rmcp`; persisted management layer, scaffold draft control plane и DB-backed runtime bridge живут в `apps/server`. | `RusToKMcpServer`, `McpServerConfig`, `McpSessionContext`, `McpRuntimeBinding`, `McpAccessResolver`, `McpAuditSink`, `McpScaffoldDraftStore`, `ScaffoldModuleRequest`, `StageModuleScaffoldResponse`, `ReviewModuleScaffoldRequest`, `ApplyModuleScaffoldRequest`, `serve_stdio`, tool re-exports. | Реализовывать отдельные MCP entrypoints в приложениях, если сценарий уже покрывает `rustok-mcp`; дублировать upstream spec/SDK docs в локальных файлах; считать текущий draft-store contract финальной заменой remote MCP bootstrap и полного codegen/publish pipeline. |
| `alloy-scripting` | Module-agnostic Alloy runtime/engine capability: script storage, execution, migrations. | Script storage/execution APIs, migrations, runtime helpers. | Регистрировать Alloy как runtime-модуль или навешивать на него tenant module lifecycle. |

## Runtime registry RBAC contract

Для runtime-модулей, которые реально регистрируются в `apps/server/src/modules/mod.rs`, канонический
RBAC-контракт задаётся тремя источниками:

- `RusToKModule::permissions()`;
- `RusToKModule::dependencies()`;
- `README.md` -> `## Interactions`.

Текущее владение RBAC surface:

- `rustok-auth` -> `users:*`
- `rustok-tenant` -> `tenants:*`, `modules:*`
- `rustok-rbac` -> `settings:*`, `logs:*`
- `rustok-content` -> content resources
- `rustok-commerce` -> commerce resources
- `rustok-blog` -> `blog_posts:*`
- `rustok-forum` -> `forum_categories:*`, `forum_topics:*`, `forum_replies:*`
- `rustok-pages` -> `pages:*`
- `rustok-workflow` -> `workflows:*`, `workflow_executions:*`

Alloy остаётся capability-слоем с permission surface `scripts:*`, но не входит в runtime registry.


