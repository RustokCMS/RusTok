# Документация `rustok-tax`

`rustok-tax` — foundation crate для tax bounded context в commerce family.

## Назначение

- typed contract для tax calculation;
- provider seam для будущих внешних tax engines;
- default provider `region_default`, который пока сохраняет текущую семантику
  `region.tax_rate` / `tax_included`;
- текущий selection hook через `regions.tax_provider_id`, чтобы provider
  choice уже был частью runtime contract до внешних tax integrations;
- единый source of truth для `provider_id` в tax-line snapshot.

## Зона ответственности

- модуль не владеет cart/order transport;
- модуль не владеет region identity, а потребляет policy snapshot;
- внешние tax providers должны подключаться поверх этого seam, а не напрямую в
  `rustok-cart` или `rustok-commerce`.

## Интеграция

- `rustok-cart` вызывает `TaxService` для пересчёта cart tax lines;
- checkout переносит provider-aware tax snapshot в `rustok-order`;
- transport surface пока публикуется через `rustok-commerce`.

## Проверка

- targeted unit tests в `rustok-tax`;
- compile-check для `rustok-tax`, `rustok-cart`, `rustok-order`, `rustok-commerce`.
