# РџР»Р°РЅ rolling-РІРµСЂРёС„РёРєР°С†РёРё С†РµР»РѕСЃС‚РЅРѕСЃС‚Рё СЏРґСЂР° РїР»Р°С‚С„РѕСЂРјС‹

- **РЎС‚Р°С‚СѓСЃ:** РђРєС‚СѓР°Р»РёР·РёСЂРѕРІР°РЅРЅС‹Р№ rolling-С‡РµРєР»РёСЃС‚
- **Р РµР¶РёРј:** РџРѕРІС‚РѕСЂСЏРµРјР°СЏ С‚РѕС‡РµС‡РЅР°СЏ РІРµСЂРёС„РёРєР°С†РёСЏ
- **Р§Р°СЃС‚РѕС‚Р°:** РџРѕСЃР»Рµ Р»СЋР±С‹С… РёР·РјРµРЅРµРЅРёР№ РІ СЏРґСЂРµ, admin-РїР°РЅРµР»СЏС…, core РјРѕРґСѓР»СЏС…, i18n РёР»Рё РєРѕРЅС„РёРіСѓСЂР°С†РёРё module registry
- **Р¦РµР»СЊ:** РЈР±РµРґРёС‚СЊСЃСЏ, С‡С‚Рѕ server + РѕР±Рµ admin-РїР°РЅРµР»Рё + core crates РѕР±СЂР°Р·СѓСЋС‚ СЃР°РјРѕРґРѕСЃС‚Р°С‚РѕС‡РЅРѕРµ СЏРґСЂРѕ, РєРѕС‚РѕСЂРѕРµ СЂР°Р±РѕС‚Р°РµС‚ РїРѕР»РЅРѕСЃС‚СЊСЋ РЅРµР·Р°РІРёСЃРёРјРѕ РѕС‚ РѕРїС†РёРѕРЅР°Р»СЊРЅС‹С… РґРѕРјРµРЅРЅС‹С… РјРѕРґСѓР»РµР№, РїСЂРµРґРѕСЃС‚Р°РІР»СЏРµС‚ РїРѕР»РЅРѕС†РµРЅРЅС‹Р№ РёРЅС‚РµСЂС„РµР№СЃ Рё РїРѕРґРґРµСЂР¶РёРІР°РµС‚ РјРЅРѕРіРѕСЏР·С‹С‡РЅРѕСЃС‚СЊ
- **Companion-РїР»Р°РЅ:** [Р“Р»Р°РІРЅС‹Р№ РїР»Р°РЅ РІРµСЂРёС„РёРєР°С†РёРё РїР»Р°С‚С„РѕСЂРјС‹](./PLATFORM_VERIFICATION_PLAN.md)

**РџСЂРёРЅС†РёРї СЂР°Р±РѕС‚С‹ СЃ РїР»Р°РЅРѕРј:**
РџСЂРё РїСЂРѕРіРѕРЅРµ вЂ” **СѓСЃС‚СЂР°РЅСЏС‚СЊ РЅР°Р№РґРµРЅРЅС‹Рµ РїСЂРѕР±Р»РµРјС‹ СЃСЂР°Р·Сѓ**, РІ С‚РѕР№ Р¶Рµ СЃРµСЃСЃРёРё. РќР°Р№РґРµРЅРЅР°СЏ РїСЂРѕР±Р»РµРјР° РЅРµ Р·Р°РєСЂС‹РІР°РµС‚СЃСЏ РґРѕ РјРѕРјРµРЅС‚Р° РёСЃРїСЂР°РІР»РµРЅРёСЏ. РќРµСЂРµС€С‘РЅРЅС‹Рµ Р±Р»РѕРєРµСЂС‹ С„РёРєСЃРёСЂСѓСЋС‚СЃСЏ РІ Р°СЂС‚РµС„Р°РєС‚Рµ РїСЂРѕРіРѕРЅР° РєР°Рє open blocker. РџРѕСЃР»Рµ РёСЃРїСЂР°РІР»РµРЅРёСЏ вЂ” РїРµСЂРµРїСЂРѕРІРµСЂРёС‚СЊ СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓСЋС‰СѓСЋ С„Р°Р·Сѓ.

**Р›РµРіРµРЅРґР° СЂРµР¶РёРјРѕРІ:**
- `рџ”§ Static` вЂ” РІС‹РїРѕР»РЅРёРјРѕ Р±РµР· Р·Р°РїСѓС‰РµРЅРЅРѕР№ РёРЅС„СЂР°СЃС‚СЂСѓРєС‚СѓСЂС‹ (cargo build/check/test, npm build/lint, git grep)
- `рџЊђ Runtime` вЂ” С‚СЂРµР±СѓРµС‚ Р·Р°РїСѓС‰РµРЅРЅС‹С… PostgreSQL + server (Рё РѕРїС†РёРѕРЅР°Р»СЊРЅРѕ Iggy)

---

## 0. РџСЂРµРґРІР°СЂРёС‚РµР»СЊРЅС‹Рµ СѓСЃР»РѕРІРёСЏ

> Р•СЃР»Рё СЃСЂРµРґР° РЅРµ РїРѕРґРґРµСЂР¶РёРІР°РµС‚ Р·Р°РїСѓСЃРє РёРЅС„СЂР°СЃС‚СЂСѓРєС‚СѓСЂС‹ вЂ” `рџЊђ Runtime`-РїСЂРѕРІРµСЂРєРё РїСЂРѕРїСѓСЃРєР°СЋС‚СЃСЏ, РІС‹РїРѕР»РЅСЏСЋС‚СЃСЏ С‚РѕР»СЊРєРѕ `рџ”§ Static`.

- `рџ”§` `docker compose config` РїСЂРѕС…РѕРґРёС‚ Р±РµР· РѕС€РёР±РѕРє.
- `рџ”§` `.env` РёР»Рё `.env.dev` СЃРѕРґРµСЂР¶РёС‚ РєРѕСЂСЂРµРєС‚РЅС‹Рµ РїРµСЂРµРјРµРЅРЅС‹Рµ РґР»СЏ СЃРѕРµРґРёРЅРµРЅРёСЏ СЃ DB Рё Iggy.
- `рџЊђ` PostgreSQL Р·Р°РїСѓС‰РµРЅ Рё РґРѕСЃС‚СѓРїРµРЅ (РїРѕСЂС‚ 5432) вЂ” `docker compose up -d db`
- `рџЊђ` Iggy event broker Р·Р°РїСѓС‰РµРЅ (TCP РїРѕСЂС‚ 8090) вЂ” `docker compose up -d iggy` вЂ” **РµСЃР»Рё СЃРєРѕРЅС„РёРіСѓСЂРёСЂРѕРІР°РЅ Iggy-С‚СЂР°РЅСЃРїРѕСЂС‚** (`rustok-iggy` / `rustok-iggy-connector`). `rustok-outbox` transport-agnostic: РїСЂРё РёСЃРїРѕР»СЊР·РѕРІР°РЅРёРё РґСЂСѓРіРѕРіРѕ С‚СЂР°РЅСЃРїРѕСЂС‚Р° РёР»Рё DB-СЂРµР¶РёРјР° Iggy РЅРµ РѕР±СЏР·Р°С‚РµР»РµРЅ.

