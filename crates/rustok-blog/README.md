# rustok-blog

## Назначение
`rustok-blog` — модуль блогового функционала платформы RusToK, построенный поверх контентного ядра.

## Что делает
- Управляет постами блога (создание, редактирование, публикация, архивирование, удаление)
- Полная поддержка i18n: locale fallback chain `requested → en → first available`
- Поля SEO: `seo_title`, `seo_description`, `featured_image_url`
- Поля i18n в ответах: `effective_locale`, `available_locales`
- Публикует доменные события для синхронизации с индексами
- Типобезопасная state machine управления статусами постов
- Работает с категориями и тегами (через metadata)

## Архитектура

### Wrapper Module
Blog — это "wrapper" модуль:
- **Нет собственных таблиц** — использует таблицы `rustok-content`
- **Добавляет бизнес-логику** — валидация, workflow, SEO-поля
- **Типобезопасная state machine** — управление статусами постов
- **Собственные DomainEvent** — `BlogPostCreated`, `BlogPostPublished` и т.д.

### State Machine

```
  ┌───────┐
  │ Draft │──────────────────┐
  └───┬───┘                  │
      │ publish()            │ archive()
      ↓                      │
  ┌───────────┐              │
  │ Published │──────────────┤
  └─────┬─────┘              │
        │ unpublish()        │ archive()
        │                    ↓
        └─────────→   ┌──────────┐
                      │ Archived │
                      └──────────┘
                         │ restore()
                         ↓
                      ┌───────┐
                      │ Draft │
                      └───────┘
```

### Ключевые компоненты

| Файл | Назначение |
|------|------------|
| `lib.rs` | Точка входа, экспорт API, определение модуля |
| `error.rs` | Обработка ошибок с RichError |
| `locale.rs` | Locale fallback chain |
| `state_machine.rs` | Типобезопасная машина состояний |
| `services/post.rs` | Бизнес-логика постов |
| `dto/` | Data Transfer Objects для API |

## I18n

### Locale fallback chain
1. Запрошенный locale (например `ru`)
2. Английский locale `en`
3. Первый доступный locale

### Поля ответа
- `locale` — запрошенный locale
- `effective_locale` — фактически использованный locale
- `available_locales` — все доступные locales

## События

Модуль публикует следующие `DomainEvent`:
- `BlogPostCreated { post_id, author_id, locale }` — после создания поста
- `BlogPostPublished { post_id, author_id }` — после публикации
- `BlogPostUnpublished { post_id }` — после снятия с публикации
- `BlogPostUpdated { post_id, locale }` — после обновления
- `BlogPostArchived { post_id, reason }` — после архивирования
- `BlogPostDeleted { post_id }` — после удаления

Все события влияют на индекс (`affects_index() = true`).

## Использование

### Создание поста

```rust
use rustok_blog::{PostService, CreatePostInput};

let service = PostService::new(db, event_bus);

let input = CreatePostInput {
    locale: "ru".to_string(),
    title: "Мой первый пост".to_string(),
    body: "Содержимое поста...".to_string(),
    excerpt: Some("Краткое описание".to_string()),
    slug: Some("my-first-post".to_string()),
    publish: false,
    tags: vec!["rust".to_string()],
    category_id: None,
    featured_image_url: Some("https://cdn.example.com/cover.jpg".to_string()),
    seo_title: Some("SEO заголовок".to_string()),
    seo_description: Some("Мета-описание для поисковых систем".to_string()),
    metadata: None,
};

let post_id = service.create_post(tenant_id, security, input).await?;
```

### Получение поста с i18n

```rust
// Запрос конкретного locale с fallback
let post = service.get_post(post_id, "ru").await?;
println!("Effective locale: {}", post.effective_locale);
println!("Available: {:?}", post.available_locales);
```

### REST API

| Метод | Путь | Описание |
|-------|------|----------|
| GET | `/api/blog/posts` | Список постов |
| GET | `/api/blog/posts/{id}?locale=ru` | Получить пост |
| POST | `/api/blog/posts` | Создать пост |
| PUT | `/api/blog/posts/{id}` | Обновить пост |
| DELETE | `/api/blog/posts/{id}` | Удалить пост |
| POST | `/api/blog/posts/{id}/publish` | Опубликовать |
| POST | `/api/blog/posts/{id}/unpublish` | Снять с публикации |

## Взаимодействие

| Модуль | Тип взаимодействия |
|--------|-------------------|
| `rustok-core` | События, типы ошибок, permissions |
| `rustok-content` | Хранение данных (nodes, bodies, translations) |
| `rustok-outbox` | TransactionalEventBus для надёжной доставки событий |
| `rustok-index` | Подписка на события для индексации |

## Permissions

### Posts
- `posts:create`, `posts:read`, `posts:update`, `posts:delete`, `posts:list`, `posts:publish`

### Comments
- `comments:create`, `comments:read`, `comments:update`, `comments:delete`, `comments:list`, `comments:moderate`

### Categories & Tags
- `categories:*`, `tags:*`

## Обработка ошибок

```rust
pub enum BlogError {
    PostNotFound(Uuid),
    CommentNotFound(Uuid),
    DuplicateSlug { slug: String, locale: String },
    CannotDeletePublished,
    CannotPublishArchived,
    AuthorRequired,
    Validation(String),
    // ...
}
```

## Тестирование

```bash
# Unit тесты
cargo test -p rustok-blog

# С property-based тестами
cargo test -p rustok-blog --features proptest

# Интеграционные тесты (требуется БД)
cargo test -p rustok-blog -- --ignored
```

## Документация
- Локальная документация: `./docs/`
- Общая документация платформы: `/docs`

## Паспорт компонента
- **Роль в системе:** Доменный модуль блога
- **Основные данные:** бизнес-логика постов, i18n, SEO-поля, события
- **Взаимодействует с:**
  - `crates/rustok-core` (events, errors, permissions)
  - `crates/rustok-content` (storage)
  - `crates/rustok-outbox` (TransactionalEventBus)
  - `crates/rustok-index` (search indexing)
- **Точки входа:** `crates/rustok-blog/src/lib.rs`
- **Статус:** ✅ Production Ready (Phase 2)
