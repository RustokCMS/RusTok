# Design System Analysis: Atomic Design vs DSD for RusToK

**Дата:** 2026-02-13  
**Контекст:** Выбор дизайн-системы для `apps/admin` (Leptos + Tailwind)  
**Рассматриваемые подходы:** Atomic Design vs DSD (Design System Driven)

---

## 📊 Текущее состояние

### Что уже есть

```
apps/admin/src/components/
├── ui.rs                    ← Простые компоненты (Button, Input)
├── ui/
│   ├── page_header.rs      ← Композитный компонент
│   └── stats_card.rs       ← Композитный компонент
└── protected_route.rs       ← Feature компонент
```

**Проблемы:**
- ❌ Нет структуры (всё в одном `ui.rs`)
- ❌ Нет вариантов компонентов (размеры, цвета)
- ❌ Дублирование стилей (hardcoded Tailwind классы)
- ❌ Нет композиции (Button не использует variants)
- ❌ Нет accessibility (нет aria-labels, focus states)

**Хорошее:**
- ✅ Уже есть `leptos-shadcn-pagination` (DSD подход)
- ✅ Tailwind настроен
- ✅ Простые компоненты работают

---

## 🔬 Atomic Design (Brad Frost)

### Структура

```
components/
├── atoms/            ← Базовые элементы (Button, Input, Icon)
├── molecules/        ← Простые композиты (SearchBar = Input + Button)
├── organisms/        ← Сложные блоки (Header = Logo + Nav + Search)
├── templates/        ← Шаблоны страниц (без данных)
└── pages/            ← Готовые страницы (с данными)
```

### Пример для RusToK

```rust
// atoms/button.rs
#[component]
pub fn Button(
    children: Children,
    #[prop(optional)] variant: ButtonVariant,
    #[prop(optional)] size: ButtonSize,
) -> impl IntoView {
    let class = match (variant, size) {
        (ButtonVariant::Primary, ButtonSize::Md) => "bg-blue-600 px-4 py-2",
        (ButtonVariant::Secondary, ButtonSize::Sm) => "bg-gray-200 px-3 py-1",
        // ...
    };
    view! { <button class={class}>{children()}</button> }
}

// molecules/search_bar.rs
#[component]
pub fn SearchBar() -> impl IntoView {
    view! {
        <div class="flex gap-2">
            <Input placeholder="Search..." />
            <Button variant=ButtonVariant::Primary>
                <Icon icon=IconType::Search />
            </Button>
        </div>
    }
}

// organisms/page_header.rs
#[component]
pub fn PageHeader() -> impl IntoView {
    view! {
        <header>
            <Logo />
            <SearchBar />
            <UserMenu />
        </header>
    }
}
```

### Плюсы для RusToK

1. ✅ **Понятная иерархия** — легко найти компонент
2. ✅ **Переиспользование** — атомы используются везде
3. ✅ **Масштабируемость** — можно добавлять новые уровни
4. ✅ **Тестируемость** — тестируем атомы → автоматически тестируем молекулы
5. ✅ **Документация** — Storybook/mdBook легко структурировать

### Минусы для RusToK

1. ❌ **Избыточность** — слишком много папок для малого проекта
2. ❌ **Споры о категориях** — "SearchBar это молекула или организм?"
3. ❌ **Дублирование** — некоторые компоненты не вписываются в категории
4. ❌ **Overhead** — создание простого компонента требует много раздумий
5. ❌ **Не подходит для shadcn** — shadcn не следует этой структуре

---

## 🎨 DSD (Design System Driven) — shadcn подход

### Структура

```
components/
├── ui/                      ← Все UI компоненты (flat)
│   ├── button.rs           ← С вариантами (primary, secondary, ghost, link)
│   ├── input.rs            ← С вариантами (text, email, password)
│   ├── card.rs             ← Композитный (Card, CardHeader, CardContent)
│   ├── dialog.rs           ← Сложный (Dialog, DialogTrigger, DialogContent)
│   ├── table.rs            ← Data компонент
│   └── ...
├── features/               ← Фича-специфичные компоненты
│   ├── auth/
│   │   ├── login_form.rs
│   │   └── register_form.rs
│   ├── dashboard/
│   │   ├── stats_card.rs
│   │   └── recent_activity.rs
│   └── users/
│       ├── user_table.rs
│       └── user_filters.rs
└── layouts/                ← Шаблоны (сайдбар, header, footer)
    ├── admin_layout.rs
    ├── auth_layout.rs
    └── error_layout.rs
```

