# Матрица заимствований (Leptos admin)

Цель: собирать **крупные, активно развиваемые** источники, из которых можно перенять
архитектуру, подходы к UI, auth, data‑fetching, таблицам и медиа.

## Где искать крупные активные проекты (ссылки)

- GitHub search: `leptos admin stars:>20`  
  https://github.com/search?q=leptos+admin+stars%3A%3E20&type=repositories
- GitHub search: `leptos dashboard stars:>20`  
  https://github.com/search?q=leptos+dashboard+stars%3A%3E20&type=repositories
- GitHub search: `leptos cms stars:>20`  
  https://github.com/search?q=leptos+cms+stars%3A%3E20&type=repositories
- GitHub search: `leptos ecommerce stars:>20`  
  https://github.com/search?q=leptos+ecommerce+stars%3A%3E20&type=repositories
- Каталог экосистемы (для индексации, но выбирать **крупные** проекты):  
  https://github.com/leptos-rs/awesome-leptos

> Критерии отбора: свежие коммиты, активные issues/PR, теги релизов, рабочий CI.

## Матрица кандидатов

| Источник | Ссылка | Что можно заимствовать | Статус | Заметки |
| --- | --- | --- | --- | --- |
| AvoRed Leptos admin (cargo) | https://github.com/avored | `console_error_panic_hook`, `console_log`, `leptos_image`, базовый CSR/роутинг | candidate | Уточнить конкретный репозиторий Leptos admin внутри org. |
| Большие Leptos‑проекты (поиск) | https://github.com/search?q=leptos+admin+stars%3A%3E20&type=repositories | UI/архитектура админки, auth, data‑fetching | backlog | Отфильтровать активные репозитории. |
| Большие Leptos‑проекты (поиск) | https://github.com/search?q=leptos+dashboard+stars%3A%3E20&type=repositories | Таблицы, графики, layout | backlog | Отфильтровать активные репозитории. |
| Большие Leptos‑проекты (поиск) | https://github.com/search?q=leptos+cms+stars%3A%3E20&type=repositories | Контентные админки, формы | backlog | Отфильтровать активные репозитории. |

This is an alpha version and requires clarification. Be careful, there may be errors in the text. So that no one thinks that this is an immutable rule.
