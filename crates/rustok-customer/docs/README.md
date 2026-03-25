# Документация `rustok-customer`

`rustok-customer` — дефолтный storefront-customer подмодуль семейства `ecommerce`.

## Что сейчас внутри

- схема `customers`;
- `CustomerModule` и `CustomerService`;
- customer profile boundary, отделённый от platform/admin user;
- optional linkage на `user_id` для сценариев `store/customers/me`.

## Архитектурная граница

- модуль не зависит от `rustok-commerce` umbrella, чтобы не создавать цикл;
- customer profile хранится отдельно от auth/user домена;
- связь с пользователем опциональна и не отменяет самостоятельность customer-модели;
- GraphQL и REST transport пока остаются в фасаде `rustok-commerce`.

## Связанные документы

- [README crate](../README.md)
- [План распила commerce](../../rustok-commerce/docs/implementation-plan.md)
