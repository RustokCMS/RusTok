# Обзор модуля Alloy Scripting: что делает, что может, рекомендации

> **См. также:** [Alloy Concept](./alloy-concept.md) — стратегическое видение Alloy как Self-Evolving Integration Runtime.

## Контекст

Ревью модуля `alloy-scripting` — проверка манифеста разработки, текущей реализации, описание возможностей и рекомендации по улучшению. Ключевая идея: **интеграция Alloy с модулем MCP** для создания AI-программируемого слоя бизнес-логики — скрипты, миграции данных, генерация нового функционала, адаптация под данные.

---

## Что делает модуль Alloy

**Alloy Scripting** — опциональный модуль RusTok, предоставляющий Rhai-based скриптовый движок для пользовательской автоматизации. Зарегистрирован как `ModuleKind::Optional` с slug `"alloy"`.

### Архитектура (5 слоёв)

| Слой | Компоненты | Файлы |
|------|-----------|-------|
| **API** | REST CRUD + GraphQL | `api/handlers.rs`, `api/routes.rs`, `graphql/alloy/` |
| **Runner** | Orchestrator, Executor, Scheduler | `runner/orchestrator.rs`, `runner/executor.rs`, `scheduler/` |
| **Engine** | Rhai wrapper + кэш AST | `engine/runtime.rs`, `engine/config.rs` |
| **Storage** | ScriptRegistry trait + реализации | `storage/traits.rs`, `storage/sea_orm.rs`, `storage/memory.rs` |
| **Integration** | HookExecutor + ScriptableEntity | `integration/hook_executor.rs`, `integration/scriptable.rs` |

### Ключевые возможности

1. **CRUD скриптов** — создание, чтение, обновление, удаление через REST и GraphQL
2. **4 типа триггеров**: Event (before/after/on_commit), Cron, API endpoint, Manual
3. **Фазы выполнения**: Before (валидация), After (side effects), OnCommit (webhooks), Manual, Scheduled
4. **EntityProxy** — доступ к полям сущности с трекингом изменений (`entity["field"]`)
5. **Sandbox безопасности** — лимиты на операции (50K), timeout (100ms), глубину вызовов (16), строки (64KB), массивы (10K)
6. **AST-кэширование** с hash-based инвалидацией
7. **Auto-disable** — скрипт отключается после 3 ошибок подряд
8. **Cron-планировщик** — background execution с tick каждую секунду
9. **RBAC** — 6 permissions (CRUD + List + Manage) на ресурсе `Scripts`
10. **Helper-функции**: log, abort, validate_email/required/min_length/max_length/range, format_money, is_empty, coalesce

---

## Найденные проблемы и рекомендации

### Критические (баги / архитектурные проблемы)

#### 1. Дублирование `json_to_dynamic` / `dynamic_to_json`
- **Проблема**: Код конвертации JSON↔Dynamic продублирован в 3 местах:
  - `apps/server/src/graphql/alloy/mod.rs:54-119`
  - `crates/alloy-scripting/src/api/handlers.rs:356-413`
  - Фактически идентичная логика
- **Рекомендация**: Вынести в `alloy_scripting::utils` как публичные функции и переиспользовать

#### 2. Отсутствие `tenant_id` фильтрации в storage
- **Проблема**: `ExecutionContext` имеет поле `tenant_id`, но `SeaOrmStorage` и `ScriptQuery` не фильтруют скрипты по tenant. В мультитенантном приложении это означает, что скрипты одного тенанта видны другому.
- **Рекомендация**: Добавить `tenant_id` в таблицу `scripts` (новая миграция), в `ScriptQuery`, и фильтровать во всех запросах storage

#### 3. `validate_script` выполняет скрипт вместо только компиляции
- **Проблема**: `handlers.rs:342` — `validate_script` вызывает `engine.execute()` вместо `engine.compile()`. Это выполняет скрипт при валидации, что может иметь side-effects (log, abort).
- **Рекомендация**: Заменить на `engine.compile()` для чистой проверки синтаксиса

