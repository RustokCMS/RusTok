# rustok-pages docs

В этой папке хранится документация модуля `crates/rustok-pages`.

## Documents

- [Implementation plan](./implementation-plan.md)
- [Admin package](../admin/README.md)
- [Storefront package](../storefront/README.md)

## Event contracts

- [Event flow contract (central)](../../../docs/architecture/event-flow-contract.md)

## API contract notes

- `PageBodyInput` поддерживает `format=markdown|rt_json_v1|grapesjs_v1`.
- Для `markdown` обязательное поле — `content` (непустой текст).
- Для `rt_json_v1` ожидается `content_json`; `content` можно использовать как raw JSON fallback для совместимости клиентов.
- Для `grapesjs_v1` ожидается `content_json` с `GrapesJS projectData`; `content` можно использовать как raw JSON fallback для совместимости клиентов.
- Перед записью payload проходит server-side sanitize/validation через `rustok_core::prepare_content_payload`.
- `BlockService` валидирует `data` по `BlockType` (schema-first DTO payload) и отклоняет неизвестные поля.
- Для `Video`/embed и URL-полей действует whitelist policy: `http/https` для ссылок, а embed только `https` + домены `youtube|youtu.be|vimeo|player.vimeo.com`.
- Для `Html` блоков запрещены опасные теги/протоколы (`<script>`, `<iframe>`, `javascript:`) и inline event handlers (`on*=`).

## Pages API (module-owned adapters)

Начиная с пилотного переноса архитектурного долга, transport-адаптеры pages живут в самом
`crates/rustok-pages`, а `apps/server` выступает composition root и тонким re-export/shim-слоем.

- REST `api/admin/pages/{id}`: `PUT` (update page), `DELETE` (delete page).
- REST блоки: `POST /api/admin/pages/{id}/blocks`, `PUT/DELETE /api/admin/pages/{page_id}/blocks/{block_id}`.
- REST reorder: `POST /api/admin/pages/{id}/blocks/reorder` с `block_ids`.
- GraphQL mutations: `createPage`/`updatePage` (поддерживают `body.format=grapesjs_v1`), `addBlock`, `updateBlock`, `deleteBlock`, `reorderBlocks`.
- Для блоковых операций используется существующая RBAC-модель `pages:*`; проверка делается по `AuthContext.permissions`, а затем в сервисы передаётся `SecurityContext`.
- GraphQL read/write entry points pages теперь по умолчанию берут tenant из `TenantContext`; optional
  `tenantId` остаётся только как override-аргумент, чтобы publishable UI-пакеты не зависели от tenant UUID в host-е.

На текущем этапе `body.format=grapesjs_v1` считается каноническим write-path для нового visual page-builder, а block endpoints сохраняются как legacy/migration-compatible поверхность до синхронизации storefront renderers.

OpenAPI и GraphQL типы/мутации должны поддерживаться синхронно при дальнейших изменениях pages-контракта.

## Module-owned UI packages

- `crates/rustok-pages/admin/` — publishable Leptos admin root package (`PagesAdmin`).
- `crates/rustok-pages/storefront/` — publishable Leptos storefront root package (`PagesView`).
- Host applications подключают их через manifest-driven generated wiring, без ручной pages-логики в `apps/admin` и `apps/storefront`.
- `PagesAdmin` теперь уже не scaffold: пакет делает реальный list/create/edit/update/publish/delete flow через модульный GraphQL.
- `PagesView` теперь рендерит реальный storefront read-path поверх `pageBySlug(slug: ...)`, каталога опубликованных страниц и generic `UiRouteContext`, который host передаёт без knowledge о конкретном модуле.
