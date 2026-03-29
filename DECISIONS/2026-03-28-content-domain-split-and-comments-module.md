# Разведение `content`-storage, введение `rustok-comments` и новая роль `rustok-content`

- Date: 2026-03-28
- Status: Accepted

## Context

Текущая модель, где `rustok-blog`, `rustok-forum` и часть `rustok-pages` опираются на shared storage
слой `rustok-content` (`nodes + metadata`), уже создаёт системные проблемы:

- доменные состояния смешиваются с generic `kind` и `metadata`;
- read-path'ы делают лишние дочитывания и теряют фильтры/порядок;
- `blog`, `forum` и `pages` развиваются как разные bounded context, но продолжают делить один
  storage-owner;
- generic `content` REST/GraphQL transport начал закрепляться как product API, хотя целевая модель
  больше не предполагает `rustok-content` как владельца доменных CRUD-поверхностей;
- нужен явный инструмент конверсии `blog post + comments <-> forum topic + replies`, но он не
  должен навязывать общую таблицу для всех discussion-сущностей.

Команда осознанно выбирает сломать текущую архитектурную границу сейчас, пока объём кода и данных
ещё позволяет сделать это контролируемо.

## Decision

### 1. `rustok-content` перестаёт быть shared product storage

`rustok-content` сохраняется в платформе, но его целевая роль меняется:

- shared content library для rich-text/Tiptap contract, locale fallback, slug/canonical helpers;
- orchestration слой для cross-domain операций;
- место для idempotency/audit/conversion records;
- не canonical CRUD/storage backend для `blog`, `forum` и `pages`.

Generic `content` REST/GraphQL/API считается переходным слоем и подлежит сворачиванию после split.

### 2. `rustok-comments` вводится как отдельный optional-модуль

Создаётся новый доменный модуль `rustok-comments` со своей storage boundary для классических
комментариев вне форума:

- комментарии к blog post;
- комментарии к page и другим non-forum content-like сущностям;
- собственные contracts по thread/comment/moderation внутри comment-domain.

На текущем шаге модуль вводится как scaffold и точка фиксации новой границы.

### 3. `forum replies` и `comments` не объединяются

Принимается жёсткое boundary-правило:

- `forum replies != comments`;
- `rustok-forum` остаётся самостоятельным discussion-доменом со своими категориями, топиками,
  replies, moderation, counters и read-model;
- `rustok-comments` не становится storage-базой для форума.

### 4. `rustok-pages` участвует в том же split

`rustok-pages` не рассматривается как вечная специализация shared `content`.
Страницы, блоки и меню в целевой архитектуре должны перейти на page-owned persistence model.

### 5. Конверсия между блогом и форумом делается через orchestration

Поддерживаемые целевые операции:

- `blog post + comments -> forum topic + replies`
- `forum topic + replies -> blog post + comments`

Это явные orchestration/conversion flows, а не live sync и не аргумент в пользу общей таблицы.
После конверсии должен существовать один canonical source для дальнейшего редактирования.

### 6. Legacy-модель подлежит удалению, а не бесконечной адаптации

При выполнении split:

- не расширять generic `content` API новыми продуктовыми сценариями;
- не развивать central string-based `kind` registry как долгосрочную доменную модель;
- не усиливать coupling продуктовых модулей к `NodeService`;
- удалять legacy abstraction после замещения, а не держать её бессрочно ради удобства.

## Consequences

- Нужно создать `rustok-comments` как фактический модуль в workspace, server wiring и документации.
- Нужен staged split `blog`, `forum`, `pages` от shared `rustok-content` storage.
- Документация `rustok-content` должна быть переписана под роль orchestration/shared library.
- `ContentOrchestrationService` нельзя переносить как есть: после split он должен работать через
  domain services, а не через перенос shared `node`-записей.
- Generic `content` transport останется переходным только на время миграции и затем будет удалён.