#### 4. Отсутствие валидации cron-выражения при создании скрипта
- **Проблема**: При создании/обновлении скрипта с `Cron` триггером нет проверки валидности cron-выражения. Ошибка обнаруживается только при загрузке в Scheduler.
- **Рекомендация**: Валидировать `cron::Schedule::from_str()` в `save()` или в GraphQL mutation `create_script`/`update_script`

#### 5. `run_before` в orchestrator не обрабатывает `ExecutionOutcome::Failed`
- **Проблема**: `orchestrator.rs:54-76` — цикл `run_before` обрабатывает `Aborted` (rejection) и `Success`, но при `Failed` продолжает выполнение следующих скриптов, молча игнорируя ошибку. В `run_after` это обрабатывается (возвращается `HookOutcome::Error`).
- **Рекомендация**: Добавить обработку `Failed` в `run_before` аналогично `run_after`

### Значительные (design gaps)

#### 6. Scheduler не запускается из `app.rs`
- **Проблема**: В `app.rs:78-88` создаются engine, storage, orchestrator, но `Scheduler` не создаётся и не запускается. Cron-триггеры описаны в модели, но фактически не работают.
- **Рекомендация**: Инициализировать `Scheduler`, вызвать `load_jobs()` и `start()` в background task при старте приложения

#### 7. REST API router не подключён к серверу
- **Проблема**: `create_router()` определён в `api/routes.rs`, но нигде не вызывается в `app.rs`. Весь REST API модуля мёртвый код.
- **Рекомендация**: Либо подключить REST router к основному серверу, либо удалить дублирующий REST API (если GraphQL достаточно)

#### 8. `health()` всегда возвращает `Healthy`
- **Проблема**: `modules/alloy.rs:49` — health check не проверяет состояние engine, storage или scheduler.
- **Рекомендация**: Проверять доступность БД (простой `find` запрос) и состояние scheduler

#### 9. Bridge placeholders для DB и External services
- **Проблема**: `bridge/mod.rs:54-63` — `register_db_services()` и `register_external_services()` пустые. After и OnCommit фазы не имеют дополнительных возможностей по сравнению с Before.
- **Рекомендация**: Определить приоритет реализации. Как минимум HTTP bridge для webhook-интеграций в OnCommit фазе

#### 10. Pagination в GraphQL делается в памяти
- **Проблема**: `query.rs:30-48` — загружает ВСЕ скрипты по статусу, потом пагинирует через `skip/take` в Rust. При большом количестве скриптов это неэффективно.
- **Рекомендация**: Добавить `LIMIT/OFFSET` в `ScriptRegistry::find()` или добавить отдельный метод `find_paginated()`

### Улучшения (quality of life)

#### 11. Нет audit log'а выполнений
- **Проблема**: Результаты выполнения скриптов нигде не сохраняются. Невозможно отследить историю, отладить проблемы, собрать метрики.
- **Рекомендация**: Добавить таблицу `script_executions` (script_id, execution_id, phase, outcome, duration_ms, error, created_at). Уже отмечено в implementation-plan как Phase 2.

#### 12. Нет метрик (observability)
- **Проблема**: Нет Prometheus/OpenTelemetry метрик для скриптов (execution count, latency, error rate).
- **Рекомендация**: Добавить spans/metrics в executor, учитывая что проект уже использует OpenTelemetry

#### 13. `validate_email` слишком примитивна
- **Проблема**: `bridge/mod.rs:33-35` — `email.contains('@') && email.contains('.')` — не валидная проверка email (пропустит `@.`, `a@b.` и т.д.)
- **Рекомендация**: Использовать regex или подключить crate вроде `email_address` для корректной проверки

#### 14. Нет `execution_id` в логах скриптов
- **Проблема**: `log()`, `log_warn()`, `log_error()` в `bridge/utils.rs` не включают execution_id или script_name в лог-сообщения. При множестве скриптов невозможно отследить, какой скрипт вывел сообщение.
- **Рекомендация**: Передавать script_name и execution_id через Scope и включать в target/span

#### 15. `take_changes()` не очищает changes
- **Проблема**: `proxy.rs:72-74` — `take_changes()` просто вызывает `changes()` (clone), а не `std::mem::take` как подразумевает имя.
- **Рекомендация**: Переименовать в `get_changes()` или реализовать настоящий take (move + clear)

