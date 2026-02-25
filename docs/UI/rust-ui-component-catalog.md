# Rust/UI Component Catalog

Список компонентов и решения по их реализации в IU-системе.

**Легенда:**
- `adopt` — реализован в `iu-leptos` / `UI/next/components/`, используется в продакшне
- `pilot` — экспериментально реализован или планируется
- `defer` — отложено (нет текущей потребности)
- `reject` — не нужен (дублирует другое, не подходит архитектурно)

## Статус компонентов

| Component | Decision | Target crate / file | Notes |
|-----------|----------|---------------------|-------|
| Accordion | defer | — | Нет текущей потребности |
| Alert | pilot | `iu-leptos` + `UI/next` | Планируется рядом с Badge |
| Alert Dialog | pilot | `iu-leptos` + `UI/next` | Нужен для confirm-диалогов |
| Animate | defer | — | CSS transitions достаточно |
| Animate Group | defer | — | CSS transitions достаточно |
| AutoForm | defer | — | Используем react-hook-form / leptos-forms |
| Avatar | adopt | `UI/next/components/Avatar.tsx` | Реализован в Next.js; Leptos — `entities/user/ui` в admin |
| Badge | adopt | `UI/leptos/src/badge.rs`, `UI/next/components/Badge.tsx` | Полный паритет |
| Bottom Nav | defer | — | Не используется в текущих приложениях |
| Breadcrumb | adopt | `apps/next-admin/src/shared/ui/breadcrumbs.tsx` | Через shadcn |
| Button | adopt | `UI/leptos/src/button.rs`, `UI/next/components/Button.tsx` | Полный паритет |
| Button Action | defer | — | Используем Button с иконками |
| Button Group | defer | — | flex + gap покрывает потребность |
| Card | adopt | `crates/leptos-ui/src/card.rs` | Leptos-only; Next.js — shadcn напрямую |
| Card Carousel | defer | — | Нет текущей потребности |
| Checkbox | adopt | `UI/leptos/src/checkbox.rs`, `UI/next/components/Checkbox.tsx` | Полный паритет |
| Chips | defer | — | Badge с dismissible покрывает |
| Combobox | pilot | — | Планируется для filter-inputs |
| Command | adopt | `apps/next-admin` через kbar | KBar — command palette |
| Context Menu | defer | — | Нет текущей потребности |
| Data Table | adopt | `apps/next-admin/src/widgets/data-table/` | @tanstack/react-table + DataTable |
| Date Picker | defer | — | Используем react-day-picker / shadcn DatePicker |
| Dialog | adopt | shadcn `Modal` | `apps/next-admin/src/components/ui/modal.tsx` |
| Drag and Drop | defer | — | Нет текущей потребности |
| Drawer | defer | — | Нет текущей потребности |
| Dropdown Menu | adopt | shadcn `DropdownMenu` | В nav-user, org-switcher |
| Dropzone | adopt | `apps/next-admin/src/shared/ui/file-uploader.tsx` | react-dropzone |
| Empty | defer | — | inline JSX достаточно |
| Form | adopt | `apps/next-admin/src/shared/ui/forms/` | 10 form-field компонентов |
| Input | adopt | `UI/leptos/src/input.rs`, `UI/next/components/Input.tsx` | Полный паритет |
| Input Group | defer | — | prefix/suffix в Input достаточно |
| Input OTP | defer | — | Нет текущей потребности |
| Input Phone | defer | — | Нет текущей потребности |
| Item | defer | — | |
| Kbd | defer | — | inline `<kbd>` достаточно |
| Label | adopt | `crates/leptos-ui/src/label.rs` | Leptos-only; Next.js — shadcn |
| Marquee | defer | — | |
| MultiSelect | defer | — | Планируется через Combobox |
| Pagination | adopt | `apps/next-admin/src/widgets/data-table/` | DataTablePagination |
| Popover | adopt | shadcn `Popover` | В нескольких местах |
| Pressable | reject | — | Button покрывает |
| Radio Button | defer | — | Используем form-radio-group |
| Radio Button Group | adopt | `apps/next-admin/src/shared/ui/forms/` | FormRadioGroup |
| Scroll Area | adopt | shadcn `ScrollArea` | PageContainer |
| Select | adopt | `UI/leptos/src/select.rs`, `UI/next/components/Select.tsx` | Полный паритет |
| Separator | adopt | `crates/leptos-ui/src/separator.rs` | Leptos-only; Next.js — shadcn |
| Sheet | defer | — | Нет текущей потребности |
| Shimmer | defer | — | Skeleton достаточно |
| Skeleton | adopt | `UI/next/components/Skeleton.tsx` | Next.js only пока; Leptos — планируется |
| Slider | adopt | `apps/next-admin/src/shared/ui/forms/` | FormSlider |
| Sonner | adopt | shadcn `Sonner` | Toast-уведомления |
| Spinner | adopt | `UI/leptos/src/spinner.rs`, `UI/next/components/Spinner.tsx` | Полный паритет |
| Status | defer | — | Badge variants покрывают |
| Switch | adopt | `UI/leptos/src/switch.rs`, `UI/next/components/Switch.tsx` | Полный паритет |
| Table | adopt | shadcn `Table` + `widgets/data-table/` | DataTable |
| Tabs | adopt | shadcn `Tabs` | Используется в profile/settings |
| Textarea | adopt | `UI/leptos/src/textarea.rs`, `UI/next/components/Textarea.tsx` | Полный паритет |
| Theme Toggle | adopt | `apps/next-admin/src/shared/lib/themes/theme-mode-toggle.tsx` | |
| Toast | adopt | shadcn Sonner | |
| Tooltip | adopt | shadcn `Tooltip` | |

## Компоненты с полным паритетом Leptos ↔ Next.js

Следующие компоненты реализованы в `UI/leptos/src/` и `UI/next/components/` с одинаковым API:

- `Button` — `ButtonVariant`, `Size`, loading state
- `Input` — size variants, invalid state, reactive binding
- `Textarea` — size variants, invalid state
- `Select` — `SelectOption[]`, size variants
- `Checkbox` — controlled via `ReadSignal<bool>` / `checked`
- `Switch` — controlled via `ReadSignal<bool>` / `checked`, `SwitchSize`
- `Badge` — `BadgeVariant`, size, dismissible
- `Spinner` — size variants

## Компоненты только в Next.js (ещё не портированы в Leptos)

- `Avatar` — в `UI/next/components/Avatar.tsx`; в Leptos используется `entities/user/ui`
- `Skeleton` — в `UI/next/components/Skeleton.tsx`

## Следующие шаги (pilot)

1. `Alert` / `Alert Dialog` — нужны для подтверждений
2. `Combobox` / `MultiSelect` — для фильтрации данных
3. `Skeleton` в Leptos — для loading states в admin

---

Последнее обновление: 2026-02-25. Статус актуален на ветку `claude/review-fsd-admin-design-sOKKf`.