---

## 1. РЎРѕСЃС‚Р°РІ СЏРґСЂР° РїР»Р°С‚С„РѕСЂРјС‹

Р­С‚РѕС‚ РїР»Р°РЅ РІРµСЂРёС„РёС†РёСЂСѓРµС‚ С‚РѕР»СЊРєРѕ СЃР»РµРґСѓСЋС‰РёРµ РєРѕРјРїРѕРЅРµРЅС‚С‹ РєР°Рє РµРґРёРЅРѕРµ С†РµР»РѕРµ:

### 1.1 РџСЂРёР»РѕР¶РµРЅРёСЏ

- **Server:** `apps/server`
- **Admin РїР°РЅРµР»СЊ #1:** `apps/admin` (Leptos CSR)
- **Admin РїР°РЅРµР»СЊ #2:** `apps/next-admin` (Next.js 16 + React 19)

### 1.2 Core crates

- `rustok-core` вЂ” РёРЅС„СЂР°СЃС‚СЂСѓРєС‚СѓСЂРЅС‹Рµ РєРѕРЅС‚СЂР°РєС‚С‹, С‚РёРїС‹ РѕС€РёР±РѕРє, cache abstractions
- `rustok-auth` вЂ” Р¶РёР·РЅРµРЅРЅС‹Р№ С†РёРєР» Р°СѓС‚РµРЅС‚РёС„РёРєР°С†РёРё, JWT, OAuth2 AS
- `rustok-rbac` вЂ” СЂРѕР»РµРІР°СЏ РјРѕРґРµР»СЊ РґРѕСЃС‚СѓРїР°, typed permissions
- `rustok-cache` вЂ” Р°Р±СЃС‚СЂР°РєС†РёСЏ РєСЌС€Р° (in-memory / Redis)
- `rustok-tenant` вЂ” multi-tenancy: СЂРµР·РѕР»СЋС†РёСЏ, РёР·РѕР»СЏС†РёСЏ, cache
- `rustok-events` вЂ” domain event definitions Рё contracts
- `rustok-outbox` вЂ” transactional outbox, transport-agnostic event relay; РєРѕРЅРєСЂРµС‚РЅС‹Р№ С‚СЂР°РЅСЃРїРѕСЂС‚ (Iggy Рё РґСЂ.) РїРѕРґРєР»СЋС‡Р°РµС‚СЃСЏ РѕРїС†РёРѕРЅР°Р»СЊРЅРѕ С‡РµСЂРµР· `rustok-iggy` / `rustok-iggy-connector`
- `rustok-search` вЂ” РїРѕРёСЃРєРѕРІС‹Р№ РґРІРёР¶РѕРє, PgSearch / РёРЅРґРµРєСЃ
- `rustok-index` вЂ” CQRS read models, РґРµРЅРѕСЂРјР°Р»РёР·Р°С†РёСЏ
- `rustok-telemetry` вЂ” OpenTelemetry, tracing, Prometheus
- `rustok-api` вЂ” shared host/API layer: TenantContext, AuthContext
- `rustok-email` вЂ” email service abstraction

### 1.3 Р“СЂР°РЅРёС†Р°

РЎР»РµРґСѓСЋС‰РёРµ РєРѕРјРїРѕРЅРµРЅС‚С‹ **РЅРµ РІС…РѕРґСЏС‚** РІ РѕР±Р»Р°СЃС‚СЊ СЌС‚РѕРіРѕ РїР»Р°РЅР°:

- РћРїС†РёРѕРЅР°Р»СЊРЅС‹Рµ РґРѕРјРµРЅРЅС‹Рµ РјРѕРґСѓР»Рё: `rustok-content`, `rustok-commerce`, `rustok-blog`, `rustok-forum`, `rustok-pages`, `rustok-media`, `rustok-workflow`
- Capability-СЃР»РѕРё: `flex`, `alloy`, `alloy`, `rustok-mcp`
- РС… UI, С‚РµСЃС‚С‹ Рё РёРЅС‚РµРіСЂР°С†РёРё РІРµСЂРёС„РёС†РёСЂСѓСЋС‚СЃСЏ РІ РїСЂРѕС„РёР»СЊРЅС‹С… РїР»Р°РЅР°С…

---

## 2. РРЅРІР°СЂРёР°РЅС‚С‹ СЏРґСЂР°

- `рџ”§` Core crates РЅРµ РёРјРїРѕСЂС‚РёСЂСѓСЋС‚ РѕРїС†РёРѕРЅР°Р»СЊРЅС‹Рµ РґРѕРјРµРЅРЅС‹Рµ crates (content, commerce, blog, forum, pages, media, workflow).
- `рџ”§` `rustok-core` РЅРµ СЃРѕРґРµСЂР¶РёС‚ РґРѕРјРµРЅРЅС‹С… С‚Р°Р±Р»РёС† вЂ” С‚РѕР»СЊРєРѕ РёРЅС„СЂР°СЃС‚СЂСѓРєС‚СѓСЂРЅС‹Рµ РєРѕРЅС‚СЂР°РєС‚С‹.
- `рџ”§` РњРѕРґСѓР»Рё СЃ `ModuleKind::Core` РїРѕРјРµС‡РµРЅС‹ `required = true` РІ `modules.toml`.
- `рџ”§` `registry.is_core()` Р·Р°РїСЂРµС‰Р°РµС‚ РѕС‚РєР»СЋС‡РµРЅРёРµ core РјРѕРґСѓР»РµР№ С‡РµСЂРµР· tenant API.
- `рџ”§` `rustok-outbox` СЏРІР»СЏРµС‚СЃСЏ `Core` РјРѕРґСѓР»РµРј Р±РµР· tenant-toggle semantics.
- `рџ”§` Р’ `build_registry()` РѕС‚СЃСѓС‚СЃС‚РІСѓСЋС‚ С†РёРєР»РёС‡РµСЃРєРёРµ Р·Р°РІРёСЃРёРјРѕСЃС‚Рё РјРµР¶РґСѓ core crates.