#### 16. Отсутствует `ScriptQuery::All`
- **Проблема**: Нет способа запросить все скрипты без фильтра по статусу. GraphQL query `scripts` без параметра `status` по умолчанию возвращает только `Active`.
- **Рекомендация**: Добавить `ScriptQuery::All` или сделать GraphQL default `None` (все скрипты)

---

## Что может делать (текущие возможности)

1. Создавать/редактировать Rhai-скрипты через GraphQL admin API
2. Привязывать скрипты к событиям сущностей (before_create, after_update, etc.)
3. Запускать валидацию данных перед сохранением (abort для отклонения)
4. Модифицировать поля сущности из скрипта (entity["field"] = value)
5. Выполнять скрипты вручную через `runScript` mutation
6. Управлять жизненным циклом: draft → active → paused → disabled → archived
7. Автоматически отключать сбоящие скрипты
8. Определять cron-триггеры (модель готова, runtime не подключён)
9. Определять API endpoint триггеры (модель готова, routing не подключён)

## Чего не может / что недореализовано

1. Cron-scheduler не запущен (код есть, но не wired в app.rs)
2. REST API router не подключён (только GraphQL работает)
3. Нет HTTP bridge для вызова внешних API из скриптов
4. Нет DB bridge для запросов к БД из скриптов
5. Нет audit log (история выполнений)
6. Нет tenant isolation (мультитенантность)
7. Нет versioning скриптов с rollback
8. Нет debug mode / step execution
9. Нет метрик и трейсинга

---

## Файлы для модификации (при реализации рекомендаций)

| # | Файл | Описание изменений |
|---|------|-------------------|
| 1 | `crates/alloy-scripting/src/api/handlers.rs` | Исправить validate_script, убрать дублирование json_to_dynamic |
| 2 | `crates/alloy-scripting/src/runner/orchestrator.rs` | Добавить обработку Failed в run_before |
| 3 | `crates/alloy-scripting/src/bridge/mod.rs` | Улучшить validate_email |
| 4 | `crates/alloy-scripting/src/bridge/utils.rs` | Добавить execution context в логи |
| 5 | `crates/alloy-scripting/src/model/proxy.rs` | Исправить take_changes() |
| 6 | `crates/alloy-scripting/src/storage/traits.rs` | Добавить tenant_id support |
| 7 | `crates/alloy-scripting/src/migration.rs` | Добавить tenant_id column |
| 8 | `apps/server/src/graphql/alloy/mod.rs` | Вынести json_to_dynamic в shared utils |
| 9 | `apps/server/src/graphql/alloy/query.rs` | Исправить default filter, DB pagination |
| 10 | `apps/server/src/app.rs` | Подключить Scheduler |
| 11 | `apps/server/src/modules/alloy.rs` | Улучшить health check |
| 12 | Новый: `crates/alloy-scripting/src/utils.rs` | Shared json_to_dynamic/dynamic_to_json |

---

## Интеграция Alloy + MCP (ключевая стратегическая рекомендация)

### Текущее состояние MCP (`rustok-mcp`)

`rustok-mcp` — MCP-сервер на базе `rmcp 0.16`, работающий через stdio transport. Текущие инструменты **только для чтения метаданных** модулей:

| Tool | Назначение |
|------|-----------|
| `list_modules` | Список всех зарегистрированных модулей |
| `query_modules` | Фильтрация модулей (slug_prefix, dependency, pagination) |
| `module_exists` | Проверка существования модуля по slug |
| `module_details` | Метаданные модуля по slug |
| `content_module` / `blog_module` / `forum_module` / `pages_module` | Быстрый доступ к конкретным модулям |
| `mcp_health` | Проверка здоровья MCP-сервера |

**Критический gap**: Нет ни одного MCP-tool для работы с данными, скриптами или бизнес-логикой. MCP сервер полностью read-only.

### Видение: AI-программируемая бизнес-логика

Alloy + MCP = AI-модель (Claude/GPT/etc.) может через MCP-протокол:

