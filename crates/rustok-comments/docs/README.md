# Документация модуля `rustok-comments`

`rustok-comments` — доменный модуль для классических комментариев вне форума.

## Назначение

- дать отдельную storage-boundary для комментариев к blog post, страницам и другим non-forum сущностям;
- убрать комментарии из shared `content`-storage модели;
- зафиксировать, что `comments` и `forum replies` — разные доменные сущности;
- подготовить модульную основу для будущих conversion flow между `blog` и `forum` через orchestration.

## Текущий статус

- модуль зарегистрирован в workspace, `modules.toml` и optional server wiring;
- module-owned schema `comment_threads`, `comments`, `comment_bodies` реализована;
- `rustok-blog` уже переведён на `rustok-comments` для comment read/write path;
- shared rich-text/body-format и locale fallback contract выровнены с `rustok-content`;
- page-level интеграция остаётся продуктовым решением, а не обязательной частью модуля.

## Архитектурная граница

- `rustok-comments` владеет только generic comments domain;
- `rustok-forum` продолжает владеть `forum_topics` и `forum_replies`;
- `rustok-content` остаётся shared library + orchestration слоем и не должен снова стать storage owner
  для комментариев;
- будущие conversion flow `post + comments -> topic + replies` и обратно должны жить в orchestration,
  а не через общую таблицу или live sync.

## Дальнейшие шаги

- расширить comment-domain contracts для threading, moderation и target binding;
- решить, какие page-level surfaces тоже используют `rustok-comments`.

## Связанные документы

- [План реализации](./implementation-plan.md)
- [README crate](../README.md)
- [Карта документации](../../../docs/index.md)
