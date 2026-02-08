# rustok-index

## Назначение
`rustok-index` — модуль CQRS/read-model. Делает быстрые индексы для поиска и витрины.

## Что делает
- Слушает события домена.
- Обновляет денормализованные таблицы индекса.
- Готовит данные для быстрого чтения.

## Как работает (простыми словами)
1. Модуль получает событие (например, `ProductUpdated`).
2. Из основной БД собирается нужная информация.
3. Записывается в индексные таблицы для быстрого поиска.

## Ключевые компоненты
- `handlers/` — обработчики событий.
- `services/` — пересборка и обновление индексов.

## События, которые триггерят пересборку
- `ProductCreated`, `ProductUpdated`, `ProductPublished`, `ProductDeleted`.
- `VariantCreated`, `VariantUpdated`, `VariantDeleted`.
- `InventoryUpdated`, `PriceUpdated` (ожидается `product_id` в payload для быстрого реиндекса).
- `ReindexRequested` (массовая пересборка или точечный реиндекс).

## Кому нужен
Поиску, витрине, любым read-heavy запросам.

This is an alpha version and requires clarification. Be careful, there may be errors in the text. So that no one thinks that this is an immutable rule.
