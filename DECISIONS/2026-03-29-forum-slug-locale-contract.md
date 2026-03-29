# Forum slug/locale contract after content split

- Date: 2026-03-29
- Status: Accepted

## Context

После переноса `forum` на module-owned storage остался последний незакрытый split
вопрос: как именно сочетаются locale fallback и slug semantics на forum read-path.

Базовый multilingual ADR уже зафиксировал общие правила:

- locale normalization идёт через shared helper из `rustok-content`;
- fallback order одинаковый для content-like доменов:
  `requested -> explicit fallback -> en -> first available`;
- `forum` не должен оставаться на неявной legacy slug/locale модели после cutover.

При этом live code форума уже показывает две разные сущности:

- category переводы действительно несут собственный `slug`;
- topic переводы хранят `slug` рядом с переводом, но при создании новой locale
  translation он копируется из seed translation и не используется как отдельный
  locale-routed lookup key;
- public forum API сегодня является ID-based и не обещает `get_by_slug` / list-by-slug
  routing ни для categories, ни для topics.

Нужно зафиксировать это явно, чтобы split можно было закрыть без ложных допущений.

## Decision

### 1. Shared locale contract обязателен и для `forum`

`rustok-forum` использует shared locale normalization/fallback helpers из
`rustok-content` для category/topic/reply read-path.

Все forum read surfaces, где есть locale-sensitive данные, должны согласованно
возвращать:

- `requested_locale`;
- `effective_locale`;
- `available_locales`.

Это правило относится и к detail, и к list DTO/GraphQL surfaces.

### 2. Category slug является locale-aware translation field

`forum_category_translation.slug` считается locale-aware slug на уровне перевода.

Следствия:

- `CategoryResponse` и `CategoryListItem` возвращают slug того же resolved translation,
  что и `name` / `description`;
- при добавлении новой locale translation category slug может отличаться от других
  locale;
- если позже появится lookup category по slug, он обязан использовать тот же
  locale fallback contract, а не обходить его.

### 3. Topic slug остаётся стабильным thread label

`forum_topic_translation.slug` пока не считается отдельным locale-routed slug
контрактом.

Текущая семантика:

- topic slug задаётся при создании темы;
- при добавлении новой locale translation slug по умолчанию копируется из
  seed translation;
- slug выступает как стабильный thread label в ответах, а не как обещанный
  locale-aware route key.

Это сохраняет совместимость текущих DTO/storefront surfaces без введения
несуществующего public routing contract.

### 4. Public forum contract остаётся ID-based

На текущем этапе `forum` не предоставляет канонический public lookup по slug.

Следствия:

- split-track считается закрытым без дополнительного topic/category slug lookup;
- если `get_by_slug` или slug-routed storefront path будет добавлен позже, это
  будет отдельный product/API change и отдельное contract решение;
- такой будущий lookup обязан явно выбрать одну из двух моделей:
  `locale-aware slug` или `stable canonical slug`, и не смешивать их внутри
  одной сущности.

## Consequences

- multilingual ADR для `forum` считается закрытым отдельным domain-specific ADR;
- `rustok-content` остаётся shared owner locale helpers, но не storage-owner forum;
- docs и public contract `rustok-forum` должны описывать category/topic slug
  semantics раздельно;
- split-track `blog / forum / pages off rustok-content` больше не зависит от
  неявной forum slug/locale договорённости.
