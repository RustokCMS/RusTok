# rustok-search

## Purpose

`rustok-search` owns product-facing search contracts for RusToK.

## Responsibilities

- Provide `SearchModule` metadata for the runtime registry.
- Define canonical search engine selection and connector metadata.
- Keep PostgreSQL as the default search engine contract.
- Own search settings persistence and future connector-facing runtime contracts.
- Own module-local `search_documents` storage and execute PostgreSQL FTS with `tsvector`,
  `websearch_to_tsquery`, `ts_rank_cd`, highlights, and facet aggregation.
- Serve as the home for storefront/admin/global-search capabilities, separate from `rustok-index`.

## Interactions

- Depends on `rustok-core` for module contracts.
- May ingest from domain tables or neutral read models, but keeps its own search storage and runtime.
- Is used by `apps/server` as a core platform capability.
- Publishes module-owned admin and storefront UI packages for host composition.
- Keeps external engine integrations behind dedicated connector crates, rather than forcing domain modules to talk to provider SDKs directly.

## Entry points

- `SearchModule`
- `SearchEngineKind`
- `SearchConnectorDescriptor`
- `SearchSettingsRecord`
- `PgSearchEngine`

## Current status

- Admin GraphQL exposes engine availability, effective settings preview, and an FTS-backed
  `searchPreview` surface with filters and facets.
- Storefront GraphQL exposes public `storefrontSearch`, limited to published content/products.
- Search queries are read-only and no longer trigger bootstrap rebuilds on the request path.
- PostgreSQL search reads from `search_documents`, not from `rustok-index` tables.
- `SearchIngestionHandler` updates `search_documents` asynchronously from domain events and supports rebuild requests.
- Admin GraphQL exposes `searchDiagnostics`, `searchLaggingDocuments`, and
  `triggerSearchRebuild` for lag/state inspection, raw stale-document diagnostics,
  and queued tenant-wide or scoped rebuilds.
- Search GraphQL now enforces tenant-local scope and validates engine/filter input before execution.
- Leptos admin and storefront packages are wired to the live GraphQL search contract.
- The Leptos admin package now ships separate overview, playground, diagnostics,
  and dictionaries surfaces under the module route, and is also exposed as a
  first-class admin sidebar entry.
- The Next admin package mirrors the same control-plane surfaces and uses the same
  GraphQL contract for diagnostics, rebuilds, and FTS preview.
