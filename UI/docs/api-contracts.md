# IU API Contracts (draft)

This document defines the minimal cross-framework API contract for the first wave of
components. The goal is parity across Next (React) and Leptos (Rust) while allowing
implementation differences.

## Conventions
- Variants and sizes must match across frameworks.
- Disabled and loading behaviors should be consistent.
- Class names may be framework-specific, but CSS variables and tokens must be shared.

## 1) Button
**Props**
- `variant`: `primary | secondary | ghost | outline | destructive`
- `size`: `sm | md | lg | icon`
- `disabled`: boolean
- `loading`: boolean
- `leftIcon` / `rightIcon`: optional

**Behavior**
- `loading` disables interaction and shows a spinner.
- `icon` size is square and icon-only.

---

## 2) Input
**Props**
- `size`: `sm | md | lg`
- `disabled`: boolean
- `invalid`: boolean
- `prefix` / `suffix`: optional

**Behavior**
- `invalid` sets error styles and aria attributes.

---

## 3) Textarea
**Props**
- `size`: `sm | md | lg`
- `disabled`: boolean
- `invalid`: boolean
- `rows`: number

---

## 4) Select
**Props**
- `size`: `sm | md | lg`
- `disabled`: boolean
- `invalid`: boolean
- `options`: array or slot-based items
- `placeholder`: string

---

## 5) Checkbox
**Props**
- `checked`: boolean
- `indeterminate`: boolean
- `disabled`: boolean

---

## 6) Switch
**Props**
- `checked`: boolean
- `disabled`: boolean
- `size`: `sm | md`

---

## 7) Badge/Tag
**Props**
- `variant`: `default | secondary | success | warning | danger`
- `size`: `sm | md`
- `dismissible`: boolean

