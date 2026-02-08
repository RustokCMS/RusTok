**Это альфа версия и требует уточнения. Будьте внимательны, в тексте могут быть ошибки. Чтобы никто не думал, что это незыблемое правило.**

# Мини-киты для админки и фронтенда

Цель: держать **минимальные** внутренние библиотеки, которые фиксируют общий
контракт между Leptos и Next.js, чтобы не дублировать код на страницах.

Охват приложений (обязательно учитываем при разработке, чтобы не плодить самопис):
- `apps/admin` (Leptos CSR)
- `apps/next-admin` (Next.js)
- `apps/storefront` (Leptos SSR)
- `apps/next-frontend` (Next.js)

## Уже есть

### 1) Leptos Auth
- Контракт для `/api/auth/*`
- Ключи хранения, маппинг ошибок
- Rust + TS реализации

### 2) Leptos GraphQL
- Контракт для `/api/graphql`
- Заголовки tenant/auth
- Rust + TS реализации

### 3) Leptos Hook Form
- Контракт для состояния формы (submitting/errors)
- Единая структура `field_errors`
- Rust + TS реализации

### 4) Leptos Zod
- Контракт для ошибок валидации (zod‑style)
- Стабильная форма ошибок для UI
- Rust + TS реализации

### 5) Leptos Table
- Контракт для таблиц (pagination/sort/filter)
- Единые структуры `TableState`, `SortRule`, `FilterRule`
- Rust + TS реализации

### 6) Leptos Zustand
- Контракт для стор‑снимков и обновлений
- Упрощённый parity со `zustand` в Next.js
- Rust + TS реализации

## Что ещё можно закрывать мини-китами (по мере надобности)

### Формы и валидация
- **Next.js:** react-hook-form + zod
- **Leptos:** покрыто `leptos-hook-form` + `leptos-zod` (контракт FormState + errors)

### Таблицы
- **Next.js:** @tanstack/react-table
- **Leptos:** покрыто `leptos-table` (контракт pagination/sort/filter)

### State
- **Next.js:** zustand
- **Leptos:** покрыто `leptos-zustand` (контракт store snapshots/updates)

## Принцип расширения

1. Сначала фиксируем контракт и минимальный API.
2. Реализуем его в Rust и TS.
3. Подключаем на 1–2 экранах, только затем масштабируем.
