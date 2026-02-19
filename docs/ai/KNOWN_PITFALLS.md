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


## Постоянные guardrails для задач с AI-генерацией

Эти пункты считаются **долгоживущими инструкциями к действию** (не milestone notes) и применяются при проектировании/ревью изменений:

- **События после write-операций**: для консистентных write-path использовать только transactional публикацию (`publish_in_tx(...)`).
- **Tenant-изоляция**: любой доступ к данным обязан быть tenant-scoped (tenant_id в фильтрах/контексте), без «глобальных» выборок в domain-коде.
- **Модульные границы**: междоменные импорты контролировать через `Cargo.toml` (нет зависимости — нет импорта), не связывать доменные модули напрямую без явной причины.
- **Безопасные ошибки вместо паник**: избегать `unwrap/expect/panic` в production-коде; использовать типизированные ошибки и явную обработку.
- **Опора на локальные эталоны**: перед генерацией нового слоя (controller/service/migration/test) сверяться с существующими примерами и reference-пакетами в `docs/references/*`.

### Что делаем в первую очередь

- Усиливаем lint/CI-проверки на unsafe error-handling (`unwrap/expect/panic`).
- Поддерживаем и расширяем набор локальных эталонов/примеров для AI-сессий.
- Проверяем tenant-изоляцию и event-consistency в integration tests для новых write-сценариев.

### Что внедряем поэтапно

- Более строгие compile-time паттерны (например, обязательный service outcome с событиями, scoped repository abstractions) внедряем итеративно, без массового breaking-рефакторинга.
