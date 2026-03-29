# rustok-pages module implementation plan (`rustok-pages`)

## Scope and objective

This document tracks the module-local implementation plan for `rustok-pages`.

Primary objective: keep evolving `rustok-pages` on top of module-owned storage
without regressing the existing GraphQL, REST, admin, and storefront
contracts.

## Current state

- `pages`, `page_translations`, `page_bodies`, and `page_blocks` now have
  module-owned migrations.
- `PageService` no longer uses `rustok-content::NodeService` for page
  read/write paths.
- `BlockService` no longer uses shared node storage for page-block CRUD.
- `MenuService` now uses module-owned `menus`, `menu_translations`,
  `menu_items`, and `menu_item_translations`.
- Channel visibility is still metadata-based for v0 and can move to typed
  relations later if the product model stabilizes.

## Delivery phases

### Phase 0 - Foundation (done)

- [x] Baseline crate/module structure is in place.
- [x] Base docs and registry presence are established.
- [x] Core compile-time integration with the workspace is available.

### Phase 1 - Contract hardening (done)

- [x] Freeze public API expectations for the current module surface.
- [x] Align error and validation conventions with platform guidance.
- [x] Expand automated tests around core invariants and boundary behavior.

### Phase 2 - Storage split (in progress)

- [x] Implement the first module-owned persistence slice for pages and blocks.
- [x] Keep GraphQL and REST page CRUD on top of page-owned storage.
- [x] Keep block CRUD and reorder flows on top of page-owned storage.
- [x] Move menu storage and menu transport off shared `NodeService`.
- [ ] Replace metadata-based visibility with typed relations if the product
  model still requires it after the broader split.

### Phase 3 - Page builder hardening (planned)

- [ ] Promote `body.format = "grapesjs_v1"` as the canonical visual
  page-builder contract while keeping legacy block APIs migration-compatible.
- [ ] Add explicit round-trip tests for GrapesJS `projectData`.
- [ ] Lock down compatibility rules for legacy pages that still depend on block
  payloads.

### Phase 4 - Productionization (planned)

- [x] Finalize rollout and migration strategy for the menu slice.
- [ ] Complete security, tenancy, and RBAC checks relevant to the module.
- [ ] Validate observability, runbooks, and operational readiness.

## Notes

- `rustok-pages` now owns page, block, and menu persistence.
- The current page body contract still supports `markdown`, `rt_json_v1`, and
  `grapesjs_v1`.
- Block endpoints remain a supported migration surface for now; they are not the
  long-term canonical page-builder model.

## Test plan

- Unit:
  - validate accepted and rejected block payload shapes;
  - validate page visibility and metadata normalization;
  - validate page body round-trip for supported formats.
- Integration:
  - create, update, get, publish, and delete a page through module-owned
    storage;
  - create, update, reorder, and delete blocks through module-owned storage;
  - keep GraphQL and REST behavior aligned with the services.
- Integration:
  - create and get nested menus through module-owned storage.

## Definition of done

- page, block, and menu storage are fully owned by `rustok-pages`;
- no page read/write path depends on shared `NodeService`;
- public/admin contracts stay covered by tests;
- page-builder status is explicit and documented;
- local docs and the central split plan reflect the final state.

## Tracking and updates

When updating `rustok-pages` architecture, API contracts, tenancy behavior,
routing, or observability expectations:

1. Update this file first.
2. Update `crates/rustok-pages/README.md` and `crates/rustok-pages/docs/README.md`
   when public behavior changes.
3. Update `docs/index.md` links if documentation structure changes.
4. If module responsibilities change, update `docs/modules/registry.md`
   accordingly.

## Checklist

- [x] Contract tests cover the public pages use-cases that are already shipped.
- [x] `pages` remains the first `rustok-channel` pilot on the public read path.
- [x] `pages` keeps the first publication-level pilot through metadata-based
  `channelSlugs` allowlists.
- [x] Pages and blocks already run on module-owned storage.
- [x] Menus now run on module-owned storage.