### Пример для RusToK

```rust
// components/ui/button.rs
#[derive(Clone, Copy)]
pub enum ButtonVariant {
    Default,    // bg-blue-600
    Destructive, // bg-red-600
    Outline,    // border-gray-300 bg-transparent
    Secondary,  // bg-gray-200
    Ghost,      // hover:bg-gray-100
    Link,       // text-blue-600 underline
}

#[derive(Clone, Copy)]
pub enum ButtonSize {
    Sm,  // px-3 py-1.5 text-sm
    Md,  // px-4 py-2 text-base
    Lg,  // px-6 py-3 text-lg
    Icon, // p-2 (квадратный)
}

#[component]
pub fn Button(
    children: Children,
    #[prop(default = ButtonVariant::Default)] variant: ButtonVariant,
    #[prop(default = ButtonSize::Md)] size: ButtonSize,
    #[prop(optional)] on_click: Option<Callback<web_sys::MouseEvent>>,
    #[prop(default = false)] disabled: bool,
    #[prop(optional, into)] class: String, // для кастомизации
) -> impl IntoView {
    let base = "inline-flex items-center justify-center rounded-lg font-medium transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50";
    
    let variant_class = match variant {
        ButtonVariant::Default => "bg-blue-600 text-white hover:bg-blue-700 focus-visible:ring-blue-500",
        ButtonVariant::Destructive => "bg-red-600 text-white hover:bg-red-700 focus-visible:ring-red-500",
        ButtonVariant::Outline => "border border-gray-300 bg-transparent hover:bg-gray-100 focus-visible:ring-gray-500",
        ButtonVariant::Secondary => "bg-gray-200 text-gray-900 hover:bg-gray-300 focus-visible:ring-gray-500",
        ButtonVariant::Ghost => "hover:bg-gray-100 focus-visible:ring-gray-500",
        ButtonVariant::Link => "text-blue-600 underline-offset-4 hover:underline",
    };
    
    let size_class = match size {
        ButtonSize::Sm => "h-8 px-3 text-sm",
        ButtonSize::Md => "h-10 px-4 text-base",
        ButtonSize::Lg => "h-12 px-6 text-lg",
        ButtonSize::Icon => "h-10 w-10",
    };
    
    let merged = format!("{} {} {} {}", base, variant_class, size_class, class);
    
    view! {
        <button
            class={merged}
            disabled={disabled}
            on:click=move |ev| {
                if let Some(handler) = on_click {
                    handler.run(ev);
                }
            }
        >
            {children()}
        </button>
    }
}

// components/ui/card.rs
#[component]
pub fn Card(children: Children, #[prop(optional, into)] class: String) -> impl IntoView {
    view! {
        <div class=format!("rounded-xl border bg-white shadow-sm {}", class)>
            {children()}
        </div>
    }
}

#[component]
pub fn CardHeader(children: Children) -> impl IntoView {
    view! { <div class="flex flex-col space-y-1.5 p-6">{children()}</div> }
}

#[component]
pub fn CardTitle(children: Children) -> impl IntoView {
    view! { <h3 class="text-2xl font-semibold leading-none">{children()}</h3> }
}

#[component]
pub fn CardContent(children: Children) -> impl IntoView {
    view! { <div class="p-6 pt-0">{children()}</div> }
}

// Использование:
view! {
    <Card>
        <CardHeader>
            <CardTitle>"Total Users"</CardTitle>
        </CardHeader>
        <CardContent>
            <p class="text-3xl font-bold">"1,234"</p>
        </CardContent>
    </Card>
}

// components/features/dashboard/stats_card.rs
#[component]
pub fn StatsCard(
    title: String,
    value: String,
    change: Option<String>,
) -> impl IntoView {
    view! {
        <Card>
            <CardHeader>
                <CardTitle>{title}</CardTitle>
            </CardHeader>
            <CardContent>
                <div class="flex items-end justify-between">
                    <p class="text-3xl font-bold">{value}</p>
                    {change.map(|c| view! { <span class="text-sm text-green-600">{c}</span> })}
                </div>
            </CardContent>
        </Card>
    }
}
```

