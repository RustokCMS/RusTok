# `rustok-pages` не получает default-интеграцию с `rustok-comments`

- Date: 2026-03-29
- Status: Accepted

## Context

После storage split `rustok-blog`, `rustok-pages` и `rustok-comments` границы хранения уже
разведены, но продуктовая граница между `pages` и `comments` оставалась незафиксированной.

Фактическое состояние кода и runtime на текущий момент такое:

- `rustok-blog` уже использует `rustok-comments` как live comment backend;
- `rustok-pages` развивает page-builder, blocks, menus и channel-aware publication surface;
- у `pages` нет встроенного comment transport/UI/runtime path;
- локальные описания `rustok-comments` всё ещё звучали так, будто интеграция с `pages`
  является обязательной или уже активной.

Если считать каждую страницу commentable по умолчанию, это создаёт неявный product contract:

- любой page read-path начинает подразумевать discussion lifecycle и moderation policy;
- статические/landing/help-center страницы получают лишнюю доменную обязанность;
- future integration становится труднее сделать адресной и opt-in.

Нужно явно закрыть этот хвост плана, чтобы `pages` и `comments` не оставались в состоянии
архитектурной двусмысленности.

## Decision

### 1. У `pages` нет default comments surface

В текущем продукте `rustok-pages` не интегрируется с `rustok-comments` по умолчанию.

Это означает:

- нет автоматического создания `comment_threads` для каждой страницы;
- нет обязательного comments UI в `pages` admin/storefront surface;
- нет implicit зависимости `rustok-pages -> rustok-comments`.

### 2. `rustok-comments` остаётся generic backend для opt-in non-forum discussions

`rustok-comments` остаётся каноническим storage-owner для blog comments и для будущих
opt-in non-forum discussion surfaces, но `pages` не считается таким surface автоматически.

### 3. Будущая page-level comments интеграция возможна только как explicit opt-in

Если продукт позже потребует comments на page-like surface, это должно оформляться как
отдельная opt-in интеграция:

- для конкретных page templates / page kinds / dedicated surfaces;
- с отдельной спецификацией по moderation, publication, SEO и storefront rendering;
- без превращения всех `pages` в default discussion targets.

## Consequences

- План `rustok-comments` закрывает пункт про `pages <-> comments` решением "not by default".
- Локальные docs и metadata `rustok-comments` больше не должны утверждать, что `pages`
  уже является live integration target.
- `rustok-pages` продолжает развиваться как page/content presentation module без встроенного
  discussion lifecycle.
- Если later появится commentable knowledge-base/article surface поверх `pages`, это потребует
  отдельного ADR/spec вместо неявного расширения текущего контракта.
