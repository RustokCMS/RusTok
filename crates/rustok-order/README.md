# rustok-order

## Purpose

`rustok-order` is the default order submodule of the `Ecommerce` family.

## Responsibilities

- Own the order write-side schema, service, and status transitions.
- Persist order snapshots and line items independently from catalog ownership.
- Publish transactional order lifecycle events through the outbox.

## Interactions

- Depends on `rustok-core` for module contracts and permission vocabulary.
- Depends on `rustok-events` and `rustok-outbox` for transactional domain-event publishing.
- Used by `rustok-commerce` as the default order submodule of the ecommerce family.
- Keeps product and variant references as snapshots so the order domain does not depend on
  the product module as a lower-level shared layer.

## Entry points

- `OrderModule`
- `OrderService`
- `dto::*`
- `entities::*`

See also `docs/README.md`.