---

## 3. Boot Р±РµР· РѕРїС†РёРѕРЅР°Р»СЊРЅС‹С… РјРѕРґСѓР»РµР№

**Р¤Р°Р№Р»С‹:**
- `apps/server/src/app.rs`
- `apps/server/src/modules/mod.rs`
- `apps/server/src/modules/manifest.rs`
- `modules.toml`

- `рџ”§` `cargo build -p rustok-server` РїСЂРѕС…РѕРґРёС‚.
- `рџЊђ` Server СЃС‚Р°СЂС‚СѓРµС‚ СЃ РІРєР»СЋС‡С‘РЅРЅС‹РјРё С‚РѕР»СЊРєРѕ core РјРѕРґСѓР»СЏРјРё.
- `рџЊђ` `validate_registry_vs_manifest()` РІС‹Р·С‹РІР°РµС‚СЃСЏ РїСЂРё СЃС‚Р°СЂС‚Рµ Рё РїСЂРѕС…РѕРґРёС‚ Р±РµР· РѕС€РёР±РѕРє.
- `рџЊђ` РњРёРіСЂР°С†РёРё (`cargo loco db migrate`) РїСЂРѕС…РѕРґСЏС‚ Р±РµР· РґРѕРјРµРЅРЅС‹С… РјРѕРґСѓР»СЊРЅС‹С… РјРёРіСЂР°С†РёР№.
- `рџЊђ` Server Р·Р°РІРµСЂС€Р°РµС‚ bootstrap Р±РµР· `unwrap()` РїР°РЅРёРє, СЃРІСЏР·Р°РЅРЅС‹С… СЃ РѕС‚СЃСѓС‚СЃС‚РІРёРµРј domain РјРѕРґСѓР»РµР№.
- `рџЊђ` `/api/health` РІРѕР·РІСЂР°С‰Р°РµС‚ HTTP 200.

---

## 4. Auth РІ РёР·РѕР»СЏС†РёРё

**Р¤Р°Р№Р»С‹:**
- `crates/rustok-auth/`
- `apps/server/src/controllers/auth.rs`
- `apps/server/src/controllers/oauth.rs`
- `apps/server/src/services/auth_lifecycle.rs`

> РџРѕР»РЅС‹Р№ auth/RBAC-Р°СѓРґРёС‚ (JWT-РєРѕРЅС‚СЂР°РєС‚, permission enforcement, hardcoded roles) вЂ” РІ [platform-foundation-verification-plan.md](./platform-foundation-verification-plan.md).

- `рџЊђ` Sign up СЂР°Р±РѕС‚Р°РµС‚ Р±РµР· РѕРїС†РёРѕРЅР°Р»СЊРЅРѕРіРѕ РјРѕРґСѓР»СЏ.
- `рџЊђ` Sign in (email + password) СЂР°Р±РѕС‚Р°РµС‚ Р±РµР· РѕРїС†РёРѕРЅР°Р»СЊРЅРѕРіРѕ РјРѕРґСѓР»СЏ.
- `рџЊђ` Token refresh СЂР°Р±РѕС‚Р°РµС‚.
- `рџЊђ` Logout Рё session invalidation СЂР°Р±РѕС‚Р°СЋС‚.
- `рџЊђ` Password reset flow СЂР°Р±РѕС‚Р°РµС‚.
- `рџЊђ` OAuth2 Authorization Server (PKCE flow, client credentials) СЂР°Р±РѕС‚Р°РµС‚ РєР°Рє С‡Р°СЃС‚СЊ СЏРґСЂР°.

---

## 5. Multi-tenancy core

**Р¤Р°Р№Р»С‹:**
- `crates/rustok-tenant/`
- `apps/server/src/middleware/tenant.rs`

> РџРѕР»РЅС‹Р№ tenancy-Р°СѓРґРёС‚ (cache, stampede, Redis invalidation) вЂ” РІ [platform-foundation-verification-plan.md](./platform-foundation-verification-plan.md).

- `рџЊђ` Tenant resolution (hostname/header-based) СЂР°Р±РѕС‚Р°РµС‚ РїСЂРё С‡РёСЃС‚РѕРј СЃС‚Р°СЂС‚Рµ.
- `рџЊђ` Core РјРѕРґСѓР»Рё РІСЃРµРіРґР° РІРєР»СЋС‡РµРЅС‹ вЂ” РїРѕРїС‹С‚РєР° disable С‡РµСЂРµР· API РІРѕР·РІСЂР°С‰Р°РµС‚ РѕС€РёР±РєСѓ.
- `рџ”§` `tenant_modules` РєРѕСЂСЂРµРєС‚РЅРѕ РѕС‚СЂР°Р¶Р°РµС‚ core РјРѕРґСѓР»Рё РєР°Рє РЅРµ-toggleable.

---

## 6. РћР±Рµ admin-РїР°РЅРµР»Рё вЂ” С„СѓРЅРєС†РёРѕРЅР°Р»СЊРЅР°СЏ РїРѕР»РЅРѕС‚Р°

Admin-РїР°РЅРµР»Рё РїСЂРµРґРѕСЃС‚Р°РІР»СЏСЋС‚ **РїРѕР»РЅРѕС†РµРЅРЅС‹Р№ РёРЅС‚РµСЂС„РµР№СЃ** СѓРїСЂР°РІР»РµРЅРёСЏ СЏРґСЂРѕРј РїР»Р°С‚С„РѕСЂРјС‹, Р° РЅРµ РіРѕР»С‹Р№ РґР°С€Р±РѕСЂРґ. РљР°Р¶РґС‹Р№ РїСѓРЅРєС‚ РјРµРЅСЋ вЂ” СЌС‚Рѕ UI, РїСЂРµРґРѕСЃС‚Р°РІР»СЏРµРјС‹Р№ РєРѕРЅРєСЂРµС‚РЅС‹Рј core РјРѕРґСѓР»РµРј.

### 6.1 Р¤СѓРЅРєС†РёРѕРЅР°Р»СЊРЅС‹Рµ СЂР°Р·РґРµР»С‹