### Плюсы для RusToK

1. ✅ **Совместимость с shadcn** — можно портировать компоненты 1:1
2. ✅ **Flat структура** — легко найти компонент (всё в `ui/`)
3. ✅ **Композиция** — Card + CardHeader + CardContent
4. ✅ **Варианты** — `variant`, `size` props
5. ✅ **Кастомизация** — `class` prop для override
6. ✅ **Copy-paste friendly** — можно скопировать компонент и изменить
7. ✅ **Accessibility** — focus-visible, disabled states, aria-*
8. ✅ **Меньше раздумий** — просто кладём всё в `ui/`

### Минусы для RusToK

1. ❌ **Большая папка `ui/`** — может быть 50+ файлов
2. ❌ **Нет чёткой иерархии** — Button и Dialog на одном уровне
3. ❌ **Меньше переиспользования** — компоненты более самодостаточные

---

## 🎯 Рекомендация для RusToK

### ✅ **Выбираем DSD (shadcn подход)**

**Причины:**

1. **Вы уже используете shadcn** — `leptos-shadcn-pagination` есть в workspace
2. **Быстрая разработка** — меньше раздумий "куда положить компонент"
3. **Leptos экосистема** — большинство UI библиотек для Leptos следуют DSD
4. **Портирование из React** — можно брать shadcn/ui компоненты и портировать
5. **Tailwind** — DSD лучше работает с utility-first CSS
6. **Малый проект** — для RusToK не нужна сложность Atomic Design

---

## 📁 Предлагаемая структура для RusToK

```
apps/admin/src/
├── components/
│   ├── ui/                          ← Все UI компоненты (shadcn-style)
│   │   ├── mod.rs                  ← Re-exports
│   │   ├── button.rs               ← Button с вариантами
│   │   ├── input.rs                ← Input с вариантами
│   │   ├── card.rs                 ← Card + CardHeader + CardTitle + CardContent
│   │   ├── dialog.rs               ← Dialog + DialogTrigger + DialogContent
│   │   ├── table.rs                ← Table + TableHeader + TableRow + TableCell
│   │   ├── tabs.rs                 ← Tabs + TabsList + TabsTrigger + TabsContent
│   │   ├── badge.rs                ← Badge с вариантами
│   │   ├── avatar.rs               ← Avatar + AvatarImage + AvatarFallback
│   │   ├── dropdown.rs             ← DropdownMenu + ...
│   │   ├── select.rs               ← Select с опциями
│   │   ├── checkbox.rs             ← Checkbox
│   │   ├── radio.rs                ← RadioGroup + RadioGroupItem
│   │   ├── switch.rs               ← Toggle Switch
│   │   ├── textarea.rs             ← Textarea
│   │   ├── label.rs                ← Label для форм
│   │   ├── separator.rs            ← Horizontal/Vertical separator
│   │   ├── skeleton.rs             ← Loading skeleton
│   │   ├── toast.rs                ← Toast notifications
│   │   ├── tooltip.rs              ← Tooltip
│   │   └── ...                     ← Добавляем по мере необходимости
│   │
│   ├── features/                    ← Feature-specific компоненты
│   │   ├── auth/
│   │   │   ├── login_form.rs
│   │   │   ├── register_form.rs
│   │   │   └── password_reset_form.rs
│   │   ├── dashboard/
│   │   │   ├── stats_grid.rs       ← Сетка статистики
│   │   │   ├── recent_activity.rs
│   │   │   └── quick_actions.rs
│   │   ├── users/
│   │   │   ├── user_table.rs
│   │   │   ├── user_filters.rs
│   │   │   └── user_form.rs
│   │   ├── content/
│   │   │   ├── page_editor.rs
│   │   │   └── media_uploader.rs
│   │   └── ...
│   │
│   ├── layouts/                     ← Layout компоненты
│   │   ├── admin_layout.rs         ← Основной layout (sidebar + header + content)
│   │   ├── auth_layout.rs          ← Layout для login/register
│   │   ├── sidebar.rs              ← Sidebar навигация
│   │   ├── header.rs               ← Top header с поиском/профилем
│   │   └── footer.rs               ← Footer
│   │
│   └── shared/                      ← Shared компоненты (не совсем UI, не совсем feature)
│       ├── protected_route.rs      ← Auth guard
│       ├── error_boundary.rs       ← Error handling
│       └── loading_spinner.rs      ← Global loader
│
├── pages/                           ← Page компоненты (используют всё выше)
│   ├── login.rs
│   ├── dashboard.rs
│   ├── users.rs
│   └── ...
│
├── providers/                       ← Context providers
│   ├── auth.rs
│   ├── locale.rs
│   └── theme.rs                    ← Dark mode (будущее)
│
└── ...
```

