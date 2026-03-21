# rustok-pages docs

В этой папке хранится документация модуля `crates/rustok-pages`.

## Documents

- [Implementation plan](./implementation-plan.md)

## Event contracts

- [Event flow contract (central)](../../../docs/architecture/event-flow-contract.md)

## API contract notes

- `PageBodyInput` поддерживает `format=markdown|rt_json_v1`.
- Для `markdown` обязательное поле — `content` (непустой текст).
- Для `rt_json_v1` ожидается `content_json`; `content` можно использовать как raw JSON fallback для совместимости клиентов.
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
- GraphQL mutations: `createPage` (поддерживает `blocks`), `addBlock`, `updateBlock`, `deleteBlock`, `reorderBlocks`.
- Для блоковых операций используется существующая RBAC-модель `pages:*`; проверка делается по `AuthContext.permissions`, а затем в сервисы передаётся `SecurityContext`.

OpenAPI и GraphQL типы/мутации должны поддерживаться синхронно при дальнейших изменениях pages-контракта.
