# Unified design system: shadcn/ui CSS vars for all apps

- Date: 2026-02-25
- Status: Accepted

## Context

RusToK имеет четыре UI-приложения на двух технологических стеках:

| Приложение | Стек | Дизайн-система до |
|-----------|------|-------------------|
| `apps/admin` | Leptos CSR | кастомные `--iu-*` CSS vars |
| `apps/next-admin` | Next.js / React | shadcn/ui |
| `apps/storefront` | Leptos SSR | DaisyUI |
| `apps/next-frontend` | Next.js / React | кастомный Button, хардкодированные цвета |

При ревью были заданы три вопроса:

1. Правильно ли унифицировали компоненты в админках?
2. Применим ли тот же подход на фронтендах?
3. Можно ли перенести весь shadcn/ui под Rust/Leptos?

### Проблемы старой реализации

**`apps/admin`**: использовал `--iu-*` CSS custom properties — собственная схема именования, несовместимая с shadcn. Визуальный паритет с `apps/next-admin` не достигался.

**`apps/storefront`**: DaisyUI — это старая зависимость от времени когда у проекта не было единой дизайн-системы. DaisyUI классы (`btn`, `badge-*`, `card-body`, `navbar`, `hero`, `stats`, `bg-base-*`) несовместимы с остальными приложениями. Leptos SSR не требует DaisyUI — рендеринг это HTML-строки, используются обычные Tailwind классы.

**`apps/next-frontend`**: хардкодированные цвета (`bg-sky-600`, `border-slate-200`, `text-slate-*`) — не следует ни одной системе.

## Decision

**Единая дизайн-система для всех четырёх приложений: shadcn CSS-переменные + Tailwind.**

DaisyUI полностью удалён из `apps/storefront`. Все четыре приложения:
- определяют shadcn-совместимый набор CSS custom properties (`--background`, `--foreground`, `--primary`, `--card`, `--muted`, `--accent`, `--destructive`, `--border`, `--input`, `--ring`, `--radius`)
- расширяют Tailwind через один и тот же паттерн `hsl(var(--name))`
- не используют хардкодированные цветовые значения

### Leptos-компоненты (`UI/leptos/src/`)

Реализованы как **прямой порт Tailwind-классов из shadcn/ui исходников**. Это обеспечивает визуальный паритет без зависимости от внешних Leptos UI-крейтов.

### Полный shadcn → Leptos порт — по требованию, не монолитом

| Категория | Подход |
|-----------|--------|
| Презентационные (Button, Badge, Input, Card, Label, Separator, Spinner) | ✅ Портированы |
| Простые интерактивные (Switch, Checkbox, Select, Textarea) | ✅ Портированы |
| Overlay (Dialog, Sheet, Popover, Tooltip) | Leptonic/Thaw → нативный порт по мере потребности |
| Сложные данные (Table, DatePicker, Combobox) | Leptonic/Thaw пока |
| Навигация (Breadcrumb, Tabs, Alert, Accordion) | Портировать нативно — CSS-only, несложно |

Приоритет следующих нативных портов:
1. `Alert` — CSS-only, нужен для информационных сообщений
2. `AlertDialog` — нужен для confirm-диалогов
3. `Tabs` — CSS + минимальный JS, нужен для страниц настроек
4. `Skeleton` — CSS-only, нужен для loading states

## Consequences

### Положительные
- Единая палитра и паттерн стилизации во всех четырёх приложениях
- Нет внешних UI-библиотек кроме Tailwind — нет versioning-рисков
- Leptos-компоненты визуально идентичны React/shadcn аналогам
- При обновлении shadcn в Next.js — обновляем только строки классов в Leptos

### Отрицательные / Trade-offs
- При обновлении shadcn в Next.js нужно вручную синхронизировать классы в Leptos-версиях
- Overlay-компоненты (Dialog и пр.) временно визуально расходятся (Leptonic/Thaw)

### Итоговое состояние после миграции

| Приложение | CSS-система | Статус |
|-----------|------------|--------|
| `apps/admin` | shadcn CSS vars + Tailwind | ✅ Мигрировано |
| `apps/next-admin` | shadcn/ui (React) | ✅ Было изначально |
| `apps/next-frontend` | shadcn CSS vars + Tailwind | ✅ Мигрировано |
| `apps/storefront` | shadcn CSS vars + Tailwind (SSR) | ✅ Мигрировано, DaisyUI удалён |