---

## 🔄 План миграции

### Фаза 1: Рефакторинг существующих компонентов (⬜ TODO)

**Задача:** Переделать `ui.rs` в структуру DSD

**Шаги:**

1. Создать `components/ui/button.rs` с вариантами
   - Варианты: `default`, `destructive`, `outline`, `secondary`, `ghost`, `link`
   - Размеры: `sm`, `md`, `lg`, `icon`
   - Props: `variant`, `size`, `disabled`, `class`

2. Создать `components/ui/input.rs` с вариантами
   - Types: `text`, `email`, `password`, `number`, `search`
   - Варианты: `default`, `error`
   - Props: `type_`, `placeholder`, `value`, `on_input`, `disabled`, `error`

3. Создать `components/ui/card.rs` (композиция)
   - Компоненты: `Card`, `CardHeader`, `CardTitle`, `CardDescription`, `CardContent`, `CardFooter`

4. Создать `components/ui/label.rs`
   - Для форм (связать с Input через `for` attribute)

5. Переместить `page_header.rs` → `components/features/dashboard/page_header.rs`

6. Переместить `stats_card.rs` → `components/features/dashboard/stats_card.rs`
   - Переделать на использование `Card` из `ui/card.rs`

**Результат:**
```
components/
├── ui/
│   ├── mod.rs
│   ├── button.rs        ← NEW (с вариантами)
│   ├── input.rs         ← NEW (с вариантами)
│   ├── card.rs          ← NEW (композиция)
│   └── label.rs         ← NEW
├── features/
│   └── dashboard/
│       ├── page_header.rs   ← MOVED from ui/
│       └── stats_card.rs    ← MOVED from ui/ + refactored
└── shared/
    └── protected_route.rs   ← MOVED from root
```

---

### Фаза 2: Добавить новые UI компоненты (⬜ TODO)

**Приоритет 1 (нужны для Login/Register):**
- `components/ui/alert.rs` — для ошибок валидации
- `components/ui/separator.rs` — визуальное разделение
- `components/ui/badge.rs` — для статусов

**Приоритет 2 (нужны для Dashboard):**
- `components/ui/table.rs` — таблицы пользователей
- `components/ui/skeleton.rs` — loading states
- `components/ui/dropdown.rs` — меню действий
- `components/ui/avatar.rs` — профиль пользователя

**Приоритет 3 (нужны для Users page):**
- `components/ui/dialog.rs` — модальные окна
- `components/ui/tabs.rs` — табы (Profile/Security/Settings)
- `components/ui/checkbox.rs` — мультивыбор
- `components/ui/select.rs` — дропдауны

---

### Фаза 3: Портировать shadcn/ui компоненты (⬜ TODO)

**Источник:** https://ui.shadcn.com/

**Компоненты для портирования:**

1. **Form Components**
   - `Form` (с react-hook-form аналогом — `leptos-hook-form` у вас есть!)
   - `Textarea`
   - `Switch`
   - `RadioGroup`
   - `Combobox`

2. **Data Display**
   - `Data Table` (с сортировкой/фильтрацией)
   - `Calendar`
   - `Chart` (у вас `leptos-chartistry` есть!)

3. **Feedback**
   - `Toast` (уведомления)
   - `Alert Dialog`
   - `Progress`
   - `Spinner`

4. **Navigation**
   - `Breadcrumb`
   - `Pagination` (у вас уже есть `leptos-shadcn-pagination`!)
   - `Command` (⌘K меню)

---

## 🛠️ Создание компонента (workflow)

### shadcn подход (Copy-paste-customize)

