# Admin ↔ Server Connection Quickstart

Практическая инструкция для сценария:

- backend (Loco server) установлен в одной папке;
- admin UI установлен в другой папке;
- нужно быстро и без боли подключить админку к серверу.

В документе собраны как автоматизированные методы запуска (Compose/PaaS/k8s), так и ручная установка.

---

## 0) Рекомендуемый режим для отладки: Docker Compose (one-command)

Да, именно так: ставим `docker compose`, поднимаем стек, открываем админку по порту,
логинимся предустановленным пользователем и попадаем в UI без ручной склейки конфигов.

### Что должно быть в compose-стеке

- `server` (Loco backend)
- `admin` (UI)
- `db` (Postgres)
- (опционально) `nginx` как единая точка входа

### Пример потокa запуска

```bash
docker compose up -d --build
docker compose ps
```

После старта:

- Админка: `http://localhost:3000`
- API: `http://localhost:5150/api/graphql` (или через прокси `http://localhost:3000/api/graphql`)

### Предустановленный пользователь для dev

В dev-режиме должен быть seed-скрипт, который создает:

- `ADMIN_EMAIL` (например, `admin@local`)
- `ADMIN_PASSWORD` (например, `admin12345`)
- базовый tenant/workspace

> Важно: dev-учетка только для локальной отладки. В staging/prod seed-пароли запрещены.

### Проверка, что подключение успешно

1. Открыли страницу логина.
2. Вошли seed-учеткой.
3. Увидели профиль (`/me`) и загрузку dashboard.
4. В network видно успешные запросы к `/api/auth/*` и `/api/graphql`.

---

## 0.1 Целевой dev-режим: "одна кнопка" на весь стек

Да — это правильный идеал для вашей команды.

В локальной разработке одной командой должны подниматься:

- `server` (Loco API)
- `admin-next` (starter/admin на Next)
- `admin-leptos` (целевая Leptos-admin)
- `storefront-next` (Next storefront)
- `storefront-leptos` (Leptos storefront)
- `db` (+ опционально `redis`, `mailhog`, `nginx`)

### Рекомендуемая карта портов (пример)

- `server`: `http://localhost:5150`
- `admin-next`: `http://localhost:3000`
- `admin-leptos`: `http://localhost:3001`
- `storefront-next`: `http://localhost:3100`
- `storefront-leptos`: `http://localhost:3101`

### One-command UX

```bash
docker compose --profile full-dev up -d --build
```

После этого разработчик:

1. открывает нужный UI по порту;
2. логинится seed-админом;
3. проверяет, что запросы идут в `server` (`/api/auth/*`, `/api/graphql`).

### Почему это важно

- мгновенный onboarding новых разработчиков;
- одинаковая среда для всей команды;
- быстрые smoke-проверки сразу в 4 UI (2 админки + 2 storefront).

### Практический минимум для реализации

- единый `.env.dev` в корне (shared переменные);
- `docker-compose.yml` + профили (`core`, `full-dev`);
- seed-скрипт в `server` для admin user + tenant;
- healthchecks и `depends_on: condition: service_healthy`.

---

## 1) Самый простой и рекомендуемый вариант: **один домен через reverse proxy**

Идея: браузер открывает админку и отправляет API-запросы на **тот же origin**.
Прокси уже пересылает `/api/*` в backend.

Это избавляет от CORS-проблем и сложной ручной настройки.

### 1.1 Пример структуры на сервере

```text
/opt/rustok/
  server/      # Loco backend
  admin/       # собранная админка (static/SSR)
```

### 1.2 Nginx конфиг (базовый)

```nginx
server {
    listen 80;
    server_name admin.example.com;

    # Админка (статика)
    root /opt/rustok/admin;
    index index.html;

    location / {
        try_files $uri /index.html;
    }

    # API -> backend
    location /api/ {
        proxy_pass http://127.0.0.1:5150/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### 1.3 Что указывать в админке

В этом режиме обычно достаточно:

- `GRAPHQL_ENDPOINT=/api/graphql`
- `AUTH_BASE_URL=/api/auth`

`api_base_url` можно не задавать, если UI и API идут через один host.

---

## 2) Вариант "разные домены": admin отдельно, API отдельно

Пример:

- admin: `https://admin.example.com`
- backend: `https://api.example.com`

Тогда в конфиге админки нужно явно задать:

- `api_base_url=https://api.example.com`
- `graphql_endpoint=/api/graphql`
- `auth_base_url=/api/auth`

И на backend обязательно:

- настроить CORS для `https://admin.example.com`;
- разрешить заголовки `Authorization`, `X-Tenant-Slug`;
- разрешить `credentials`, если используете cookie-based auth.

