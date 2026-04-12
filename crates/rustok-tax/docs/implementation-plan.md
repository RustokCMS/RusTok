# План реализации `rustok-tax`

Статус: foundation phase.

## Цель

- вынести tax calculation из hardcoded cart runtime в отдельный bounded context;
- зафиксировать provider seam до реальных внешних интеграций;
- сделать `provider_id` частью tax snapshot contract.

## Текущее состояние

- default provider `region_default` сохраняет текущую region-based tax policy;
- `rustok-cart` вызывает `TaxService`, а не считает налог напрямую из `region`;
- current provider selection hook lives in `regions.tax_provider_id`;
- cart/order tax lines получают typed `provider_id`.

## Следующие шаги

- tax rules beyond flat region rate;
- provider registry и external engine adapters;
- richer jurisdiction metadata и transport parity tests.
