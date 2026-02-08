# rustok-outbox

## Назначение
`rustok-outbox` — надежный транспорт событий уровня L1. Сохраняет события в БД (таблица `sys_events`) и доставляет их асинхронно.

## Что делает
- Пишет события в таблицу `sys_events` транзакционно.
- Готовит базу для фоновой доставки (relay).
- Позволяет гарантировать доставку между транзакциями.

## Как работает (простыми словами)
1. Сервис пишет данные в БД.
2. В той же транзакции записывается событие в `sys_events`.
3. Фоновый worker читает pending события и отправляет дальше.

## Ключевые компоненты
- `entity.rs` — модель `sys_events`.
- `transport.rs` — `OutboxTransport` (EventTransport).
- `relay.rs` — фоновая доставка (worker).
- `migration.rs` — миграция таблицы.

## Кому нужен
Продакшену на одном узле, когда in-memory событий недостаточно, но полноценный стриминг ещё не нужен.

This is an alpha version and requires clarification. Be careful, there may be errors in the text. So that no one thinks that this is an immutable rule.
