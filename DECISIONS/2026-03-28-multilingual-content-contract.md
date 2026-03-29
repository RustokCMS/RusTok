# Многоязычный content contract для `blog` / `pages` / `comments`

- Date: 2026-03-28
- Status: Accepted

## Context

После разреза `blog`, `pages` и `comments` по своим таблицам оказалось, что сама
модель многоязычности всё ещё расходится между модулями:

- locale-коды нормализуются по-разному;
- fallback locale в read-path ведёт себя не одинаково;
- `comments` жили на своей самописной locale-резолюции, а не на shared helper;
- `pages` не делали fallback при slug lookup по локали;
- slug policy уже различается между доменами:
  - `blog` использует глобальный canonical slug;
  - `pages` используют locale-aware slug на уровне перевода;
  - `forum` пока ещё остаётся на legacy-модели и должен быть приведён к явному контракту до cutover.

Если это не зафиксировать сейчас, новый split просто размножит несовместимые
locale/slug semantics.

## Decision

### 1. Locale normalization едина для всех content-like модулей

Каноническая normalization rule:

- входной locale trim;
- `_` заменяется на `-`;
- значение приводится к lowercase;
- пустой locale и явно невалидные значения отвергаются.

Shared helper живёт в `rustok-content::locale` и переиспользуется всеми
content-like модулями вместо локальных копий.

### 2. Locale fallback order фиксирован

Канонический порядок locale resolution:

1. requested locale;
2. explicit fallback locale, если он передан вызывающим слоем;
3. platform fallback locale `en`;
4. first available locale.

Это правило считается обязательным для `blog`, `pages`, `comments` и будущего
forum-owned storage после миграции off `NodeService`.

### 3. `rustok-content` остаётся shared owner для locale helpers

`rustok-content` не владеет доменными таблицами `blog/pages/comments`, но остаётся
каноническим местом для:

- locale normalization helpers;
- locale fallback helpers;
- shared rich-text body contracts.

### 4. Slug policy должна быть явной на уровне домена

Платформа не навязывает один slug mode для всех сущностей.

Разрешены два режима, но каждый домен обязан выбрать его явно:

- `canonical/global slug`
  - один slug на сущность независимо от locale;
  - подходит для `blog post` и других canonical publication entities;
- `locale-aware slug`
  - slug живёт на уровне перевода;
  - подходит для `pages` и других locale-routed surfaces.

Смешивать оба режима внутри одной и той же сущности нельзя.

### 5. `forum` должен выбрать slug/locale contract до storage cutover

Перед переносом `forum` на forum-owned persistence нужно явно решить:

- останется ли topic slug locale-aware;
- как forum topic lookup должен использовать tenant fallback locale;
- будет ли forum разделять page-like или blog-like slug semantics.

Это решение должно быть принято до завершения forum storage split, а не после него.

## Consequences

- `blog`, `pages`, `comments` должны использовать shared locale helpers из `rustok-content`.
- Любой новый content-like модуль не должен вводить собственную locale normalization logic.
- `pages` slug lookup должен использовать тот же fallback contract, что и остальные read-path'ы.
- `comments` не должны иметь отдельную locale resolution policy.
- В backlog split-плана нужно отдельно держать решение по `forum` slug/locale semantics.
