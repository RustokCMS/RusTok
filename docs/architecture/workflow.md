# Workflow — визуальная автоматизация на платформенной очереди

> Статус: реализован и интегрирован в live runtime  
> Модуль: `rustok-workflow`  
> Вид: `ModuleKind::Optional`

---

## 1. Что это

Workflow — это платформенный модуль автоматизаций в стиле n8n / Directus Flows:

- использует существующую событийную инфраструктуру платформы;
- не поднимает отдельную очередь или отдельный event loop;
- изолирован по tenant и защищён через платформенный RBAC;
- может оркестрировать вызовы Alloy, HTTP, notify и публикацию событий.

---

## 2. Runtime-модель

```text
DomainEvent / cron / webhook / manual trigger
       ↓
WorkflowTriggerHandler / WorkflowCronScheduler
       ↓
WorkflowService + WorkflowEngine
       ↓
Linear step chain
       ↓
Step outputs / execution logs / optional emitted events
```

Ключевые свойства текущего runtime:

- event trigger path работает через платформенный event dispatcher;
- cron trigger path работает через `WorkflowCronScheduler`;
- executions и step executions сохраняются в модульные таблицы;
- workflow не зависит от конкретного event transport implementation, а использует общую платформенную событийную модель.

---

## 3. Триггеры

Поддерживаемые trigger-режимы:

| Тип | Источник |
|-----|----------|
| `event` | Платформенное доменное событие |
| `cron` | Планировщик `WorkflowCronScheduler` |
| `webhook` | Входящий платформенный HTTP/webhook trigger |
| `manual` | Явный запуск из UI/API |

Webhook и versioning больше не являются future-идеями: они входят в текущий модульный контракт.

---

## 4. Типы шагов

Текущий набор step types:

| Тип | Назначение |
|-----|------------|
| `action` | Платформенное/модульное действие |
| `emit_event` | Публикация события |
| `condition` | Ветвление по данным контекста |
| `delay` | Отложенное выполнение |
| `http` | Внешний HTTP-вызов |
| `alloy_script` | Шаг с интеграцией Alloy |
| `notify` | Уведомление |

Текущая execution model остаётся линейной. Полноценный DAG — это отдельный будущий шаг, не часть текущего live contract.

---

## 5. Модель данных

Live schema модуля включает:

### `workflows`

- `tenant_id`
- `name`
- `description`
- `status`
- `trigger_config`
- `created_by`
- `failure_count`
- `auto_disabled_at`
- `webhook_slug`
- `webhook_secret`
- `created_at`
- `updated_at`

### `workflow_steps`

- `workflow_id`
- `position`
- `step_type`
- `config`
- `on_error`
- `timeout_ms`

### `workflow_executions`

- `workflow_id`
- `tenant_id`
- `trigger_event_id`
- `status`
- `context`
- `error`
- `started_at`
- `completed_at`

### `workflow_step_executions`

- `execution_id`
- `step_id`
- `status`
- `input`
- `output`
- `error`
- `started_at`
- `completed_at`

### `workflow_versions`

- `workflow_id`
- `version`
- `snapshot`
- `created_by`
- `created_at`

---

## 6. Интеграция с платформой

### События

- `WorkflowTriggerHandler` подписывается на платформенные события;
- matching workflows ищутся по tenant и trigger config;
- шаги могут эмитить события обратно в платформенный event runtime.

### Планировщик

- `WorkflowCronScheduler` запускается в server runtime;
- cron workflows обрабатываются без отдельной системы очередей.

### RBAC

Используются ресурсы:

- `Workflows`: `Create`, `Read`, `Update`, `Delete`, `List`, `Execute`, `Manage`
- `WorkflowExecutions`: `Read`, `List`

### Tenant isolation

Все модульные таблицы tenant-aware. Выполнения и triggers работают в tenant scope.

---

## 7. Связь с Alloy

Workflow и Alloy решают разные задачи:

- Workflow оркестрирует процесс;
- Alloy исполняет произвольную логику внутри шага.

Пример:

```text
Trigger: order.paid
  -> alloy_script: generate invoice payload
  -> notify: send customer message
  -> http: sync downstream CRM
```

Важно: интеграция Alloy уже является частью живого workflow contract, но некоторые step implementations всё ещё могут иметь backlog на углубление поведения, тестов или интеграционных адаптеров.

---

## 8. UI

Primary admin UI для workflow уже присутствует в `apps/admin`:

- список workflows;
- detail page;
- step editor;
- execution history;
- version history;
- template gallery;
- module navigation integration.

Workflow больше не должен описываться как “только будущий визуальный редактор”.

---

## 9. Что уже закрыто

- модуль зарегистрирован в `apps/server`;
- feature-gated GraphQL query/mutation подключены в общей схеме;
- базовые entities и migrations реализованы;
- event trigger и cron scheduler интегрированы в runtime;
- versioning и webhook fields присутствуют в схеме;
- Leptos admin UI присутствует в живом приложении.

---

## 10. Что остаётся будущим scope

Следующие вещи не стоит описывать как полностью закрытые:

- более глубокая реализация отдельных step adapters (`alloy_script`, `notify`) там, где они пока работают через abstraction/stub contracts;
- расширение linear execution model до DAG;
- более широкий набор интеграционных тестов с реальной БД/runtime evidence;
- расширение observability/system events вокруг `workflow.execution.*`, если это ещё не оформлено как жёсткий платформенный контракт.

---

## 11. Связанные документы

- [crates/rustok-workflow/docs/README.md](../../crates/rustok-workflow/docs/README.md)
- [events.md](./events.md)
- [event-flow-contract.md](./event-flow-contract.md)
- [modules.md](./modules.md)
