# rustok-forum

## Назначение
`rustok-forum` — модуль форума: категории, темы, ответы, модерация и форумные workflow.

## Что делает
- Управляет категориями, темами и ответами.
- Использует `rustok-content` как слой хранения.
- Обеспечивает локализацию контента через `locale`.

## Как работает (простыми словами)
1. API обращается к сервису форума.
2. Сервис сохраняет данные через `NodeService`.
3. Событие отправляется в EventBus.

## Ключевые компоненты
- `constants.rs` — типы узлов форума.
- `dto/` — структуры запросов и ответов (включая `locale`).
- `services/` — категории, темы, ответы, модерация.
- `entities/` — модели SeaORM.

## Кому нужен
Форуму, комьюнити-разделам, поддержке и контентным обсуждениям.

This is an alpha version and requires clarification. Be careful, there may be errors in the text. So that no one thinks that this is an immutable rule.
