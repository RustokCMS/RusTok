# rustok-tenant

Multi-tenancy helpers and tenant metadata for RusToK.


## Взаимодействие
- crates/rustok-core
- crates/rustok-content/commerce/blog
- apps/server

## Документация
- Локальная документация: `./docs/`
- Общая документация платформы: `/docs`

## Паспорт компонента
- **Роль в системе:** Модуль tenant-контекста и мультиарендности на уровне платформы.
- **Основные данные/ответственность:** бизнес-логика и API данного компонента; структура кода и документации в корне компонента.
- **Взаимодействует с:**
  - crates/rustok-core
  - доменные модули (content/commerce/blog)
  - apps/server
- **Точки входа:**
  - `crates/rustok-tenant/src/lib.rs`
- **Локальная документация:** `./docs/`
- **Глобальная документация платформы:** `/docs/`

