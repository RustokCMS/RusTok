# rustok-telemetry

## Назначение
`crates/rustok-telemetry` — модуль/приложение RusToK. Здесь находится его код и корневая документация.

## Взаимодействие
- crates/rustok-core
- apps/server/apps/mcp
- внешние observability backends

## Документация
- Локальная документация: `./docs/`
- Общая документация платформы: `/docs`

## Паспорт компонента
- **Роль в системе:** Набор инициализации tracing/metrics и observability-интеграций.
- **Основные данные/ответственность:** бизнес-логика и API данного компонента; структура кода и документации в корне компонента.
- **Взаимодействует с:**
  - crates/rustok-core
  - apps/server/apps/mcp
  - внешние observability backends
- **Точки входа:**
  - `crates/rustok-telemetry/src/lib.rs`
- **Локальная документация:** `./docs/`
- **Глобальная документация платформы:** `/docs/`

