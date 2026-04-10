# Документация `rustok-order`

`rustok-order` — дефолтный order-подмодуль семейства `ecommerce`.

## Назначение

- схема `orders`, `order_line_items` и `order_adjustments`;
- `OrderModule` и `OrderService`;
- write-side lifecycle заказа: `pending -> confirmed -> paid -> shipped -> delivered/cancelled`;
- публикация order events через transactional outbox;
- module-owned admin UI пакет `rustok-order/admin` для order operations.

## Зона ответственности

- модуль не зависит от `rustok-commerce` umbrella, чтобы не создавать цикл;
- product/variant ссылки в заказе хранятся как snapshot references, а не как
  обязательные cross-module foreign keys;
- order line items теперь тоже несут nullable `seller_id` как canonical multivendor snapshot key;
- order adjustments хранят promotion/discount snapshot как typed business data: `source_type/source_id`,
  `amount/currency_code`, optional line-item binding и metadata без localized display label;
- GraphQL и REST transport пока остаются в фасаде `rustok-commerce`;
- admin UI ownership вынесен в `rustok-order/admin`.

## Контракты событий

- [Event flow contract (central)](../../../docs/architecture/event-flow-contract.md)

## Интеграция

- модуль входит в ecommerce family и должен сохранять собственную
  storage/runtime-границу без возврата ответственности в umbrella `rustok-commerce`;
- transport и GraphQL публикуются через `rustok-commerce`, а operator UX для
  order list/detail/lifecycle публикуется через `rustok-order/admin`;
- checkout/create-order snapshot передаёт typed adjustments в `rustok-order`, а `subtotal_amount`,
  `adjustment_total` и net `total_amount` остаются устойчивыми к смене default locale;
- изменения cross-module контракта нужно синхронизировать с `rustok-commerce`
  и соседними split-модулями.

## Проверка

- `cargo xtask module validate order`
- `cargo xtask module test order`
- targeted commerce tests для order-домена при изменении runtime wiring

## Связанные документы

- [README crate](../README.md)
- [README admin package](../admin/README.md)
- [План распила commerce](../../rustok-commerce/docs/implementation-plan.md)
