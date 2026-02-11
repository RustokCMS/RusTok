# rustok-pages

Pages and menus domain logic for RusToK.

## Назначение
`rustok-pages` — модуль для управления статическими страницами, блоками и меню в CMS-части RusToK.

## Что делает
- Управляет страницами, блоками и меню.
- Использует `rustok-content` как слой хранения.
- Публикует события изменений через `TransactionalEventBus`.

## Как работает (простыми словами)
1. API обращается к сервисам страниц/блоков/меню.
2. Сервисы сохраняют данные через `NodeService`.
3. События отправляются через `TransactionalEventBus` из `rustok-outbox` для надёжной доставки.

This is an alpha version and requires clarification. Be careful, there may be errors in the text. So that no one thinks that this is an immutable rule.

## Взаимодействие
- crates/rustok-core
- crates/rustok-content
- crates/rustok-outbox (TransactionalEventBus)
- crates/rustok-index

## Документация
- Локальная документация: `./docs/`
- Общая документация платформы: `/docs`

## Паспорт компонента
- **Роль в системе:** Доменный модуль страниц и меню для CMS-части RusToK.
- **Основные данные/ответственность:** бизнес-логика и API данного компонента; структура кода и документации в корне компонента.
- **Взаимодействует с:**
  - crates/rustok-core
  - crates/rustok-content
  - crates/rustok-outbox (TransactionalEventBus)
  - crates/rustok-index
- **Точки входа:**
  - `crates/rustok-pages/src/lib.rs`
- **Локальная документация:** `./docs/`
- **Глобальная документация платформы:** `/docs/`

