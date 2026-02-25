# IU API Contracts

This document defines the cross-framework API contract for UI components.
The goal is visual and behavioral parity across Next.js (React/shadcn) and Leptos (Rust).

## Theming contract

Both frameworks share the same CSS variable set (shadcn-compatible format).
The variables are defined in each app's CSS entry point and resolved by Tailwind:

```css
:root {
  --background: 0 0% 100%;
  --foreground: 222.2 84% 4.9%;
  --card: 0 0% 100%;
  --card-foreground: 222.2 84% 4.9%;
  --popover: 0 0% 100%;
  --popover-foreground: 222.2 84% 4.9%;
  --primary: 222.2 47.4% 11.2%;
  --primary-foreground: 210 40% 98%;
  --secondary: 210 40% 96.1%;
  --secondary-foreground: 222.2 47.4% 11.2%;
  --muted: 210 40% 96.1%;
  --muted-foreground: 215.4 16.3% 46.9%;
  --accent: 210 40% 96.1%;
  --accent-foreground: 222.2 47.4% 11.2%;
  --destructive: 0 84.2% 60.2%;
  --destructive-foreground: 210 40% 98%;
  --border: 214.3 31.8% 91.4%;
  --input: 214.3 31.8% 91.4%;
  --ring: 222.2 84% 4.9%;
  --radius: 0.5rem;
}
```

Dark mode uses `.dark` class strategy on the root element.

## Implementation conventions

- Variants and sizes must match across frameworks.
- Disabled and loading behaviors must be consistent.
- Class names are framework-specific, but CSS variables are shared.
- Leptos components port class strings directly from shadcn/ui source.
- No hardcoded color values — only semantic CSS variable utilities.

---

## 1) Button

**Variants**
- `default` — `bg-primary text-primary-foreground shadow hover:bg-primary/90`
- `destructive` — `bg-destructive text-destructive-foreground shadow-sm hover:bg-destructive/90`
- `outline` — `border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground`
- `secondary` — `bg-secondary text-secondary-foreground shadow-sm hover:bg-secondary/80`
- `ghost` — `hover:bg-accent hover:text-accent-foreground`
- `link` — `text-primary underline-offset-4 hover:underline`

**Props**
- `variant`: `default | destructive | outline | secondary | ghost | link`
- `size`: `sm | md | lg | icon`
- `disabled`: boolean
- `loading`: boolean — disables interaction and shows a spinner
- `type`: `button | submit | reset`

**Base classes** (same in both frameworks):
```
inline-flex items-center justify-center whitespace-nowrap rounded-md font-medium
ring-offset-background transition-colors
focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2
disabled:pointer-events-none disabled:opacity-50
```

---

## 2) Input

**Props**
- `type`: `text | password | email | number | ...`
- `size`: `sm | md | lg`
- `disabled`: boolean
- `invalid`: boolean — sets `border-destructive focus-visible:ring-destructive` and `aria-invalid`
- `placeholder`: string

**Base classes**:
```
flex w-full rounded-md border bg-background text-foreground shadow-sm transition-colors
placeholder:text-muted-foreground
focus-visible:outline-none focus-visible:ring-1
disabled:cursor-not-allowed disabled:opacity-50
```

---

## 3) Textarea

**Props**
- `size`: `sm | md | lg`
- `disabled`: boolean
- `invalid`: boolean
- `rows`: number (default 3)
- `placeholder`: string

Same styling pattern as Input.

---

## 4) Select

**Props**
- `size`: `sm | md | lg`
- `disabled`: boolean
- `invalid`: boolean
- `options`: array of `{ value, label, disabled? }`
- `placeholder`: string

Uses native `<select>` element styled with shadcn border/bg classes.

---

## 5) Checkbox

**Props**
- `checked`: reactive boolean signal (Leptos) / boolean (React)
- `indeterminate`: boolean
- `disabled`: boolean

**Classes**: `h-4 w-4 rounded border border-primary text-primary`

---

## 6) Switch

**Props**
- `checked`: reactive boolean signal (Leptos) / boolean (React)
- `disabled`: boolean
- `size`: `sm | md`

Track: `bg-primary` (checked) / `bg-input` (unchecked).
Thumb: `bg-background` rounded circle.

---

## 7) Badge

**Variants**
- `default` — `bg-primary text-primary-foreground`
- `secondary` — `bg-secondary text-secondary-foreground`
- `destructive` — `bg-destructive text-destructive-foreground`
- `outline` — `text-foreground` with border
- `success` — `bg-emerald-100 text-emerald-700` (dark: `bg-emerald-900/30 text-emerald-400`)
- `warning` — `bg-amber-100 text-amber-700` (dark: `bg-amber-900/30 text-amber-400`)

**Props**
- `variant`: `default | secondary | destructive | outline | success | warning`
- `size`: `sm | md`
- `dismissible`: boolean

---

## 8) Spinner

Custom component (shadcn/ui has no Spinner).

**Props**
- `size`: `sm | md | lg`

**Implementation**: `border-primary border-t-transparent animate-spin rounded-full`
