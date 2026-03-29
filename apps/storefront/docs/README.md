# Документация Leptos Storefront

Локальная документация для `apps/storefront` как SSR-host приложения витрины.

## Текущий runtime contract

- Host storefront рендерит shell, домашнюю страницу и generic module pages по маршруту `/modules/{route_segment}`.
- Перед SSR для module-route host сначала делает GraphQL preflight `resolveCanonicalRoute` в `apps/server`.
- Если server возвращает alias-hit, storefront отдаёт HTTP redirect на canonical URL до рендера страницы.
- Для canonical lookup параметр `lang` не входит в route key: locale передаётся в query отдельно.
- Если redirect произошёл по alias, storefront сохраняет явно запрошенный `lang` в target URL, чтобы не терять locale SSR.
- Locale lookup внутри canonical preflight идёт с shared fallback policy из `rustok-content`, поэтому alias,
  записанный для `en`, корректно резолвится и для запросов вроде `en-us`, если более точного locale нет.
- Enabled modules резолвятся отдельно и фильтруют storefront registry до рендера.
- Host прокидывает `UiRouteContext` (`locale`, `route_segment`, `query params`) в module-owned storefront packages.
- SSR идёт через in-order HTML streaming, чтобы async module-owned surfaces могли честно получать данные на сервере.

## Generated module UI wiring

- `apps/storefront/build.rs` читает `modules.toml` и модульные `rustok-module.toml`, затем генерирует registry wiring в `OUT_DIR`.
- Publishable storefront UI по-прежнему подключается через `[provides.storefront_ui].leptos_crate`.
- Live generic route `/modules/{route_segment}` остаётся точкой входа для blog, forum, pages, commerce и других publishable storefront packages.

## Canonical routing

- Canonical и alias state хранится не в storefront, а в `rustok-content`.
- Storefront не ходит в БД напрямую и не знает о `content_canonical_urls` / `content_url_aliases`.
- Весь redirect flow идёт через server GraphQL surface, чтобы storefront оставался thin SSR-host.

## Ограничения

- Nested storefront routing и более богатые page-layouts для модулей остаются отдельным слоем поверх текущего generic root-page contract.
- Для внешних crate-ов вне workspace всё ещё нужен publishable storefront package плюс явная server-side install story.

## Связанные документы

- [План реализации](./implementation-plan.md)
- [Заметки по storefront UI](../../../docs/UI/storefront.md)
- [Карта документации](../../../docs/index.md)
