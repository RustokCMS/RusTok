# План реализации `rustok-pricing`

Статус: pricing boundary выделен как отдельный модуль; модуль держит pricing runtime
baseline, module-owned admin UI уже включает base-price write path, а promotion/rule-aware
transport и полный `pricing 2.0` остаются в активном backlog umbrella `rustok-commerce`.

## Область работ

- удерживать `rustok-pricing` как owner pricing service boundary;
- синхронизировать pricing runtime contract, module-owned admin UI и local docs;
- не смешивать pricing storage с product catalog, promotions или tax orchestration.

## Текущее состояние

- `PricingModule`, `PricingService` и pricing migrations уже выделены;
- модуль зависит от `product`, не создавая цикла с umbrella `rustok-commerce`;
- transport adapters по-прежнему публикуются фасадом `rustok-commerce`;
- `rustok-pricing/admin` уже публикует pricing-owned admin route для price visibility,
  sale markers, currency coverage inspection, operator-side effective price context,
  selector активных price lists и write actions по base rows или active price-list
  overlays для variant prices, включая quantity tiers и typed percentage-discount
  preview/apply по canonical base row или выбранному active `price_list` override; туда
  же теперь вынесен selected active `price_list` rule editor;
- `rustok-pricing/storefront` уже публикует pricing-owned storefront route для public
  pricing atlas, currency coverage, sale-marker visibility и selector активных
  price lists поверх existing effective context;
- storefront package по-прежнему остаётся read-side surface, но admin package уже
  использует native-first `#[server]` transport не только для read-side, но и для
  base-price write actions, оставляя product GraphQL контракт как fallback для чтения.

## Этапы

### 1. Contract stability

- [x] закрепить pricing boundary как отдельный модуль;
- [x] удерживать зависимость `pricing -> product` без цикла на umbrella;
- [x] вынести pricing admin UI в module-owned пакет `rustok-pricing/admin`;
- [x] вынести pricing storefront UI в module-owned пакет `rustok-pricing/storefront`;
- [ ] удерживать sync между pricing runtime contract, admin UI, commerce transport
  и module metadata.

### 2. Pricing transport split

- [~] вынести dedicated pricing read/write transport из umbrella `rustok-commerce`;
- [x] перевести pricing admin UI с read-only product-backed transport на targeted
  base-price mutations и operator workflows;
- [~] покрывать transport parity, money semantics и compare-at invariants targeted tests.

### 3. Pricing 2.0 rollout

- [~] перейти от базовых цен к rule-driven price resolution;
- [x] ввести typed resolver foundation по `currency_code + optional region_id + optional quantity`
  с deterministic precedence для base prices;
- [x] активировать explicit `price_list_id` overlay в resolver для active tenant-scoped
  price lists с base-price fallback;
- [x] добавить channel-aware foundation в resolver/read-side contract через
  host-provided `channel_id` / `channel_slug`, channel-scoped base rows и
  channel-filtered active price lists без ownership drift в `rustok-channel`;
- [x] протянуть этот же channel-aware contract в module-owned admin authoring для
  variant price rows, typed discount preview/apply и active price-list scope без
  отдельного seller/channel portal;
- [x] заменить raw `channel_id/channel_slug` authoring inputs в pricing admin на
  selector поверх `rustok-channel` read model с global fallback и legacy-scope
  compatibility option;
- [x] протянуть effective price context в module-owned storefront/admin read-side surfaces
  через native-first `#[server]` transport с GraphQL fallback;
- [x] добавить explicit channel selector в storefront/admin effective-context controls,
  чтобы channel-aware resolution можно было переключать без raw query editing и без
  возврата к package-local fallback chain;
- [x] перевести admin active `price_list` selector на context-aware read path, чтобы
  список overlays и rule editor пересчитывались по явно выбранному `channel`, а не
  только по bootstrap host context;
- [x] дотянуть тот же selector metadata contract до GraphQL fallback для
  `rustok-pricing/admin` и `rustok-pricing/storefront`, чтобы degraded path не
  терял `available_channels` и channel-aware active `price_lists`;
- [x] перевести GraphQL fallback detail contract на dedicated pricing-facing facade
  roots `adminPricingProduct` / `storefrontPricingProduct`, чтобы degraded path
  сохранял variant-level `effective_price` parity для explicit resolution context;
- [x] отдать active tenant-scoped price lists как pricing-owned read contract,
  чтобы admin/storefront route выбирали overlays без raw UUID-only UX;
- [~] добавить tiers, adjustments и promotion-ready semantics;
- [ ] покрывать deterministic price resolution и rounding targeted tests.

Что уже закрыто дополнительно:

- module-owned `rustok-pricing/admin` теперь имеет targeted SSR tests для native
  `update-variant-price` transport path, включая quantity-tier happy path, active
  `price_list_id` override happy path и permission gate;
- тот же admin transport теперь уже покрывает и typed `preview_percentage_discount` /
  `apply_percentage_discount` path по canonical base-price row, включая targeted SSR
  tests на happy path и permission gate; active `price_list` override adjustment path
  теперь покрыт тем же transport parity слоем;
- runtime tests уже покрывают `set_price_tier` для quantity windows, invalid tier ranges
  и normalized `discount_percent` в `ResolvedPrice`, а admin/storefront surfaces
  уже показывают sale math поверх typed read-side contract.
- тот же runtime теперь ещё и покрывает channel-aware deterministic resolution:
  channel-scoped base row выигрывает у global только при совпавшем host channel,
  а active price list selector не отдаёт channel-scoped list вне его scope.
- legacy `apply_discount` больше не живёт как отдельная ad-hoc mutation: pricing runtime
  теперь держит typed `preview_percentage_discount` / `apply_percentage_discount` поверх
  canonical base-price row, а старый helper остаётся compatibility wrapper.
- promotion-ready semantics тоже уже сдвинулись вперёд: active `price_list` теперь может
  держать typed percentage rule, resolver умеет fallback'иться к base row через это правило,
  а module-owned admin transport уже даёт first-class write path для rule authoring.

### 4. Operability

- [x] документировать новые pricing guarantees одновременно с изменением runtime surface;
- [ ] удерживать local docs и `README.md` синхронизированными;
- [ ] обновлять umbrella commerce docs при изменении pricing/promotion scope.

## Проверка

- `cargo xtask module validate pricing`
- `cargo xtask module test pricing`
- targeted tests для price resolution, pricing transport и money semantics

## Правила обновления

1. При изменении pricing runtime contract сначала обновлять этот файл.
2. При изменении public/runtime surface синхронизировать `README.md`, `admin/README.md`
   и `docs/README.md`.
3. При изменении module metadata синхронизировать `rustok-module.toml`.
4. При изменении pricing/promotion boundary обновлять umbrella commerce docs.
