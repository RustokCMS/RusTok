# rustok-search implementation plan

## Purpose

This document is the canonical implementation roadmap for `rustok-search`.

It replaces the older central platform draft that previously lived in
`docs/modules/search-module-plan.md`. Search planning now lives with the actual
module code so architecture, runtime behavior, and rollout phases stay aligned.

## Objective

Build `rustok-search` as a dedicated core search module that:

- keeps search separate from `rustok-index`
- uses PostgreSQL as the default engine
- owns storefront and admin search surfaces
- evolves Leptos and Next UI in parallel on the same API contract
- supports optional external search connectors

## Architecture boundary

### `rustok-index`

`rustok-index` is responsible for:

- centralized index and read-model infrastructure
- ingestion and denormalized read models
- cross-module joins and filtering substrate
- rebuild, sync, and consistency workflows for index data

`rustok-index` is not responsible for:

- full-text search UX
- search ranking and relevance
- synonyms and stop words
- autocomplete and suggestions
- search analytics
- engine-specific search connectors

### `rustok-search`

`rustok-search` is responsible for:

- canonical search API and runtime contracts
- search document materialization
- query parsing and normalization
- ranking and relevance
- admin and storefront search UI
- global admin search
- search analytics and observability
- connector integration for external engines

## Runtime model

Preferred interaction model:

1. Domain modules emit changes.
2. `rustok-search` builds and updates its own `search_documents`.
3. Search UI and API talk only to `rustok-search`.
4. `rustok-search` may read from domain data directly and may optionally use
   neutral read models where that improves ingestion, but it does not depend on
   `rustok-index` as its canonical search storage.

Practical conclusion:

- `index` and `search` can cooperate, but they must not collapse into one module.
- `search -> index` can be allowed where useful.
- `index -> search` should remain forbidden.

## Engine strategy

### Default engine

PostgreSQL is the default and baseline engine:

- full-text search
- weighted `tsvector`
- `ts_rank` / `ts_rank_cd`
- `websearch_to_tsquery`
- `ts_headline`
- GIN indexes
- `pg_trgm`
- `unaccent`

### Optional engines

External engines should be delivered as dedicated optional connector crates:

- `rustok-search-meilisearch`
- `rustok-search-typesense`
- `rustok-search-algolia`

Recommended connector model:

- one connector per engine
- official SDK for each provider
- thin shared contract inside `rustok-search`
- engine selection through `rustok-search` settings

Why not a Laravel Scout style foundation:

- provider capabilities differ too much
- a single abstraction becomes either too weak or leaks provider behavior
- crate-based connectors fit the RusToK modular architecture better

## Engine selection UX

Expected operator experience:

1. A connector crate is added to the platform.
2. The connector registers capabilities and settings schema.
3. Search settings expose only actually installed engines.
4. `postgres` remains the default and fallback option.
5. Domain modules stay unaware of which engine is currently active.

## Current implementation checkpoint

Already implemented:

- dedicated `rustok-search` core module
- module-local `search_documents` storage
- PostgreSQL FTS runtime over `search_documents`
- canonical search contracts and settings persistence
- tenant-scoped GraphQL access and stricter search input validation
- async ingestion from content and product events
- queued rebuild flow for `search`, `content`, `product`, and optional target IDs
- admin diagnostics via `searchDiagnostics`
- raw stale-document diagnostics via `searchLaggingDocuments`
- admin FTS preview via `searchPreview`
- public storefront search via `storefrontSearch`
- read-only search query path without bootstrap rebuild side effects
- Leptos admin control plane with overview, playground, diagnostics, dictionaries
- Next admin control plane mirroring the same contract
- first-class sidebar entry in both Leptos admin and Next admin

Not implemented yet:

- global admin search UX
- editor UI for engine settings
- stop words and synonym dictionaries
- analytics dashboards and query intelligence
- external connector crates

## Delivery phases

### Phase S0 - Architecture fixation

- [ ] Record ADR: `index != search`
- [x] Keep `rustok-search` as a dedicated core module
- [x] Keep dependency direction explicit
- [x] Define crate layout for core/admin/storefront packages
- [x] Keep engine selection centered in `rustok-search`

### Phase S1 - Core contracts

- [x] Define `SearchQuery`
- [x] Define `SearchResult`
- [x] Define `SearchEngine`
- [x] Define settings-driven engine selection
- [ ] Expand capability matrix and contract tests
- [ ] Finalize search-facing error catalog and validation policy

