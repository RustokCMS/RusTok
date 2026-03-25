# Документация Leptos Storefront

Локальная документация для `apps/storefront` (Leptos SSR storefront).

## Текущий runtime contract

- Host storefront рендерит shell, домашнюю страницу, generic module pages и slot-based module sections.
- Enabled modules резолвятся отдельно и фильтруют storefront registry перед рендером.
- `StorefrontSlot` теперь поддерживает несколько host extension points для module-owned UI: `HomeAfterHero`, `HomeAfterCatalog`, `HomeBeforeFooter`.
- SSR host теперь рендерит Leptos через in-order HTML streaming, чтобы async module-owned storefront surfaces могли честно получать данные во время server-side render.
- Host также прокидывает module-agnostic `UiRouteContext` (locale, route segment, query params), чтобы publishable storefront packages могли читать generic route state без knowledge о конкретном модуле.

## Generated module UI wiring

- `apps/storefront/build.rs` теперь читает `modules.toml` и модульные `rustok-module.toml`, а затем генерирует manifest-driven storefront registry wiring в `OUT_DIR`.
- Текущий contract для publishable Leptos storefront UI: `[provides.storefront_ui].leptos_crate` плюс экспорт корневого компонента `<PascalSlug>View`, optional `slot`, `route_segment` и `page_title`.
- Live generated wiring регистрирует module-owned storefront sections в выбранный host slot и публикует generic storefront route `/modules/{route_segment}`.
- Референсные publishable storefront packages в workspace сейчас: `rustok-blog-storefront`, `rustok-commerce-storefront`, `rustok-content-storefront`, `rustok-forum-storefront` и `rustok-pages-storefront`.
- `rustok-pages-storefront` остаётся первым data-driven exemplar для page-driven storefront surface.
- `rustok-blog-storefront` теперь служит вторым рабочим эталоном для контентного storefront read-path: пакет сам читает published post по `?slug=` и список публикаций через модульный GraphQL и `UiRouteContext`.
- `rustok-content-storefront` теперь служит третьим рабочим эталоном для generic content storefront read-path: пакет сам резолвит `currentTenant`, читает published nodes и рендерит выбранную публикацию по `?id=` через тот же host-contract.
- `rustok-commerce-storefront` теперь служит рабочим эталоном для commerce storefront read-path: пакет сам читает published products и выбранный product detail через public GraphQL surface модуля и рендерит каталог на generic route `/modules/commerce`.
- `rustok-forum-storefront` теперь служит рабочим эталоном для forum storefront read-path: пакет сам читает categories, topic feed, выбранный thread и replies через public GraphQL surface модуля и рендерит NodeBB-inspired layout на generic route `/modules/forum`.

## Ограничения

- Nested storefront routing и более богатые page layouts для модулей всё ещё остаются отдельным слоем поверх текущего generic root-page contract.
- Для внешних crate-ов вне текущего workspace всё ещё нужен publishable storefront package плюс явный server-side dependency/install story, даже при уже существующем entry-point contract.

## Связанные документы

- [План реализации](./implementation-plan.md)
- [Заметки по storefront UI](../../../docs/UI/storefront.md)
- [Карта документации](../../../docs/index.md)