1. **Найти компонент на shadcn.com**
   - https://ui.shadcn.com/docs/components/button

2. **Скопировать код**
   ```tsx
   // React (shadcn)
   export const Button = ({ variant, size, children }) => {
     const baseClass = "inline-flex items-center...";
     const variantClass = variants[variant];
     return <button className={cn(baseClass, variantClass)}>{children}</button>;
   }
   ```

3. **Портировать в Leptos**
   ```rust
   #[component]
   pub fn Button(
       children: Children,
       #[prop(default = ButtonVariant::Default)] variant: ButtonVariant,
   ) -> impl IntoView {
       let base = "inline-flex items-center...";
       let variant_class = match variant { ... };
       view! { <button class={format!("{} {}", base, variant_class)}>{children()}</button> }
   }
   ```

4. **Добавить в `components/ui/mod.rs`**
   ```rust
   pub mod button;
   pub use button::*;
   ```

5. **Использовать в коде**
   ```rust
   use crate::components::ui::{Button, ButtonVariant};
   
   view! {
       <Button variant=ButtonVariant::Destructive>
           "Delete"
       </Button>
   }
   ```

---

## 📖 Документация компонентов

### Использовать mdBook

Создать `docs/components/` с примерами:

```markdown
# Button

A button component with multiple variants.

## Usage

\`\`\`rust
use crate::components::ui::{Button, ButtonVariant};

view! {
    <Button variant=ButtonVariant::Default>
        "Click me"
    </Button>
}
\`\`\`

## Variants

- `Default` — Primary action
- `Destructive` — Dangerous action (delete, etc.)
- `Outline` — Secondary action
- `Ghost` — Tertiary action
- `Link` — Text link style

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `variant` | `ButtonVariant` | `Default` | Visual style |
| `size` | `ButtonSize` | `Md` | Button size |
| `disabled` | `bool` | `false` | Disabled state |
| `class` | `String` | `""` | Additional classes |

## Examples

### Destructive Button
\`\`\`rust
<Button variant=ButtonVariant::Destructive>
    "Delete User"
</Button>
\`\`\`

### Small Outline Button
\`\`\`rust
<Button variant=ButtonVariant::Outline size=ButtonSize::Sm>
    "Cancel"
</Button>
\`\`\`
```

---

## 🎨 Tailwind конфигурация

Для DSD нужно настроить design tokens:

```js
// tailwind.config.js
module.exports = {
  theme: {
    extend: {
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        // ...
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
    },
  },
}
```

```css
/* input.css */
@layer base {
  :root {
    --background: 0 0% 100%;
    --foreground: 222.2 47.4% 11.2%;
    --primary: 222.2 47.4% 11.2%;
    --primary-foreground: 210 40% 98%;
    --destructive: 0 84.2% 60.2%;
    --destructive-foreground: 210 40% 98%;
    --radius: 0.5rem;
  }
  
  .dark {
    --background: 224 71% 4%;
    --foreground: 213 31% 91%;
    /* ... */
  }
}
```

---

## ✅ Вывод

### Для RusToK рекомендую **DSD (shadcn подход)**:

**Структура:**
```
components/
├── ui/              ← Все UI компоненты (flat, shadcn-style)
├── features/        ← Feature-specific компоненты
├── layouts/         ← Layout компоненты
└── shared/          ← Shared utilities
```

**Причины:**
1. ✅ Совместимость с `leptos-shadcn-pagination`
2. ✅ Быстрая разработка (меньше раздумий)
3. ✅ Легко портировать shadcn/ui компоненты
4. ✅ Flat структура (легко найти)
5. ✅ Tailwind-friendly
6. ✅ Copy-paste friendly

**Против Atomic Design:**
- ❌ Избыточно для проекта размера RusToK
- ❌ Больше споров "куда положить компонент"
- ❌ Не совместимо с shadcn экосистемой

---

## 🚀 Next Steps

1. ⬜ Создать `components/ui/button.rs` с вариантами (Phase 1)
2. ⬜ Создать `components/ui/input.rs` с вариантами
3. ⬜ Создать `components/ui/card.rs` (композиция)
4. ⬜ Рефакторить существующие компоненты
5. ⬜ Портировать shadcn компоненты (Phase 2-3)

**Готов начать имплементацию?** 🎯
