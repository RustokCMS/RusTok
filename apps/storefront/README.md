# storefront

## Назначение
`apps/storefront` — модуль/приложение RusToK. Здесь находится его код и корневая документация.

## Взаимодействие
- apps/server (API)
- crates/rustok-commerce и crates/rustok-content через backend
- apps/admin (общая экосистема UI)

## Документация
- Локальная документация: `./docs/`
- Общая документация платформы: `/docs`

## Паспорт компонента
- **Роль в системе:** Leptos storefront-приложение (альтернативная/внутренняя витрина).
- **Основные данные/ответственность:** бизнес-логика и API данного компонента; структура кода и документации в корне компонента.
- **Взаимодействует с:**
  - apps/server (API)
  - crates/rustok-commerce и rustok-content через backend
  - apps/admin (единый контур управления)
- **Точки входа:**
  - `apps/storefront/src/main.rs`
  - `apps/storefront/src/pages/*`
- **Локальная документация:** `./docs/`
- **Глобальная документация платформы:** `/docs/`

