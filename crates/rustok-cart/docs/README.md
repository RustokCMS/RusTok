# Документация `rustok-cart`

`rustok-cart` — дефолтный cart-подмодуль семейства `ecommerce`.

## Назначение

- схема `carts`, `cart_line_items` и `cart_line_item_translations` (localized line-item titles вынесены из base rows);
- `CartModule` и `CartService`;
- persisted cart context snapshot: `region_id`, `country_code`, `locale_code`, `selected_shipping_option_id`,
  `customer_id`, `email`, `currency_code`;
- typed `cart_adjustments` для promotion/discount snapshot: `source_type/source_id`, `amount/currency_code`,
  optional line-item binding и language-neutral metadata без display label;
- lifecycle корзины: `active -> checking_out -> completed` и `active -> abandoned`;
- CRUD line items, расчёт totals, seller-aware delivery-group snapshot с canonical `seller_id` и нормализация locale/country snapshot для storefront-контекста;
- перепрайс line items при изменении количества или storefront context (region/channel), чтобы unit_price не дрейфовал;
- module-owned storefront пакет `rustok-cart/storefront` для cart inspection и безопасных line-item decrement/remove действий.

## Зона ответственности

- модуль не зависит от `rustok-commerce` umbrella, чтобы не создавать цикл;
- product/variant ссылки в корзине хранятся как snapshot references, а не как обязательные cross-module foreign keys;
- cart хранит snapshot storefront context, но не владеет region/locale policy: tenant locale enablement и
  cross-module orchestration остаются на уровне `rustok-commerce` umbrella;
- GraphQL и REST transport по-прежнему остаются в фасаде `rustok-commerce`;
- storefront cart UI теперь живёт внутри самого модуля и не возвращает cart ownership обратно в host или umbrella UI.

## Интеграция

- модуль входит в ecommerce family и должен сохранять собственную storage/runtime-границу без возврата ответственности в umbrella `rustok-commerce`;
- transport и GraphQL по-прежнему публикуются через `rustok-commerce`, но storefront cart read-side, seller-aware delivery-group snapshot и безопасный line-item write-side уже вынесены в отдельный module-owned surface `rustok-cart/storefront`;
- `seller_scope` в cart contract остаётся только transitional compatibility field для legacy snapshot'ов без `seller_id`; canonical grouping и shipping selection теперь опираются на `seller_id`.
- `cart_adjustments` являются source of truth для скидочного snapshot в cart: `subtotal_amount`, `adjustment_total`
  и net `total_amount` не зависят от default locale или localized promotion label.
- изменения cross-module контракта нужно синхронизировать с `rustok-commerce` и соседними split-модулями;
- storefront package использует native Leptos `#[server]` functions как default data layer и сохраняет GraphQL storefront contract как fallback.

## Проверка

- `cargo xtask module validate cart`
- `cargo xtask module test cart`
- targeted commerce tests для cart-домена при изменении runtime wiring

## Связанные документы

- [README crate](../README.md)
- [План развития `rustok-cart`](./implementation-plan.md)
- [План распила commerce](../../rustok-commerce/docs/implementation-plan.md)
