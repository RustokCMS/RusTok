# План реализации `rustok-cart`

Статус: cart boundary выделен; модуль остаётся owner-ом cart state и storefront
context snapshot, а orchestration над checkout живёт в umbrella `rustok-commerce`.

## Область работ

- удерживать `rustok-cart` как owner cart lifecycle и line-item state;
- синхронизировать cart snapshot contract, runtime dependencies, storefront UI ownership и local docs;
- не допускать возврата cart domain logic обратно в umbrella или host слой.

## Текущее состояние

- `carts` и `cart_line_items` уже module-owned;
- `cart_adjustments` уже module-owned и фиксируют language-neutral promotion/discount snapshot без display labels;
- cart lifecycle и persisted storefront context snapshot уже встроены в базовый contract;
- transport adapters по-прежнему публикуются фасадом `rustok-commerce`, без цикла зависимостей;
- storefront cart inspection, safe decrement/remove write-side и seller-aware delivery-group snapshot уже вынесены в `rustok-cart/storefront`;
- channel/context/deliverability orchestration поверх cart по-прежнему выполняется на уровне umbrella-модуля.

## Этапы

### 1. Contract stability

- [x] зафиксировать cart lifecycle и storefront context snapshot;
- [x] удерживать line-item CRUD и totals внутри `rustok-cart`;
- [x] добавить typed cart adjustment snapshot с `subtotal_amount`, `adjustment_total` и net `total_amount`;
- [x] удерживать sync между cart runtime contract, commerce orchestration, storefront route ownership и module metadata.

### 2. Storefront ownership

- [x] вынести storefront cart inspection в `rustok-cart/storefront`;
- [x] использовать native Leptos `#[server]` functions как default internal data layer;
- [x] сохранить GraphQL storefront contract как fallback;
- [x] вынести безопасные cart-owned line-item decrement/remove mutations из aggregate storefront surface;
- [ ] не смешивать cart-owned UI с quantity increase, add-to-cart и checkout orchestration, пока эти write-path требуют cross-domain validation.

### 3. Checkout hardening

- [ ] удерживать `checking_out`/recovery semantics совместимыми с payment/order orchestration;
- [x] покрывать stale snapshot, shipping selection и multi-group edge-cases targeted tests;
- [ ] развивать cart state только через explicit snapshot/versioning semantics.

### 4. Operability

- [ ] документировать новые cart guarantees одновременно с изменением checkout flows;
- [x] удерживать local docs и `README.md` синхронизированными со storefront contract;
- [ ] расширять diagnostics только при реальном runtime pressure.

## Проверка

- `cargo xtask module validate cart`
- `cargo xtask module test cart`
- targeted tests для cart lifecycle, line items, typed adjustments, snapshot context и checkout-preflight semantics

## Правила обновления

1. При изменении cart runtime contract сначала обновлять этот файл.
2. При изменении public/runtime surface синхронизировать `README.md`, `docs/README.md` и `storefront/README.md`.
3. При изменении module metadata синхронизировать `rustok-module.toml`.
4. При изменении checkout orchestration expectations обновлять umbrella docs в `rustok-commerce`.