### Phase S2 - PostgreSQL MVP

- [x] Implement PostgreSQL as default engine
- [x] Use PostgreSQL FTS primitives first
- [x] Add filters and facets for `entity_type`, `source_module`, `status`
- [x] Add highlights/snippets
- [ ] Finish ranking profile hardening
- [ ] Add broader sorting/profile controls

### Phase S3 - Ingestion pipeline

- [x] Materialize `search_documents`
- [x] Add event-driven upsert/delete
- [x] Add queued rebuild flows
- [ ] Add retry / DLQ strategy
- [ ] Add consistency checks between source state and search state
- [ ] Add richer rebuild scopes such as locale-wide reindexing where needed

### Phase S4 - API surfaces

- [x] Add GraphQL search queries
- [x] Add admin/debug queries
- [x] Add rebuild mutation
- [ ] Add settings mutation/editor flow beyond preview
- [ ] Decide whether any REST diagnostics endpoints are still needed

### Phase S5 - Storefront UI

- [x] Ship Leptos storefront package
- [x] Ship Next storefront package scaffold
- [x] Keep storefront search on shared backend contract
- [ ] Add suggestions/autocomplete
- [ ] Add richer URL/state model and SSR polish

### Phase S6 - Admin UI

- [x] Ship Leptos admin package
- [x] Ship Next admin package
- [x] Add overview, playground, diagnostics, dictionaries surfaces
- [x] Add scoped rebuild UI
- [x] Add raw lagging document diagnostics
- [ ] Add engine/settings editor
- [ ] Build global admin search experience

### Phase S7 - Standard search features

- [ ] Stop words
- [ ] Synonyms
- [ ] Query normalization policy
- [ ] Typo tolerance
- [ ] Suggestions/autocomplete
- [ ] Filter presets
- [ ] Ranking profiles
- [ ] Boost/bury rules
- [ ] Query rules and redirects
- [ ] Audit trail for search settings changes

### Phase S8 - Analytics and observability

- [ ] Query volume metrics
- [ ] Search latency metrics
- [ ] Zero-result rate
- [ ] Slow-query rate
- [ ] Indexing lag metrics
- [ ] Failed indexing rate
- [ ] Top queries and zero-result query analysis
- [ ] CTR and abandonment analysis
- [ ] Admin dashboards and operator runbooks

### Phase S9 - External engines

- [ ] Add `rustok-search-meilisearch`
- [ ] Add `rustok-search-typesense`
- [ ] Add `rustok-search-algolia`
- [ ] Add connector health and schema sync
- [ ] Add degraded-mode and fallback behavior

### Phase S10 - Advanced features

- [ ] Merchandising and pinned results
- [ ] Hidden results
- [ ] "Did you mean"
- [ ] Curated landing/query rules
- [ ] Per-surface ranking profiles
- [ ] A/B testing for relevance strategies
- [ ] Semantic or hybrid search as a late additive phase

## Search document baseline

Minimum search document shape should support:

- `id`
- `tenant_id`
- `locale`
- `entity_type`
- `source_module`
- `title`
- `subtitle`
- `body`
- `keywords`
- `facets`
- `visibility` / `status`
- `admin_url` / public URL
- `payload`
- `updated_at`

## Definition of done for first production-ready release

- [x] `rustok-search` exists as a dedicated core module
- [x] PostgreSQL works as the default engine
- [x] Search GraphQL API exists
- [x] Leptos and Next admin surfaces exist
- [x] Rebuild and diagnostics exist
- [x] Baseline filters/facets exist
- [ ] Storefront UX is fully polished on both hosts
- [ ] Global admin search is implemented
- [ ] Settings editor is complete
- [ ] Telemetry and analytics are production-ready

## Open questions

- Should any search ingestion paths still read from `rustok-index`, or should
  the module remain domain-event/domain-table first?
- What is the exact MVP scope for global admin search?
- Should synonyms and stop words be platform-level first, then tenant-level?
- When external engines are enabled, what degraded-mode UX should operators see?

## Documentation rules for this module

When architecture, API contracts, routing, tenancy behavior, observability, or
admin/storefront UI responsibilities change:

1. Update this file first.
2. Update `crates/rustok-search/README.md`.
3. Update `crates/rustok-search/docs/README.md` if the documentation map changes.
4. Update `docs/index.md` if central documentation links need to move.
