# leptos-graphql

## Назначение
`crates/leptos-graphql` — модуль/приложение RusToK. Здесь находится его код и корневая документация.

## Взаимодействие
- apps/admin
- apps/storefront
- apps/server (GraphQL endpoint)

## Документация
- Локальная документация: `./docs/`
- Общая документация платформы: `/docs`

## Паспорт компонента
- **Роль в системе:** Leptos-утилиты для GraphQL-запросов и интеграции с backend.
- **Основные данные/ответственность:** бизнес-логика и API данного компонента; структура кода и документации в корне компонента.
- **Взаимодействует с:**
  - apps/admin
  - apps/storefront
  - apps/server (GraphQL endpoint)
- **Точки входа:**
  - `crates/leptos-graphql/src/lib.rs`
- **Локальная документация:** `./docs/`
- **Глобальная документация платформы:** `/docs/`



## Практический подход (Leptos-way)
- Этот crate — тонкий transport/utils слой поверх `reqwest` + GraphQL payload/response.
- Управление async-состоянием (`loading/error/data`) остается в `leptos::Resource`/actions в приложениях.
- Для строгой типизации запросов можно подключать `graphql-client` в приложениях и отправлять сгенерированные payload через этот слой.


## Манифест библиотеки (`leptos-graphql`)

### Базовые battle-tested зависимости
- `reqwest` — HTTP transport к GraphQL endpoint (`POST /api/graphql`).
- `serde`, `serde_json` — сериализация GraphQL payload/response/`extensions`.
- `thiserror` — единый typed error mapping (`Network`, `Http`, `Unauthorized`, `Graphql`).

### Что делает crate
- Формирует стандартный GraphQL request shape (`query`, `variables`, `extensions`).
- Даёт helper для persisted query extensions (`sha256Hash`).
- Выполняет HTTP-запросы с заголовками авторизации и tenant-scope.

### Что crate намеренно **не** делает
- Не дублирует state-management UI клиента (это зона `leptos::Resource`/actions).
- Не является Apollo/urql-клоном с собственным кешем и lifecycle слоем.

### Типизация запросов (рекомендация)
- Для compile-time typed запросов подключается `graphql-client` **на уровне приложения**
  (`apps/admin`, `apps/storefront`) и отправляет сгенерированные payload через этот crate.