| РџСѓРЅРєС‚ РјРµРЅСЋ | Core РјРѕРґСѓР»СЊ вЂ” РёСЃС‚РѕС‡РЅРёРє UI |
|------------|---------------------------|
| РџРѕР»СЊР·РѕРІР°С‚РµР»Рё (Users) | `rustok-auth` |
| РЎРµСЃСЃРёРё (Sessions) | `rustok-auth` |
| Р РѕР»Рё Рё СЂР°Р·СЂРµС€РµРЅРёСЏ (Roles & Permissions) | `rustok-rbac` |
| Tenant-С‹ / РћСЂРіР°РЅРёР·Р°С†РёРё | `rustok-tenant` |
| РЈРїСЂР°РІР»РµРЅРёРµ РјРѕРґСѓР»СЏРјРё | server / module registry |
| Email-РЅР°СЃС‚СЂРѕР№РєРё | `rustok-email` |
| РљСЌС€ (Cache management) | `rustok-cache` |
| OAuth РїСЂРёР»РѕР¶РµРЅРёСЏ | `rustok-auth` (OAuth2 AS) |
| РќР°СЃС‚СЂРѕР№РєРё РїР»Р°С‚С„РѕСЂРјС‹ (Settings) | `rustok-core` |
| Р›РѕРєР°Р»РёР·Р°С†РёСЏ / РњРЅРѕРіРѕСЏР·С‹С‡РЅРѕСЃС‚СЊ | i18n layer (СЃРј. С„Р°Р·Сѓ 7) |

### 6.2 Leptos Admin (`apps/admin`)

- `рџ”§` `cargo build -p rustok-admin` РїСЂРѕС…РѕРґРёС‚.
- `рџЊђ` РџСЂРёР»РѕР¶РµРЅРёРµ Р·Р°РїСѓСЃРєР°РµС‚СЃСЏ Рё СѓСЃС‚Р°РЅР°РІР»РёРІР°РµС‚ СЃРѕРµРґРёРЅРµРЅРёРµ СЃ server.
- `рџЊђ` РђСѓС‚РµРЅС‚РёС„РёРєР°С†РёСЏ СЂР°Р±РѕС‚Р°РµС‚ С‡РµСЂРµР· GraphQL auth flow.
- `рџЊђ` Dashboard Р·Р°РіСЂСѓР¶Р°РµС‚СЃСЏ РїРѕСЃР»Рµ СѓСЃРїРµС€РЅРѕРіРѕ РІС…РѕРґР°.
- `рџЊђ` Р’СЃРµ С„СѓРЅРєС†РёРѕРЅР°Р»СЊРЅС‹Рµ СЂР°Р·РґРµР»С‹ РёР· С‚Р°Р±Р»РёС†С‹ 6.1 РїСЂРёСЃСѓС‚СЃС‚РІСѓСЋС‚ РІ РЅР°РІРёРіР°С†РёРё.
- `рџЊђ` РљР°Р¶РґС‹Р№ СЂР°Р·РґРµР», С‡РµР№ backend-РјРѕРґСѓР»СЊ РІРєР»СЋС‡С‘РЅ, РѕС‚РѕР±СЂР°Р¶Р°РµС‚ СЂР°Р±РѕС‡РёР№ РёРЅС‚РµСЂС„РµР№СЃ.
- `рџЊђ` Р Р°Р·РґРµР»С‹ Р±РµР· РІРєР»СЋС‡С‘РЅРЅРѕРіРѕ backend-РјРѕРґСѓР»СЏ РґРµРіСЂР°РґРёСЂСѓСЋС‚ РєРѕСЂСЂРµРєС‚РЅРѕ (РЅРµС‚ РєСЂР°С€Р°, РЅРµС‚ 500).
- `рџ”§` Module-owned routing (`/modules/:module_slug/*`) Р·Р°СЂРµРіРёСЃС‚СЂРёСЂРѕРІР°РЅ РґР»СЏ core РјРѕРґСѓР»РµР№.

### 6.3 Next.js Admin (`apps/next-admin`)

- `рџ”§` `npm run build` РїСЂРѕС…РѕРґРёС‚.
- `рџ”§` `npm run lint` РїСЂРѕС…РѕРґРёС‚.
- `рџ”§` `npm run typecheck` РїСЂРѕС…РѕРґРёС‚.
- `рџЊђ` РџСЂРёР»РѕР¶РµРЅРёРµ Р·Р°РїСѓСЃРєР°РµС‚СЃСЏ Рё СѓСЃС‚Р°РЅР°РІР»РёРІР°РµС‚ СЃРѕРµРґРёРЅРµРЅРёРµ СЃ server.
- `рџЊђ` РђСѓС‚РµРЅС‚РёС„РёРєР°С†РёСЏ СЂР°Р±РѕС‚Р°РµС‚ С‡РµСЂРµР· NextAuth credentials flow.
- `рџЊђ` Dashboard Р·Р°РіСЂСѓР¶Р°РµС‚СЃСЏ РїРѕСЃР»Рµ СѓСЃРїРµС€РЅРѕРіРѕ РІС…РѕРґР°.
- `рџЊђ` Р’СЃРµ С„СѓРЅРєС†РёРѕРЅР°Р»СЊРЅС‹Рµ СЂР°Р·РґРµР»С‹ РёР· С‚Р°Р±Р»РёС†С‹ 6.1 РїСЂРёСЃСѓС‚СЃС‚РІСѓСЋС‚ РІ РЅР°РІРёРіР°С†РёРё.
- `рџЊђ` РљР°Р¶РґС‹Р№ СЂР°Р·РґРµР», С‡РµР№ backend-РјРѕРґСѓР»СЊ РІРєР»СЋС‡С‘РЅ, РѕС‚РѕР±СЂР°Р¶Р°РµС‚ СЂР°Р±РѕС‡РёР№ РёРЅС‚РµСЂС„РµР№СЃ.
- `рџЊђ` Р Р°Р·РґРµР»С‹ Р±РµР· РІРєР»СЋС‡С‘РЅРЅРѕРіРѕ backend-РјРѕРґСѓР»СЏ РґРµРіСЂР°РґРёСЂСѓСЋС‚ РєРѕСЂСЂРµРєС‚РЅРѕ (РЅРµС‚ РєСЂР°С€Р°, РЅРµС‚ 500).

---

## 7. РњРЅРѕРіРѕСЏР·С‹С‡РЅРѕСЃС‚СЊ (i18n) РєР°Рє С‡Р°СЃС‚СЊ СЏРґСЂР°

