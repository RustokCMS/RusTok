# Leptos Table

Internal, minimal helpers for table state (pagination/sorting/filtering) across
admin and storefront frontends (Leptos + Next.js).

Target apps:
- `apps/admin` (Leptos CSR)
- `apps/next-admin` (Next.js)
- `apps/storefront` (Leptos SSR)
- `apps/next-frontend` (Next.js)

---

## Why this exists

- Next.js relies on `@tanstack/react-table` for complex table behavior.
- Leptos has no 1:1 equivalent; we need a tiny shared contract to keep parity.
- We standardize pagination, sorting, and filtering shapes for API calls.

---

## Contract

- `TableState` (`page`, `page_size`, `sort`, `filters`)
- `SortRule` (`field`, `direction`)
- `FilterRule` (`field`, `value`)

---

## Runtime implementations

### 1) Leptos (Rust)
Rust helpers live in `crates/leptos-table`.

Exports:
- `TableState`, `SortRule`, `FilterRule`, `SortDirection`

### 2) Next.js (TypeScript)
TypeScript helpers live in `packages/leptos-table/next`.

Exports:
- `TableState`, `SortRule`, `FilterRule`, `SortDirection`

---

## Usage (examples)

### Leptos
```rust
use leptos_table::{TableState, SortDirection, SortRule};

let state = TableState::new(1, 25).with_sort(vec![SortRule {
    field: "name".to_string(),
    direction: SortDirection::Asc,
}]);
```

### Next.js
```ts
import { TableState, SortDirection } from "@/lib/leptos-table";

const state = TableState.withSort({
  page: 1,
  pageSize: 25,
  sort: [{ field: "name", direction: SortDirection.Asc }],
  filters: [],
});
```

---

## Status

**Minimal baseline** for shared table state. Extend only when needed.

This is an alpha version and requires clarification. Be careful, there may be errors in the text. So that no one thinks that this is an immutable rule.
