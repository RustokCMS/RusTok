# Паритет библиотек для фронтендов (Leptos-first)

Этот документ фиксирует обязательный стек библиотек для наших фронтендов и приоритет на переиспользование готовых решений вместо самописа.

## Контекст

- **Описание:** единый набор библиотек для `apps/admin` (CSR) и `apps/storefront` (SSR).
- **Приоритет:** сначала фронтенды (storefront + admin), затем расширение в смежные приложения.
- **Ссылки:** [UI документы](./) • [UI parity](./ui-parity.md) • [IU библиотеки](../../IU/README.md)

## Согласованный core-стек для старта админки

Ниже — актуальный перечень, который мы согласовали для старта без лишнего усложнения.

| Категория | Библиотека/модуль | Что делает у нас |
| --- | --- | --- |
| UI core | `leptos`, `leptos_router` | Основа UI, реактивность и маршрутизация страниц админки |
| API transport | `leptos-graphql` | Внутренний GraphQL transport-слой (headers, extensions, error mapping) |
| HTTP/serialization | `reqwest`, `serde`, `serde_json` | Вызовы API и сериализация payload'ов |
| Auth | `leptos-auth` | Auth-flow для админки (наша Leptos-альтернатива NextAuth-подходу) |
| I18n | `leptos_i18n` | Многоязычность интерфейса |
| Metadata/SEO | `leptos-next-metadata` | Управление metadata/head в Next-like стиле |
| Forms | `leptos-hook-form`, `leptos-zod` | Управление формами и валидация |
| Tables/listing | `leptos-struct-table`, `leptos-shadcn-pagination` | Таблицы и пагинация для админских списков (из shadcn-слоя используем именно pagination crate) |
| Browser/reactive utils | `leptos-use` | LocalStorage/hooks/observer и прочие browser utilities |
| Styling | `tailwind-rs` | Tailwind pipeline для Leptos-UI |
| DX/debug | `console_error_panic_hook`, `console_log`, `log` | Диагностика ошибок и логирование в браузере |

## Что считаем опциональным

- `graphql-client` — подключаем в приложении, когда реально включаем typed `.graphql` codegen flow.
- `leptos-query` — подключаем точечно, когда нужен кэш/стратегии refetch/stale beyond базовых `Resource`/actions.
- `leptos-shadcn-ui` как полный набор примитивов сейчас **не является обязательным core**; в согласованном baseline используем `leptos-shadcn-pagination` и локальные UI-компоненты.

## Обязательный набор библиотек (подключаем в фронтендах)

| Категория | Библиотека | Где используем | Примечание |
| --- | --- | --- | --- |
| UI core | `leptos`, `leptos_router` | admin + storefront | Базовый UI и роутинг |
| Auth | `leptos-auth` | admin + storefront | Встроенная интеграция auth-flow |
| GraphQL transport | `leptos-graphql` | admin + storefront | Клиентский transport слой к `/api/graphql` |
| Forms | `leptos-hook-form` | admin + storefront | Единое состояние форм |
| Validation | `leptos-zod` | admin + storefront | Маппинг и формат ошибок валидации |
| Tables | `leptos-struct-table` | admin + storefront | Табличный UI-слой для Leptos |
| Pagination | `leptos-shadcn-pagination` | admin + storefront | Пагинация в shadcn-style |
| Local state | `leptos-zustand` | admin + storefront | Store snapshots/updates |
| Reactive/browser utils | `leptos-use` | admin + storefront | Подписки/observer/storage/events/debounce |
| I18n | `leptos_i18n` | admin + storefront | Мультиязычность (RU/EN и далее) |
| Metadata/SEO | `leptos-next-metadata` | storefront (+ при необходимости admin) | Next-like модель метаданных для Leptos |
| Async data/query | `leptos-query` | admin + storefront | Кэш, stale/refetch, query lifecycle |
| Styling pipeline | `tailwind-rs` | admin + storefront | TailwindCSS pipeline и токены |

## Правило для разработки

1. Перед новым UI-функционалом проверяем этот список и используем библиотеку из него.
2. Самопис допускается только если библиотека отсутствует или не закрывает критичный кейс.
3. Если добавляется новая библиотека, обновляем этот документ и `Cargo.toml` фронтендов в одном PR.

This is an alpha version and requires clarification. Be careful, there may be errors in the text. So that no one thinks that this is an immutable rule.

## Аудит паритета со starter `apps/next-admin` (2026-02)

Источник для сверки: `apps/next-admin/package.json`.
Цель: понять, что уже закрыто нашими библиотеками, а где нужен явный поиск/разработка замены.

### Матрица ключевых библиотек Next Starter → RusTok/Leptos

