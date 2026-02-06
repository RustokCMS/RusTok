# rustok-iggy

## Назначение
`rustok-iggy` — транспорт событий уровня L2 (стриминг + replay). Заменяет стандартную очередь на Iggy
и использует отдельный connector-модуль для переключения embedded/remote режимов.

> **Статус:** начальная реализация. Connector-слой сейчас использует заглушки
> (logging-only) и требует замены на полноценный SDK/embedded runtime.

## Что делает
- Поддерживает Embedded и Remote режимы.
- Создаёт топологию потоков и топиков автоматически.
- Обеспечивает строгий порядок по `tenant_id`.
- Поддерживает JSON (по умолчанию) и Bincode (для high-throughput).

## Как работает (простыми словами)
1. IggyTransport сериализует событие и выбирает топик.
2. Партиционирование по `tenant_id` гарантирует порядок.
3. Топология (stream + topics) создаётся автоматически.
4. Для масштабирования используются consumer groups.

## Ключевые компоненты
- `config.rs` — режимы, топология, retention, сериализация.
- `transport.rs` — EventTransport поверх Iggy.
- `rustok-iggy-connector` — embedded/remote connector реализации.
- `topology.rs` — создание stream/topics.
- `serialization.rs` — JSON/Bincode сериализация.
- `consumer.rs` — управление группами.
- `dlq.rs` — dead-letter очередь.

## Кому нужен
Крупным инсталляциям, где нужны стриминг, replay и горизонтальное масштабирование потребителей.
