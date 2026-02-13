# Event transport в `apps/server`

## Что это

`apps/server` публикует доменные события через общий `EventBus`, а далее пересылает их в настроенный
`EventTransport`.

Схема:

1. Сервисы модулей (`rustok-content`, `rustok-commerce`, `rustok-forum` и т.д.) публикуют событие в `EventBus`.
2. В `apps/server` поднимается фоновый forwarder, который читает события из общего `EventBus`.
3. Forwarder отправляет их в выбранный транспорт (`memory | outbox | iggy`).
4. В `outbox` режиме дополнительно запускается relay worker, который выгружает события из outbox.

## Конфигурация

Настройка находится в `settings.rustok.events`:

- `transport`: `memory | outbox | iggy`
- `relay_interval_ms`: интервал для outbox relay worker
- `iggy`: вложенная конфигурация `IggyConfig`

Пример:

```yaml
settings:
  rustok:
    events:
      transport: outbox
      relay_interval_ms: 1000
```

Можно переопределить через переменную окружения:

```bash
RUSTOK_EVENT_TRANSPORT=memory
```

Если указано некорректное значение, сервер падает на старте с ошибкой:

`Invalid RUSTOK_EVENT_TRANSPORT='...' Expected one of: memory, outbox, iggy`

## Где в коде

- settings: `apps/server/src/common/settings.rs`
- фабрика транспорта: `apps/server/src/services/event_transport_factory.rs`
- общий event bus и forwarder: `apps/server/src/services/event_bus.rs`
- подключение в runtime: `apps/server/src/app.rs`

## Ограничения текущей реализации

- В `outbox` режиме relay сейчас отправляет из outbox в `MemoryTransport` (локальная цель по умолчанию).
  Для production-streaming сценариев нужно настроить целевой transport (например Iggy) для relay chain.
- Если transport не инициализирован, `EventBus` продолжает работать in-memory (с warning в логах).
