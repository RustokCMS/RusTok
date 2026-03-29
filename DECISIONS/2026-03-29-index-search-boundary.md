# ADR: Граница между `rustok-index` и `rustok-search`

## Статус

Accepted

## Контекст

После выделения `rustok-search` в отдельный core-модуль в репозитории всё ещё
оставался архитектурный риск: `rustok-index` и `rustok-search` решают близкие,
но не одинаковые задачи, и без явной фиксации границы продуктовые search-flow
легко снова начинают стекаться в index/read-model слой.

Это особенно опасно для следующих зон:

- ownership над `search_documents` и search-facing query contract;
- ranking/relevance, dictionaries и merchandising rules;
- admin/storefront search UI и analytics;
- optional connector crates для внешних движков;
- runtime dependency direction между indexing и search capabilities.

## Решение

Принимаем следующую архитектурную границу:

- `rustok-index` остаётся модулем платформенного indexing/read-model substrate.
- `rustok-search` остаётся модулем продуктового поиска и единственным owner'ом
  search-facing API/UX/runtime contract.
- Каноническое search storage (`search_documents`, query analytics, dictionaries,
  query rules) живёт в `rustok-search`, а не в `rustok-index`.
- `rustok-search` может читать domain tables напрямую и при необходимости может
  использовать нейтральные read-model данные из `rustok-index`, но не зависит от
  него как от source-of-truth для продуктового поиска.
- Направление зависимости допускается только как `search -> index`, если это
  реально помогает ingestion/read-model reuse; обратная зависимость
  `index -> search` запрещена.
- Внешние search engines подключаются только через dedicated connector crates,
  зарегистрированные за `rustok-search`; доменные модули не интегрируются с
  provider SDK напрямую.

## Последствия

- Product search contracts, ranking, synonyms/stop words, query rules,
  autocomplete, analytics и search UI не возвращаются в `rustok-index`.
- `rustok-index` может развиваться как общий substrate для denormalized reads,
  sync/rebuild/consistency tooling и cross-module joins без давления со стороны
  storefront/admin UX.
- Любая попытка перенести search-facing API или engine-specific behavior в
  `rustok-index` теперь требует нового ADR.