РџРѕРґРґРµСЂР¶РєР° РјРЅРѕРіРѕСЏР·С‹С‡РЅРѕСЃС‚Рё вЂ” РїР»Р°С‚С„РѕСЂРјРµРЅРЅР°СЏ С„СѓРЅРєС†РёСЏ, Р° РЅРµ РґРѕРјРµРЅРЅС‹Р№ РјРѕРґСѓР»СЊ.

### 7.1 Server / API

- `рџЊђ` API РІРѕР·РІСЂР°С‰Р°РµС‚ Р»РѕРєР°Р»РёР·РѕРІР°РЅРЅС‹Рµ СЃРѕРѕР±С‰РµРЅРёСЏ РѕР± РѕС€РёР±РєР°С… РїСЂРё Р·Р°РїСЂРѕСЃРµ СЃ Р·Р°РіРѕР»РѕРІРєРѕРј `Accept-Language`.
- `рџЊђ` Auth messages (РѕС€РёР±РєРё РІР°Р»РёРґР°С†РёРё, email-С‚РµРєСЃС‚С‹) Р»РѕРєР°Р»РёР·РѕРІР°РЅС‹.
- `рџЊђ` GraphQL API РїРѕРґРґРµСЂР¶РёРІР°РµС‚ РїРµСЂРµРґР°С‡Сѓ locale С‡РµСЂРµР· РїР°СЂР°РјРµС‚СЂ РёР»Рё Р·Р°РіРѕР»РѕРІРѕРє.

### 7.2 Leptos Admin вЂ” РїРѕРєСЂС‹С‚РёРµ locale-РєР»СЋС‡Р°РјРё

Locale-С„Р°Р№Р»С‹: `apps/admin/locales/en.json` Рё `apps/admin/locales/ru.json`.

**РЎРёРЅС…СЂРѕРЅРЅРѕСЃС‚СЊ С„Р°Р№Р»РѕРІ:**

- `рџ”§` EN Рё RU С„Р°Р№Р»С‹ СЃРѕРґРµСЂР¶Р°С‚ РѕРґРёРЅР°РєРѕРІС‹Рµ top-level namespace-РєР»СЋС‡Рё:
  ```
  git diff --no-index \
    <(jq 'keys' apps/admin/locales/en.json) \
    <(jq 'keys' apps/admin/locales/ru.json)
  ```
  РћР¶РёРґР°РµРјС‹Р№ СЂРµР·СѓР»СЊС‚Р°С‚: diff РїСѓСЃС‚РѕР№.

**РџРѕРєСЂС‹С‚РёРµ СЃС‚СЂР°РЅРёС†** (РєР°Р¶РґР°СЏ СЃС‚СЂР°РЅРёС†Р° РґРѕР»Р¶РЅР° РёСЃРїРѕР»СЊР·РѕРІР°С‚СЊ `use_i18n()` Рё `t_string!()`):

- `рџ”§` `pages/dashboard.rs` вЂ” namespace `app.dashboard.*` вњ“
- `рџ”§` `pages/login.rs` вЂ” namespace `auth.*` вњ“
- `рџ”§` `pages/register.rs` вЂ” namespace `register.*` вњ“
- `рџ”§` `pages/reset.rs` вЂ” namespace `reset.*` вњ“
- `рџ”§` `pages/profile.rs` вЂ” namespace `profile.*` вњ“
- `рџ”§` `pages/security.rs` вЂ” namespace `security.*` вњ“
- `рџ”§` `pages/users.rs` вЂ” namespace `users.*` вњ“
- `рџ”§` `pages/user_details.rs` вЂ” namespace `users.*` вњ“
- `рџ”§` `pages/modules.rs` вЂ” namespace `modules.*` вњ“
- `рџ”§` `pages/module_admin.rs` вЂ” namespace `modules.moduleDisabled`, `modules.moduleNotFound` вњ“
- `рџ”§` `pages/cache.rs` вЂ” namespace `cache.*` вњ“
- `рџ”§` `pages/email_settings.rs` вЂ” namespace `email.*` вњ“
- `рџ”§` `pages/roles.rs` вЂ” namespace `roles.*` вњ“
- `рџ”§` `pages/oauth_apps.rs` вЂ” namespace `oauthApps.*` вњ“
- `рџ”§` `pages/events.rs` вЂ” namespace `events.*` вњ“
- `рџ”§` `pages/workflows.rs` вЂ” namespace `workflows.*` вњ“
- `рџ”§` `pages/workflow_detail.rs` вЂ” namespace `workflows.*` вњ“
- `рџ”§` `pages/not_found.rs` вЂ” namespace `app.notFound.*` вњ“

**РџСЂРѕРІРµСЂРєР° РѕС‚СЃСѓС‚СЃС‚РІРёСЏ С…Р°СЂРґРєРѕРґРЅС‹С… СЃС‚СЂРѕРє** (РЅРё РѕРґРЅР° СЃС‚СЂР°РЅРёС†Р° РЅРµ РґРѕР»Р¶РЅР° СЃРѕРґРµСЂР¶Р°С‚СЊ РЅРµРїРµСЂРµРІРµРґС‘РЅРЅС‹Рµ Р°РЅРіР»РёР№СЃРєРёРµ/СЂСѓСЃСЃРєРёРµ СЃС‚СЂРѕРєРё РІ view! macro):

```bash
# РџРѕРёСЃРє РїРѕС‚РµРЅС†РёР°Р»СЊРЅРѕ РЅРµРїРµСЂРµРІРµРґС‘РЅРЅС‹С… СЃС‚СЂРѕРє РІ .rs СЃС‚СЂР°РЅРёС†Р°С…
grep -rn '"[A-Z][a-z]' apps/admin/src/pages/ | grep -v '//' | grep -v 'placeholder' | grep -v 'class=' | grep -v 'href='
```

**РћР±СЏР·Р°С‚РµР»СЊРЅС‹Рµ locale namespaces РІ en.json:**

```bash
jq 'keys' apps/admin/locales/en.json
# РћР¶РёРґР°РµС‚СЃСЏ: ["app","auth","cache","email","errors","events","modules","oauthApps","profile","register","reset","roles","security","users","workflows"]
```

### 7.3 Leptos Admin вЂ” runtime РїРµСЂРµРєР»СЋС‡РµРЅРёРµ

