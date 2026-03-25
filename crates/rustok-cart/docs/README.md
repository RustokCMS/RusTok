# Документация `rustok-cart`

`rustok-cart` — дефолтный cart-подмодуль семейства `ecommerce`.

## Что сейчас внутри

- схема `carts` и `cart_line_items`;
- `CartModule` и `CartService`;
- lifecycle корзины: `active -> completed/abandoned`;
- CRUD line items и расчёт cart totals.

## Архитектурная граница

- модуль не зависит от `rustok-commerce` umbrella, чтобы не создавать цикл;
- product/variant ссылки в корзине хранятся как snapshot references, а не как обязательные cross-module foreign keys;
- GraphQL и REST transport пока остаются в фасаде `rustok-commerce`.

## Связанные документы

- [README crate](../README.md)
- [План распила commerce](../../rustok-commerce/docs/implementation-plan.md)
