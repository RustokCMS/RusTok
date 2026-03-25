# rustok-customer

## Purpose

`rustok-customer` is the default storefront customer submodule of the `Ecommerce` family.

## Responsibilities

- Own the storefront customer profile schema and service logic.
- Keep customer identity separate from admin/runtime users while allowing optional linkage by `user_id`.
- Prepare a stable customer boundary for later checkout and payment flows.

## Interactions

- Depends on `rustok-core` for module contracts and customer permission vocabulary.
- Used by `rustok-commerce` as the default customer submodule of the ecommerce family.
- Keeps an optional `user_id` link to the platform user record without collapsing customer and user into one domain model.

## Entry points

- `CustomerModule`
- `CustomerService`
- `dto::*`
- `entities::*`

See also `docs/README.md`.