1. **Создавать скрипты** — анализировать структуру данных и генерировать Rhai-скрипты валидации/трансформации
2. **Мигрировать данные** — писать migration-скрипты для массовых изменений данных
3. **Генерировать бизнес-логику** — создавать event-hooks, cron-задачи, API endpoints на лету
4. **Адаптироваться под данные** — читать схему/сущности, подстраивать скрипты под реальные поля и типы

### Предлагаемые MCP-tools для Alloy

#### Уровень 1: Управление скриптами

| Tool | Описание | Параметры |
|------|----------|-----------|
| `alloy_list_scripts` | Список скриптов с фильтрацией | `status`, `trigger_type`, `entity_type` |
| `alloy_get_script` | Получить скрипт по имени/id | `name` или `id` |
| `alloy_create_script` | Создать новый скрипт | `name`, `code`, `trigger`, `description` |
| `alloy_update_script` | Обновить код/триггер скрипта | `id`, `code`, `trigger`, `status` |
| `alloy_delete_script` | Удалить скрипт | `id` |
| `alloy_validate_script` | Проверить синтаксис Rhai-кода | `code` |

#### Уровень 2: Выполнение

| Tool | Описание | Параметры |
|------|----------|-----------|
| `alloy_run_script` | Запустить скрипт вручную | `name`, `params`, `entity` |
| `alloy_dry_run` | Тестовый запуск без side-effects | `code`, `entity_data` |
| `alloy_execution_log` | История выполнений скрипта | `script_id`, `limit` (требует audit log) |

#### Уровень 3: Introspection (адаптация под данные)

| Tool | Описание | Параметры |
|------|----------|-----------|
| `alloy_list_entity_types` | Все типы сущностей в системе | — |
| `alloy_entity_schema` | Поля и типы сущности | `entity_type` |
| `alloy_list_events` | Все доступные события по entity_type | `entity_type` |
| `alloy_script_helpers` | Список доступных helper-функций | `phase` |

#### Уровень 4: Миграции данных

| Tool | Описание | Параметры |
|------|----------|-----------|
| `alloy_create_migration` | Создать migration-скрипт | `name`, `code`, `description` |
| `alloy_run_migration` | Запустить миграцию данных | `name`, `dry_run` |
| `alloy_migration_status` | Статус миграций | — |

### Архитектура интеграции

```
AI Model (Claude) ──MCP──▶ rustok-mcp (RusToKMcpServer)
                                │
                                ├── Module tools (существующие)
                                │
                                └── Alloy tools (новые) ──▶ alloy-scripting
                                        │
                                        ├── ScriptRegistry (CRUD)
                                        ├── ScriptOrchestrator (Run)
                                        └── ScriptEngine (Validate/Compile)
```

### Необходимые изменения для интеграции

| Файл | Изменение |
|------|-----------|
| `crates/rustok-mcp/Cargo.toml` | Добавить `alloy-scripting` в зависимости |
| `crates/rustok-mcp/src/tools.rs` | Добавить alloy tool definitions, request/response types |
| `crates/rustok-mcp/src/server.rs` | Добавить `AlloyState` в `McpState`, обработчики alloy tools в `call_tool` |
| `crates/rustok-mcp/src/lib.rs` | Экспортировать новые alloy-tools |
| `crates/alloy-scripting/src/lib.rs` | Убедиться что публичный API достаточен для MCP |

### Безопасность MCP+Alloy

- MCP-tool `alloy_create_script` и `alloy_run_script` должны требовать **admin-level** авторизацию
- `alloy_dry_run` — sandbox execution без записи в БД, без side-effects
- Лимиты Rhai (max_operations, timeout) защищают от генерации AI бесконечных циклов
- Audit log обязателен: кто (AI/user), когда, какой скрипт создал/запустил

---

## Верификация

- `cargo test -p alloy-scripting` — запустить существующие тесты
- `cargo check -p alloy-scripting` — проверить компиляцию после изменений
- `cargo clippy -p alloy-scripting` — линтинг
- `cargo test -p rustok-server` — проверить интеграцию с сервером
- `cargo test -p rustok-mcp` — проверить MCP тесты
