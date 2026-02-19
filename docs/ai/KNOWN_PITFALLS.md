# KNOWN_PITFALLS для AI (RusToK)

Короткий список типичных ошибок перед изменениями кода.

## Loco

- Не обходить Loco hooks (`Hooks::routes`, `Hooks::after_routes`, `Hooks::connect_workers`) через отдельный жизненный цикл «чистого Axum». См. `docs/references/loco/README.md`.
- Не заменять `AppContext` на глобальные singleton-объекты, если зависимость уже должна жить в `ctx.shared_store`.
- Не смешивать произвольные контракты ошибок: для контроллеров придерживаться `loco_rs::Result<...>`.

## Iggy / Outbox

- Для write + event не использовать fire-and-forget `publish(...)`; нужен `publish_in_tx(...)`.
- Не переносить в код Kafka/NATS-специфичные API (offset commits, subject-only routing), которых нет в текущем abstraction.
- Не выдумывать конфигурацию Iggy: сначала сверяться с актуальными `IggyConfig`, `ConnectorConfig`, `ConnectorMode`.


## MCP

- Не обходить typed tools/response envelope (`McpToolResponse`) ad-hoc JSON-ответами.
- Не переносить бизнес-логику в MCP адаптер: слой должен оставаться тонким над service/registry.
- Для ограниченного доступа использовать allow-list инструментов через `McpServerConfig::with_enabled_tools(...)`.

## Outbox

- Для write + event, требующих консистентности, использовать `publish_in_tx(...)`, а не `publish(...)`.
- Не запускать production c outbox без relay-воркера.

## Telemetry

- Не делать многократную инициализацию telemetry runtime.
- Не разносить метрики по разным registry без необходимости.

## Обязательная проверка перед изменениями

Если задача затрагивает Loco/Iggy/MCP/Outbox/Telemetry:
1. Сначала открыть соответствующий reference-пакет:
   - `docs/references/loco/README.md`
   - `docs/references/iggy/README.md`
   - `docs/references/mcp/README.md`
   - `docs/references/outbox/README.md`
   - `docs/references/telemetry/README.md`
2. Только после этого менять код/документацию.
