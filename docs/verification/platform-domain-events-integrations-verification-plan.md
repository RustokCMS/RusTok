# План верификации платформы: события, доменные модули и интеграции

- **Статус:** Актуализированный детальный чеклист
- **Контур:** Event flow, outbox/runtime transport, доменные модули, модульные зависимости, межконтурные интеграции
- **Примечание:** API- и UI-поверхности проверяются в отдельных планах, но интеграционные склейки остаются здесь.

---

## Фаза 6: Событийная система

### 6.1 Event runtime

**Файлы:**
- `apps/server/src/services/event_transport_factory.rs`
- `apps/server/src/services/event_bus.rs`
- `crates/rustok-outbox/`
- `crates/rustok-iggy/`

- [ ] Подтверждено, что server bootstrap поднимает актуальный event runtime.
- [ ] Подтверждено, что поддерживаемые transport modes соответствуют коду и settings.
- [ ] `outbox` остаётся production-first transport path там, где это ожидается архитектурой.
- [ ] Iggy transport не дрейфует от текущих contracts и feature gates.

### 6.2 Transactional publish path

- [ ] Write-path доменных сервисов публикует события через transactional mechanism.
- [ ] Нет критичных publish-after-commit сценариев в content/commerce/blog/forum/pages/workflow.
- [ ] Event envelope и retry metadata совпадают с текущим contract layer.

### 6.3 Read-side / consumers

- [ ] `rustok-index` и другие consumers подписываются на актуальные события.
- [ ] Error handling в consumers не ломает batch/runtime loop.
- [ ] Backlog, retries и failed/DLQ semantics соответствуют текущему коду.

### 6.4 Coverage доменных событий

- [ ] Content: creation/update/publish/archive/delete сценарии отражены в текущем event vocabulary.
- [ ] Commerce: product/variant/inventory/price сценарии отражены в текущем event vocabulary.
- [ ] Blog/Forum: wrapper-модули не теряют модуль-специфичные события.
- [ ] Pages: корректно используют content/node event path.
- [ ] Media и Workflow event surfaces отражены там, где они уже реализованы в коде.
- [ ] Для не реализованных product areas нет устаревших обещаний в плане.

---

## Фаза 7: Доменные модули

### 7.1 `rustok-content`

- [ ] Entities, DTOs, GraphQL/REST adapters и `NodeService` соответствуют текущему коду.
- [ ] State machine, translations, bodies и tenant scoping отражены корректно.
- [ ] Миграции через `apps/server/migration` и/или shared migration path задокументированы честно.

### 7.2 `rustok-commerce`

- [ ] Product/variant/inventory/pricing surfaces соответствуют текущему набору сервисов.
- [ ] DTO validation и state machine checks отражены без устаревших допущений.
- [ ] Order-related ожидания не опережают фактическую реализацию.

### 7.3 `rustok-blog`

- [ ] `BlogModule` остаётся wrapper-модулем поверх content.
- [ ] Post/category/comment/tag surfaces соответствуют текущему коду.
- [ ] i18n, state machine и event publishing path актуальны.

### 7.4 `rustok-forum`

- [ ] Topic/reply/category/moderation surfaces соответствуют текущему коду.
- [ ] Wrapper-логика поверх content задокументирована корректно.
- [ ] Permissions и события не расходятся с module contract.

### 7.5 `rustok-pages`

- [ ] `pages -> content` отражено как runtime dependency.
- [ ] `PageService`, blocks, menus и node-backed persistence задокументированы корректно.
- [ ] Module-owned admin/storefront surfaces соответствуют коду.

### 7.6 `rustok-media`

- [ ] `MediaModule` включён в optional modules и отражён в плане.
- [ ] Entities, DTOs, GraphQL surface и `MediaService` соответствуют текущему коду.
- [ ] Storage integration и localized metadata отражены без устаревших предположений.

### 7.7 `rustok-workflow`

- [ ] `WorkflowModule` включён в optional modules и отражён в плане.
- [ ] Entities, GraphQL/REST surfaces, engine, trigger handler и built-in steps соответствуют текущему коду.
- [ ] Workflow не описан как runtime dependency Alloy, если в коде такой зависимости нет.

### 7.8 `rustok-index`

- [ ] `IndexModule` и search/read-model contract соответствуют текущему коду.
- [ ] Content/Product indexers и search engine wiring задокументированы корректно.

### 7.9 `rustok-rbac` и `rustok-tenant`

- [ ] `rustok-rbac` описан как `Core` module с relation/policy/runtime resolvers.
- [ ] `rustok-tenant` описан как `Core` module с CRUD + tenant_modules lifecycle.
- [ ] Migration ownership для этих модулей задокументирован честно.

### 7.10 Alloy и другие capability-crate'ы

- [ ] `alloy` и `alloy-scripting` не описаны как обычные tenant-toggle доменные модули.
- [ ] Capability boundaries и связь с workflow/MCP отражены без смешения с taxonomy platform modules.

---

## Фаза 13: Интеграционные связи

### 13.1 Module dependency contract

- [ ] Manifest dependencies и runtime dependencies совпадают.
- [ ] `blog/forum/pages -> content` проверяются как build-time и runtime инвариант.
- [ ] Optional modules не обходят dependency checks через host-приложения.

### 13.2 Write -> Event -> Read model

- [ ] Content/commerce/blog/forum/pages сценарии проходят путь write -> event -> index/read-side без разрыва.
- [ ] Workflow/event trigger path не расходится с текущим engine runtime.
- [ ] Build/event hub и GraphQL subscription path соответствуют текущему server runtime.

### 13.3 Host apps и module-owned surfaces

- [ ] Leptos Admin использует module-owned admin pages через `/modules/:module_slug` и `/*module_path`.
- [ ] Leptos Storefront использует module-owned page registrations и slot injections.
- [ ] Next.js/other hosts не документированы как потребители тех surfaces, которых в коде ещё нет.

### 13.4 Build, manifest и lifecycle integration

- [ ] Manifest diff -> build request -> build progress path соответствует текущим `BuildService` / GraphQL subscription / event hub контрактам.
- [ ] Tenant module lifecycle и build pipeline не описаны как независимые, если в коде они связаны.
