# utoipa-swagger-ui-vendored

## Назначение
`crates/utoipa-swagger-ui-vendored` — модуль/приложение RusToK. Здесь находится его код и корневая документация.

## Взаимодействие
- apps/server
- OpenAPI/Swagger UI endpoints
- tooling для API документации

## Документация
- Локальная документация: `./docs/`
- Общая документация платформы: `/docs`

## Паспорт компонента
- **Роль в системе:** Вендорный crate Swagger UI для публикации OpenAPI документации API.
- **Основные данные/ответственность:** бизнес-логика и API данного компонента; структура кода и документации в корне компонента.
- **Взаимодействует с:**
  - apps/server
  - utoipa/OpenAPI pipeline
  - инструменты dev/documentation
- **Точки входа:**
  - `crates/utoipa-swagger-ui-vendored/src/lib.rs`
- **Локальная документация:** `./docs/`
- **Глобальная документация платформы:** `/docs/`

