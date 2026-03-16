# WebSocket Channels

Описание архитектуры real-time WebSocket-каналов платформы RusToK.

## Обзор

WebSocket-каналы реализованы поверх `axum::extract::ws` и интегрированы в Loco-сервер как обычные HTTP-маршруты, обновляемые до WebSocket-соединения (`WebSocketUpgrade`).

Каналы используются для:
- Стриминга событий сборки (build pipeline) в реальном времени
- Будущих уведомлений: медиа-обработка, импорт данных, прогресс задач

## Текущие каналы

### `GET /ws/builds`

Стримит события сборки (`BuildEvent`) от `BuildEventHub` всем подключённым клиентам.

**Аутентификация**: Bearer JWT в заголовке `Authorization` (тот же формат, что и REST-эндпоинты).

**Формат сообщений**: newline-delimited JSON с полем `type` для различения типов событий.

## Архитектура

```
Client
  │
  │  GET /ws/builds
  │  Upgrade: websocket
  ▼
ws_builds() handler
  │
  │  WebSocketUpgrade::on_upgrade()
  ▼
handle_socket(socket, hub)
  │
  ├── hub.subscribe()  ──► tokio::sync::broadcast::Receiver<BuildEvent>
  │
  └── tokio::select! loop
        ├── rx.recv()        → serialize + send JSON
        └── socket.recv()    → handle Close/Ping
```

### `BuildEventHub`

```rust
// apps/server/src/services/build_event_hub.rs
pub struct BuildEventHub {
    tx: tokio::sync::broadcast::Sender<BuildEvent>,
}

impl BuildEventHub {
    pub fn subscribe(&self) -> Receiver<BuildEvent> { ... }
    pub fn publish(&self, event: BuildEvent) { ... }
}
```

Хаб хранится в `ctx.shared_store` и инициализируется один раз при старте.

### Wire-format

Каждое событие сериализуется в JSON с тегом `type`:

```json
{ "type": "build_requested", "build_id": "...", "requested_by": "alice" }
{ "type": "build_started",   "build_id": "...", "stage": "compile", "progress": 10 }
{ "type": "build_progress",  "build_id": "...", "stage": "test",    "progress": 55 }
{ "type": "build_completed", "build_id": "...", "release_id": "v1.2.3" }
{ "type": "build_failed",    "build_id": "...", "stage": "deploy",  "progress": 90, "error": "..." }
{ "type": "build_cancelled", "build_id": "...", "stage": "compile", "progress": 20 }
```

Тег `type` генерируется через `#[serde(tag = "type", rename_all = "snake_case")]`.

### Обработка разрыва соединения

`handle_socket` завершается при:
- `RecvError::Closed` — хаб остановлен (graceful shutdown)
- `Message::Close` или `None` от клиента — клиент отключился
- Ошибка записи в сокет (`socket.send(...).is_err()`)

При отставании (`RecvError::Lagged(n)`) пропущенные события логируются через `tracing::warn!` — соединение не разрывается.

## Регистрация маршрутов

```rust
// apps/server/src/channels/builds.rs
pub fn routes() -> loco_rs::controller::Routes {
    loco_rs::controller::Routes::new()
        .prefix("ws")
        .add("/builds", get(ws_builds))
}
```

```rust
// apps/server/src/app.rs (в Hooks::routes())
routes = routes.add_route(channels::builds::routes());
```

Маршрут добавляется безусловно (не зависит от feature-флагов).

## Graceful Shutdown

При получении сигнала завершения:
1. Loco вызывает `Hooks::on_shutdown(ctx)`
2. `StopHandle::stop()` отправляет `true` через `watch::Sender<bool>`
3. Сервисы, слушающие `rx.changed()`, завершают свои циклы
4. `BuildEventHub` остаётся активным до закрытия последнего соединения — `RecvError::Closed` не возникает до дропа `Arc<BuildEventHub>`

## Добавление нового канала

1. Создайте `apps/server/src/channels/<name>.rs`
2. Определите enum wire-сообщений с `#[serde(tag = "type", rename_all = "snake_case")]`
3. Создайте хаб (или переиспользуйте существующий) через `shared_store`
4. Реализуйте `handle_socket` с `tokio::select!` по `rx.recv()` + `socket.recv()`
5. Экспортируйте `routes()` и добавьте в `app.rs`

## Зависимости

| Crate | Зачем |
|-------|-------|
| `axum` (feature `ws`) | `WebSocketUpgrade`, `WebSocket`, `Message` |
| `tokio::sync::broadcast` | Многоадресная рассылка событий |
| `serde_json` | Сериализация wire-сообщений |
| `tracing` | Логирование lag / ошибок |

## Связанные документы

- [Руководство по планировщику](../guides/scheduler.md)
- [Архитектура событий](./events.md)
- [Контракт потока событий](./event-flow-contract.md)
