# Документация `rustok-order`

`rustok-order` — дефолтный order-подмодуль семейства `ecommerce`.

## Что сейчас внутри

- схема `orders` и `order_line_items`;
- `OrderModule` и `OrderService`;
- write-side lifecycle заказа: `pending -> confirmed -> paid -> shipped -> delivered/cancelled`;
- публикация order events через transactional outbox.

## Архитектурная граница

- модуль не зависит от `rustok-commerce` umbrella, чтобы не создавать цикл;
- product/variant ссылки в заказе хранятся как snapshot references, а не как обязательные cross-module foreign keys;
- GraphQL и REST transport пока остаются в фасаде `rustok-commerce`.

## Контракты событий

- [Event flow contract (central)](../../../docs/architecture/event-flow-contract.md)

## Связанные документы

- [README crate](../README.md)
- [План распила commerce](../../rustok-commerce/docs/implementation-plan.md)