---

## 3) Минимальный runtime config (что должно быть у админки)

Сводка полей:

- `api_base_url` — базовый URL backend (опционально при same-origin)
- `graphql_endpoint` — обычно `/api/graphql`
- `auth_base_url` — обычно `/api/auth`
- `tenant_slug` — опционально, если нужно предзаполнить tenant
- `app_env` — `local` / `staging` / `production`

---

## 4) Что сделать "тупому пользователю" пошагово (чеклист)

1. Поднять backend и убедиться, что доступен `GET /api/health` (или ваш health endpoint).
2. Положить сборку админки в папку веб-сервера.
3. Включить reverse proxy `/api/* -> backend`.
4. Открыть админку в браузере.
5. Проверить вход (login) и запрос `me`.
6. Проверить GraphQL-запрос к `/api/graphql`.

Если login проходит, `me` работает, и GraphQL отвечает — подключение выполнено корректно.

---

## 5) Диагностика (если не работает)

### Симптом: `401 Unauthorized`

Проверьте:

- отправляется ли `Authorization: Bearer <token>`;
- отправляется ли `X-Tenant-Slug` (если обязателен);
- не протух ли токен.

### Симптом: CORS error

Почти всегда это из-за cross-origin режима.

Быстрый фикс: перейти на same-origin через reverse proxy.

### Симптом: `404 /api/graphql`

Проверьте:

- что прокси действительно пробрасывает `/api/`;
- что backend слушает правильный порт;
- что endpoint именно `/api/graphql`.

---

## 6) Рекомендация для production

Используйте **same-origin схему** (admin + `/api/*` за одним доменом).

Плюсы:

- проще эксплуатация;
- меньше проблем с CORS/cookies;
- предсказуемая конфигурация для пользователей.

---

## 7) Способы установки и запуска

Ниже — нейтральный список вариантов. Без приоритизации по окружениям.

### 7.1 Docker Compose

Подходит, если хотите поднять весь стек одной командой.

```bash
docker compose up -d --build
docker compose ps
```

Для полного локального стека (если настроен профиль):

```bash
docker compose --profile full-dev up -d --build
```

### 7.2 VPS + Docker (+ Nginx/Traefik)

Базовая схема установки:

1. Установить Docker и Docker Compose на VPS.
2. Скопировать проект и `.env`.
3. Запустить сервисы:

```bash
docker compose up -d --build
```

4. Настроить reverse proxy на `/` (admin) и `/api/*` (backend).

### 7.3 Kubernetes (k8s)

Базовая схема установки:

1. Подготовить namespace и секреты.
2. Применить манифесты приложения (Deployments/Services/Ingress).
3. Проверить rollout и доступность:

```bash
kubectl apply -f k8s/
kubectl rollout status deploy/server -n rustok
kubectl get ingress -n rustok
```

### 7.4 Railway / Render / Fly.io

Общий порядок:

1. Подключить репозиторий.
2. Задать переменные окружения.
3. Настроить команду сборки/старта.
4. Проверить, что доступны `/api/auth/*` и `/api/graphql`.

### 7.5 Что проверить после установки (для любого варианта)

1. Открывается страница логина админки.
2. Успешный вход под валидным пользователем.
3. В Network есть успешные запросы к `/api/auth/*` и `/api/graphql`.
4. Открываются основные маршруты админки.

### 7.6 Ручная установка (без Docker)

Если нужен полностью ручной путь:

1. Установить зависимости backend и frontend.
2. Поднять БД и применить миграции backend.
3. Запустить backend на `:5150`.
4. Собрать админку (`build`) и отдать статику через Nginx (или запустить SSR-режим).
5. Настроить reverse proxy `/api/* -> backend`.
6. Проверить login, `/api/auth/*`, `/api/graphql`.

Пример минимального ручного потока (адаптируйте под ваш стек):

```bash
# backend
cd /opt/rustok/server
# install deps
# run migrations
# start server on 5150

# admin
cd /opt/rustok/admin
# install deps
# build
# serve (or run SSR)
```

### 7.7 Одна команда через install-скрипт (план)

Можно сделать единый bootstrap-скрипт по аналогии с one-command setup:

```bash
./scripts/install.sh --mode compose
```

Или для ручного режима:

```bash
./scripts/install.sh --mode manual
```

Что должен делать такой скрипт:

1. Проверять окружение (docker, docker compose, nginx, kubectl — по выбранному режиму).
2. Подготавливать `.env` из шаблона.
3. Поднимать инфраструктуру и сервисы выбранным способом.
4. Выполнять health-check (`/api/health`, `/api/graphql`).
5. Печатать итоговые URL админки и API.