| Next starter библиотека | Назначение | Текущий статус в RusTok | Явная замена/действие |
| --- | --- | --- | --- |
| `react-hook-form` + `@hookform/resolvers` + `zod` | Формы и валидация | ✅ Закрыто | `leptos-hook-form` + `leptos-zod` |
| `@tanstack/react-table` | Таблицы | ✅ Закрыто | `leptos-struct-table` + `leptos-shadcn-pagination` |
| `zustand` | Локальные сторы | ✅ Закрыто | `leptos-zustand` |
| `recharts` | Графики на dashboard | ✅/⚠️ Частично | Основной путь: `leptos-chartistry`; если нужен parity по конкретному chart-type — фиксировать gap |
| `next-themes` | Тема/переключение dark-light | ✅ Закрыто | Theme слой через Leptos + Tailwind токены |
| `nuqs` | Синхронизация состояния с query params | ⚠️ Частично | `leptos_router` + `leptos-use` + утилиты сериализации; при сложных кейсах добавить shared helper crate |
| `sonner` | Toast-уведомления | ⚠️ Частично | Использовать существующий notification слой; при нехватке — сделать `leptos-sonner`-совместимый wrapper |
| `@dnd-kit/*` | Drag-and-drop (kanban) | ⚠️ Candidate found | Кандидат: `leptos_dnd` (`docs.rs/crate/leptos_dnd/0.1.4`). Проверить production-ready критерии; fallback: `crates/leptos-dnd` |
| `kbar` / `cmdk` | Command palette / быстрый поиск | ❌ Gap | Поиск готовой Leptos command palette; fallback: `crates/leptos-command-palette` |
| `react-dropzone` | Upload/dropzone | ⚠️ Gap (есть implementation guide) | Rust/UI Dropzone — это не готовая crates.io библиотека, а инструкция/шаблон компонента. Берем как reference implementation и выносим в `crates/leptos-dropzone`. |
| `react-day-picker` | Date picker/calendar | ⚠️ Частично | Проверить существующий date UI в `leptos-shadcn`; при отсутствии — выделить отдельный date-picker crate |
| `vaul` | Drawer/Sheet UX | ⚠️ Частично | Проверить покрытие текущими UI primitives; если не хватает — добавить shared drawer primitive |
| `@sentry/nextjs` | Monitoring/trace | ⚠️ Частично | Проверить текущий Rust/FE telemetry контур; для web-клиента зафиксировать единый Sentry adapter |
| `@clerk/nextjs` | Auth provider в шаблоне | ✅ Не переносим | В RusTok используем `leptos-auth` + backend `/api/auth/*` |

### Явные parity-gap задачи (чтобы не упустить)

1. **DnD/kanban gap**: выбрать библиотеку или создать `leptos-dnd`.
2. **Command palette gap**: выбрать библиотеку или создать `leptos-command-palette`.
3. **Dropzone gap**: выбрать библиотеку или создать `leptos-dropzone`.
4. **Date picker parity**: определить единый shared date picker.
5. **Toast parity**: формализовать единый notification API для Next и Leptos.
6. **URL-state parity (`nuqs`-подобно)**: добавить shared helper для query-state.
7. **Monitoring parity**: утвердить единый adapter для frontend telemetry/Sentry.

### Кандидаты из текущего обсуждения

- DnD: `leptos_dnd` — https://docs.rs/crate/leptos_dnd/0.1.4
- Dropzone: Rust UI Dropzone — https://www.rust-ui.com/docs/components/dropzone

### Важно: Rust/UI — это не "готовая библиотека", а система инструкций/дистрибуции компонентов

Согласно документации Rust/UI, это подход "how you build your component library", а не пакет, который просто ставится и импортируется.

Практическое правило для RusTok:

- Rust/UI используем как source шаблонов/паттернов и reference implementation.
- Production-реализацию компонентов размещаем в наших shared crates (`crates/*`).
- В parity-матрице Rust/UI не считаем прямой библиотечной заменой 1:1.

Мини-чек перед принятием в стек:

1. Поддержка текущей версии `leptos` в workspace.
2. Совместимость с SSR/CSR режимами там, где это нужно.
3. Состояние поддержки (релизы/issue-активность).
4. Отсутствие блокирующих лицензий/ограничений.
5. Возможность завернуть в наш shared API (без прямой привязки к app-слою).

### Rust UI каталог (`/docs/components`) — план проверки

Локальный рабочий snapshot каталога и таблица решений:

- [`docs/UI/rust-ui-component-catalog.md`](./rust-ui-component-catalog.md)
- Исходный snapshot навигации: [`docs/UI/rust-ui-nav-snapshot.txt`](./rust-ui-nav-snapshot.txt)
- Генератор: [`docs/UI/tools/rust_ui_catalog_parser.py`](./tools/rust_ui_catalog_parser.py)

В текущем container-сценарии прямой запрос к `https://www.rust-ui.com/docs/components/` блокируется сетью (`CONNECT tunnel failed, response 403`),
поэтому полный auto-audit каталога здесь не выполнен.

Что фиксируем как обязательный follow-up при доступной сети:

1. Снять список доступных Rust UI компонентов.
2. Сопоставить его с нашим gap-логом (DnD, Dropzone, Date picker, Drawer, Command palette, Toast).
3. Для каждого совпадения зафиксировать решение: `adopt` / `pilot` / `reject`.
4. Если `adopt`/`pilot` — переносим код компонента в наш shared crate и подключаем в оба UI (Next+Leptos).

Шаблон строки для gap-log:

- `component`: `<name>`
- `source`: `rust-ui`
- `decision`: `adopt|pilot|reject`
- `target crate`: `crates/<shared-lib>`
- `owner`: `<team/person>`
- `notes`: `compatibility, SSR/CSR, maintenance`

### Правило внедрения по gap-задачам

- Нельзя закрывать gap ad-hoc кодом в `apps/next-admin` или `apps/admin`.
- Каждая замена/реализация делается в shared crate (`crates/*`) и затем подключается в оба UI.
- Для каждой gap-задачи фиксируем: `owner`, `target crate`, `deadline`, `fallback`.

