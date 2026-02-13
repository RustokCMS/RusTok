# rustok-iggy-connector

## Назначение
`crates/rustok-iggy-connector` — модуль/приложение RusToK. Здесь находится его код и корневая документация.

## Взаимодействие
- crates/rustok-iggy
- apps/server
- внешний Iggy runtime/cluster

## Документация
- Локальная документация: `./docs/`
- Общая документация платформы: `/docs`

## Паспорт компонента
- **Роль в системе:** Коннектор к Iggy runtime (embedded/remote) и lifecycle-обвязка.
- **Основные данные/ответственность:** бизнес-логика и API данного компонента; структура кода и документации в корне компонента.
- **Взаимодействует с:**
  - crates/rustok-iggy
  - apps/server
  - внешний Iggy cluster/runtime
- **Точки входа:**
  - `crates/rustok-iggy-connector/src/lib.rs`
- **Локальная документация:** `./docs/`
- **Глобальная документация платформы:** `/docs/`