- `рџЊђ` Р Р°Р·РґРµР» СѓРїСЂР°РІР»РµРЅРёСЏ СЏР·С‹РєР°РјРё / РїРµСЂРµРІРѕРґР°РјРё РїСЂРёСЃСѓС‚СЃС‚РІСѓРµС‚ РІ РЅР°РІРёРіР°С†РёРё.
- `рџЊђ` UI РєРѕСЂСЂРµРєС‚РЅРѕ РїРµСЂРµРєР»СЋС‡Р°РµС‚СЃСЏ РјРµР¶РґСѓ СЏР·С‹РєР°РјРё (РјРёРЅРёРјСѓРј: RU, EN).
- `рџЊђ` Р¤РѕСЂРјР°С‚РёСЂРѕРІР°РЅРёРµ РґР°С‚ Рё С‡РёСЃРµР» СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓРµС‚ Р°РєС‚РёРІРЅРѕРјСѓ locale.

### 7.4 Next.js Admin вЂ” РїРѕРєСЂС‹С‚РёРµ locale-РєР»СЋС‡Р°РјРё

Locale-С„Р°Р№Р»С‹: `apps/next-admin/messages/en.json` Рё `apps/next-admin/messages/ru.json`.

**РЎРёРЅС…СЂРѕРЅРЅРѕСЃС‚СЊ С„Р°Р№Р»РѕРІ:**

- `рџ”§` EN Рё RU С„Р°Р№Р»С‹ СЃРѕРґРµСЂР¶Р°С‚ РѕРґРёРЅР°РєРѕРІС‹Рµ top-level namespace-РєР»СЋС‡Рё:
  ```bash
  git diff --no-index \
    <(jq 'keys' apps/next-admin/messages/en.json) \
    <(jq 'keys' apps/next-admin/messages/ru.json)
  ```

**РџРѕРєСЂС‹С‚РёРµ РєРѕРјРїРѕРЅРµРЅС‚РѕРІ** (client components РґРѕР»Р¶РЅС‹ РёСЃРїРѕР»СЊР·РѕРІР°С‚СЊ `useTranslations()`):

- `рџ”§` `features/cache/components/cache-status.tsx` вЂ” namespace `cache.*` вњ“
- `рџ”§` `features/email/components/email-settings-form.tsx` вЂ” namespace `email.*` вњ“
- `рџ”§` `features/rbac/components/roles-table.tsx` вЂ” namespace `roles.*` вњ“
- `рџ”§` `features/events/components/events-form.tsx` вЂ” namespace `events.*` вњ“
- `рџ”§` `features/modules/components/modules-list.tsx` вЂ” namespace `modules.*` вњ“

**РћР±СЏР·Р°С‚РµР»СЊРЅС‹Рµ locale namespaces РІ messages/en.json:**

```bash
jq 'keys' apps/next-admin/messages/en.json
# РћР¶РёРґР°РµС‚СЃСЏ: ["app","auth","cache","email","errors","events","modules","profile","register","reset","roles","security","users","workflows"]
```

### 7.5 Next.js Admin вЂ” runtime РїРµСЂРµРєР»СЋС‡РµРЅРёРµ

- `рџ”§` `next-intl` РЅР°СЃС‚СЂРѕРµРЅ Рё РїРѕРґРєР»СЋС‡С‘РЅ (`apps/next-admin/`).
- `рџЊђ` Р РѕСѓС‚РёРЅРі СЃ locale-РїСЂРµС„РёРєСЃРѕРј СЂР°Р±РѕС‚Р°РµС‚ РєРѕСЂСЂРµРєС‚РЅРѕ.
- `рџЊђ` Р Р°Р·РґРµР» СѓРїСЂР°РІР»РµРЅРёСЏ СЏР·С‹РєР°РјРё / РїРµСЂРµРІРѕРґР°РјРё РїСЂРёСЃСѓС‚СЃС‚РІСѓРµС‚ РІ РЅР°РІРёРіР°С†РёРё.
- `рџЊђ` UI РєРѕСЂСЂРµРєС‚РЅРѕ РїРµСЂРµРєР»СЋС‡Р°РµС‚СЃСЏ РјРµР¶РґСѓ СЏР·С‹РєР°РјРё (РјРёРЅРёРјСѓРј: RU, EN).

---

## 8. UI core РјРѕРґСѓР»РµР№ (РЅР°Р»РёС‡РёРµ Рё СЃР±РѕСЂРєР°)

> вљ пёЏ **Р’ СЂР°Р·СЂР°Р±РѕС‚РєРµ:** UI-РєРѕРјРїРѕРЅРµРЅС‚С‹ core РјРѕРґСѓР»РµР№ РЅР°С…РѕРґСЏС‚СЃСЏ РІ Р°РєС‚РёРІРЅРѕР№ СЂР°Р·СЂР°Р±РѕС‚РєРµ Рё РјРѕРіСѓС‚ С‡Р°СЃС‚РёС‡РЅРѕ РѕС‚СЃСѓС‚СЃС‚РІРѕРІР°С‚СЊ. Р­С‚Р° РїРѕРјРµС‚РєР° Р±СѓРґРµС‚ СЃРЅСЏС‚Р° РїРѕ РіРѕС‚РѕРІРЅРѕСЃС‚Рё UI.

> **РћР±Р»Р°СЃС‚СЊ РґРµР№СЃС‚РІРёСЏ:** Р­С‚РѕС‚ РїР»Р°РЅ РІРµСЂРёС„РёС†РёСЂСѓРµС‚ **С‚РѕР»СЊРєРѕ UI core РјРѕРґСѓР»РµР№** (rustok-auth, rustok-rbac, rustok-tenant, rustok-email, rustok-cache, rustok-core). UI РґРѕРјРµРЅРЅС‹С… РѕРїС†РёРѕРЅР°Р»СЊРЅС‹С… РјРѕРґСѓР»РµР№ Рё capability-СЃР»РѕС‘РІ (flex, rustok-mcp, alloy) РІРµСЂРёС„РёС†РёСЂСѓСЋС‚СЃСЏ РІ РїСЂРѕС„РёР»СЊРЅС‹С… РїР»Р°РЅР°С….

### 8.1 Leptos UI РєРѕРјРїРѕРЅРµРЅС‚С‹ core РјРѕРґСѓР»РµР№

- `рџ”§` `rustok-auth` вЂ” РЅР°Р»РёС‡РёРµ admin-UI Leptos (users, sessions, OAuth apps).
- `рџ”§` `rustok-rbac` вЂ” РЅР°Р»РёС‡РёРµ admin-UI Leptos (roles, permissions).
- `рџ”§` `rustok-tenant` вЂ” РЅР°Р»РёС‡РёРµ admin-UI Leptos (tenant management).
- `рџ”§` `rustok-email` вЂ” РЅР°Р»РёС‡РёРµ admin-UI Leptos (email settings).
- `рџ”§` `rustok-cache` вЂ” РЅР°Р»РёС‡РёРµ admin-UI Leptos (РµСЃР»Рё РїСЂРµРґСѓСЃРјРѕС‚СЂРµРЅ).
- `рџ”§` РЎР±РѕСЂРєР° РІСЃРµС… РЅР°Р№РґРµРЅРЅС‹С… Leptos UI РїР°РєРµС‚РѕРІ core РјРѕРґСѓР»РµР№ РїСЂРѕС…РѕРґРёС‚ (`cargo build`).

