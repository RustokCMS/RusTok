# rustok-cart-storefront

Leptos storefront UI package for the `rustok-cart` module.

## Responsibilities

- Exposes the module-owned storefront cart route used by `apps/storefront`.
- Shows cart read-side state, line items, and delivery-group snapshots from the cart boundary.
- Supports safe cart-owned line-item decrement and remove actions without taking over checkout orchestration.
- Uses native Leptos `#[server]` calls as the default internal data layer and keeps GraphQL as fallback.
- Leaves checkout completion and broader cross-domain orchestration inside `rustok-commerce`.

## Entry Points

- `CartView` - root storefront view rendered from the host storefront slot registry.
- `api::fetch_storefront_cart` - native-first cart read transport with GraphQL fallback.
- `api::decrement_storefront_cart_line_item` - safe line-item decrement path with native-first transport.
- `api::remove_storefront_cart_line_item` - safe line-item removal path with native-first transport.

## Interactions

- Consumed by `apps/storefront` via manifest-driven `build.rs` code generation.
- Reads `CartService` through server functions and enforces customer-owned cart access with the host auth context.
- Stays compatible with locale-prefixed module routes via `UiRouteContext::module_route_base()`.
- Coexists with the `rustok-commerce` storefront/transport layer while checkout and shipping orchestration remain aggregate concerns.

## Documentation

- See [platform docs](../../../docs/index.md).
