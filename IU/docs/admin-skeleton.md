# Admin Architecture Skeleton (draft)

This outlines the initial layout and module structure for a classic admin UI with a
left sidebar and top header.

## Layout Regions
- `Sidebar`: primary navigation, collapsible.
- `Header`: global actions, user menu, breadcrumbs.
- `Content`: page body with optional toolbar.
- `Footer`: optional.

## Page Template
- `PageHeader`: title, actions, breadcrumbs.
- `PageBody`: content area with sections and cards.
- `PageFooter`: optional pagination or summary.

## Module Structure (recommended)
- `modules/<domain>/pages/*`
- `modules/<domain>/components/*`
- `modules/<domain>/api/*`
- `modules/<domain>/types/*`

## Navigation
- Sidebar items map to `modules/*/pages` routes.
- Access control hooks can be placed at route boundaries.

This is an alpha version and requires clarification. Be careful, there may be errors in the text. So that no one thinks that this is an immutable rule.