### 8.2 Next.js UI РїР°РєРµС‚С‹ core РјРѕРґСѓР»РµР№

- `рџ”§` РќР°Р»РёС‡РёРµ Next.js РїР°РєРµС‚РѕРІ РґР»СЏ СѓРїСЂР°РІР»РµРЅРёСЏ РїРѕР»СЊР·РѕРІР°С‚РµР»СЏРјРё/СЂРѕР»СЏРјРё/tenant-Р°РјРё РІ `apps/next-admin/packages/`.
- `рџ”§` РЎР±РѕСЂРєР° РїР°РєРµС‚РѕРІ РїСЂРѕС…РѕРґРёС‚ (`npm run build`).
- `рџ”§` Lint РїСЂРѕС…РѕРґРёС‚ (`npm run lint`).

### 8.3 РРЅС‚РµРіСЂР°С†РёСЏ UI РІ admin-РїР°РЅРµР»Рё

- `рџ”§` Leptos Admin СЂРµРіРёСЃС‚СЂРёСЂСѓРµС‚ UI core РјРѕРґСѓР»РµР№ С‡РµСЂРµР· module-owned routing.
- `рџ”§` Next.js Admin РёРјРїРѕСЂС‚РёСЂСѓРµС‚ РїР°РєРµС‚С‹ core РјРѕРґСѓР»РµР№ РєРѕСЂСЂРµРєС‚РЅРѕ Рё Р±РµР· С†РёРєР»РёС‡РµСЃРєРёС… Р·Р°РІРёСЃРёРјРѕСЃС‚РµР№.
- `рџ”§` РћС‚СЃСѓС‚СЃС‚РІСѓСЋС‰РёРµ (РІ СЂР°Р·СЂР°Р±РѕС‚РєРµ) UI РЅРµ Р±Р»РѕРєРёСЂСѓСЋС‚ СЃР±РѕСЂРєСѓ Рё Р·Р°РїСѓСЃРє admin-РїР°РЅРµР»РµР№.

---

## 9. GraphQL schema Р±РµР· РѕРїС†РёРѕРЅР°Р»СЊРЅС‹С… РјРѕРґСѓР»РµР№

**Р¤Р°Р№Р»С‹:**
- `apps/server/src/graphql/schema.rs`
- `apps/server/src/graphql/queries.rs`
- `apps/server/src/graphql/mutations.rs`

> РџРѕР»РЅС‹Р№ GraphQL-Р°СѓРґРёС‚ вЂ” РІ [platform-api-surfaces-verification-plan.md](./platform-api-surfaces-verification-plan.md).

- `рџ”§` GraphQL schema РєРѕРјРїРёР»РёСЂСѓРµС‚СЃСЏ Р±РµР· РїР°РЅРёРєРё РїСЂРё РѕС‚СЃСѓС‚СЃС‚РІРёРё domain resolver-РѕРІ.
- `рџЊђ` Queries РґР»СЏ auth, users, tenant-РѕРІ, settings СЂРµР·РѕР»РІСЏС‚СЃСЏ.
- `рџЊђ` Mutations РґР»СЏ СѓРїСЂР°РІР»РµРЅРёСЏ РїРѕР»СЊР·РѕРІР°С‚РµР»СЏРјРё, СЂРѕР»СЏРјРё, tenant-Р°РјРё СЂР°Р±РѕС‚Р°СЋС‚.

---

## 10. РљРѕРјР°РЅРґС‹

### 10.1 РЎР±РѕСЂРєР°

```sh
# Server
# РџСЂРёРјРµС‡Р°РЅРёРµ: РґРµС„РѕР»С‚РЅС‹Рµ features РІРєР»СЋС‡Р°СЋС‚ embed-admin (RustEmbed РёР· apps/admin/dist).
# Р•СЃР»Рё apps/admin/dist РЅРµ СЃРѕР±СЂР°РЅ, РёСЃРїРѕР»СЊР·СѓР№ РІР°СЂРёР°РЅС‚ Р±РµР· embed:
cargo build -p rustok-server --no-default-features \
  --features "redis-cache,mod-product,mod-pricing,mod-inventory,mod-cart,\
mod-customer,mod-order,mod-payment,mod-fulfillment,mod-commerce,mod-content,\
mod-blog,mod-forum,mod-pages,mod-alloy,mod-media,mod-workflow"
# Р›РёР±Рѕ РїСЂРµРґРІР°СЂРёС‚РµР»СЊРЅРѕ СЃРѕР±РµСЂРё Leptos admin: cd apps/admin && trunk build

# Leptos admin
cargo build -p rustok-admin

# Core crates workspace check
cargo check --workspace

# Next.js Admin
cd apps/next-admin && npm run build
cd apps/next-admin && npm run lint
# РџСЂРёРјРµС‡Р°РЅРёРµ: СЃРєСЂРёРїС‚ typecheck РІ apps/next-admin РЅРµ РѕРїСЂРµРґРµР»С‘РЅ; РёСЃРїРѕР»СЊР·СѓР№ tsc РЅР°РїСЂСЏРјСѓСЋ:
# cd apps/next-admin && npx tsc --noEmit
```

### 10.2 РўРµСЃС‚С‹ core

```sh
cargo test -p rustok-core --lib
cargo test -p rustok-auth --lib
cargo test -p rustok-rbac --lib
cargo test -p rustok-tenant --lib
cargo test -p rustok-outbox --lib
cargo test -p rustok-server --lib
```

### 10.3 РР·РѕР»СЏС†РёСЏ: РїРѕРёСЃРє РЅРµР¶РµР»Р°С‚РµР»СЊРЅС‹С… Р·Р°РІРёСЃРёРјРѕСЃС‚РµР№

```sh
git grep -rn "rustok-content\|rustok-commerce\|rustok-blog\|rustok-forum\|rustok-pages\|rustok-media\|rustok-workflow" \
  -- crates/rustok-core/ crates/rustok-auth/ crates/rustok-rbac/ \
     crates/rustok-tenant/ crates/rustok-events/ crates/rustok-outbox/ \
     crates/rustok-index/ crates/rustok-cache/ crates/rustok-email/
```

### 10.4 Health check

```sh
curl -f http://localhost:5150/api/health
```

### 10.5 Docker

```sh
docker compose config
docker compose up -d db
```

---

## 11. Stop-the-line СѓСЃР»РѕРІРёСЏ

РџСЂРё РѕР±РЅР°СЂСѓР¶РµРЅРёРё Р»СЋР±РѕРіРѕ РёР· РЅРёР¶РµРїРµСЂРµС‡РёСЃР»РµРЅРЅС‹С… СЃР»СѓС‡Р°РµРІ вЂ” **РѕСЃС‚Р°РЅРѕРІРёС‚СЊ РїСЂРѕРіРѕРЅ, РёСЃРїСЂР°РІРёС‚СЊ, РїРµСЂРµРїСЂРѕРІРµСЂРёС‚СЊ С„Р°Р·Сѓ, С‚РѕР»СЊРєРѕ Р·Р°С‚РµРј РїСЂРѕРґРѕР»Р¶Р°С‚СЊ**. РќРµ РѕСЃС‚Р°РІР»СЏС‚СЊ РІ Р°СЂС‚РµС„Р°РєС‚Рµ РєР°Рє В«known issueВ».

РЎС‡РёС‚Р°С‚СЊ Р±Р»РѕРєРёСЂСѓСЋС‰РёРј drift Р»СЋР±РѕР№ РёР· СЃР»РµРґСѓСЋС‰РёС… СЃР»СѓС‡Р°РµРІ:

- `cargo build -p rustok-server` РёР»Рё `cargo build -p rustok-admin` РЅРµ РєРѕРјРїРёР»РёСЂСѓСЋС‚СЃСЏ.
- Core crate РёРјРїРѕСЂС‚РёСЂСѓРµС‚ РѕРїС†РёРѕРЅР°Р»СЊРЅС‹Р№ РґРѕРјРµРЅРЅС‹Р№ crate (content, commerce, blog, forum, pages, media, workflow).
- Server РЅРµ СЃС‚Р°СЂС‚СѓРµС‚ РїСЂРё РІРєР»СЋС‡С‘РЅРЅС‹С… С‚РѕР»СЊРєРѕ core РјРѕРґСѓР»СЏС….
- `/api/health` РІРѕР·РІСЂР°С‰Р°РµС‚ РЅРµ 200 РїСЂРё С‡РёСЃС‚РѕРј СЃС‚Р°СЂС‚Рµ.
- GraphQL schema РїР°РЅРёРєСѓРµС‚ РїСЂРё СЃР±РѕСЂРєРµ Р±РµР· domain resolver-РѕРІ.
- Core РјРѕРґСѓР»СЊ СѓСЃРїРµС€РЅРѕ РѕС‚РєР»СЋС‡Р°РµС‚СЃСЏ С‡РµСЂРµР· tenant API (РѕР¶РёРґР°РµС‚СЃСЏ РѕС€РёР±РєР°).
- Р›СЋР±Р°СЏ admin-РїР°РЅРµР»СЊ РєСЂР°С€РёС‚СЃСЏ РїСЂРё РїРѕРїС‹С‚РєРµ РѕС‚РєСЂС‹С‚СЊ auth/dashboard СЃ С‚РѕР»СЊРєРѕ core.
- Р’ Р»СЋР±РѕР№ admin-РїР°РЅРµР»Рё РѕС‚СЃСѓС‚СЃС‚РІСѓРµС‚ РЅР°РІРёРіР°С†РёСЏ РїРѕ core С„СѓРЅРєС†РёСЏРј (auth, rbac, tenants, modules).

---

## 12. РђСЂС‚РµС„Р°РєС‚С‹

РљР°Р¶РґС‹Р№ РїСЂРѕРіРѕРЅ РґРѕР»Р¶РµРЅ РѕСЃС‚Р°РІР»СЏС‚СЊ РєРѕСЂРѕС‚РєРёР№ evidence bundle:

- РґР°С‚Р°
- branch / commit
- РІС‹РїРѕР»РЅРµРЅРЅС‹Рµ РєРѕРјР°РЅРґС‹
- pass/fail РїРѕ РєР°Р¶РґРѕР№ С„Р°Р·Рµ
- СЃРїРёСЃРѕРє UI-РєРѕРјРїРѕРЅРµРЅС‚РѕРІ core РјРѕРґСѓР»РµР№, РєРѕС‚РѕСЂС‹Рµ РѕС‚СЃСѓС‚СЃС‚РІСѓСЋС‚ (РІ СЂР°Р·СЂР°Р±РѕС‚РєРµ)
- СЃРїРёСЃРѕРє РІС‹СЏРІР»РµРЅРЅС‹С… РїСЂРѕР±Р»РµРј
- РѕСЃС‚Р°РІС€РёРµСЃСЏ Р±Р»РѕРєРµСЂС‹

**РњРµСЃС‚Рѕ С…СЂР°РЅРµРЅРёСЏ:** `artifacts/verification/platform-core-integrity/<yyyy-mm-dd>.md`

---

## РЎРІСЏР·Р°РЅРЅС‹Рµ РґРѕРєСѓРјРµРЅС‚С‹

- [Р“Р»Р°РІРЅС‹Р№ РїР»Р°РЅ РІРµСЂРёС„РёРєР°С†РёРё РїР»Р°С‚С„РѕСЂРјС‹](./PLATFORM_VERIFICATION_PLAN.md)
- [РџР»Р°РЅ foundation-РІРµСЂРёС„РёРєР°С†РёРё](./platform-foundation-verification-plan.md) вЂ” РїРѕР»РЅС‹Р№ auth/RBAC/tenancy/registry Р°СѓРґРёС‚
- [РџР»Р°РЅ РІРµСЂРёС„РёРєР°С†РёРё API-РїРѕРІРµСЂС…РЅРѕСЃС‚РµР№](./platform-api-surfaces-verification-plan.md) вЂ” РїРѕР»РЅС‹Р№ GraphQL/REST Р°СѓРґРёС‚
- [РџР»Р°РЅ rolling-РІРµСЂРёС„РёРєР°С†РёРё RBAC РґР»СЏ server Рё runtime-РјРѕРґСѓР»РµР№](./rbac-server-modules-verification-plan.md)
- [README РєР°С‚Р°Р»РѕРіР° verification](./README.md)

