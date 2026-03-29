<div align="center">

# <img src="assets/rustok-logo-512x512.png" width="72" align="center" /> RusToK

**РЎРѕР±С‹С‚РёР№РЅР°СЏ РјРѕРґСѓР»СЊРЅР°СЏ РїР»Р°С‚С„РѕСЂРјР° РЅР° Rust**

*РћРґРёРЅ СЂРµРїРѕР·РёС‚РѕСЂРёР№ РґР»СЏ СЃРµСЂРІРµСЂР°, РёРЅС‚РµРіСЂРёСЂРѕРІР°РЅРЅС‹С… Leptos host-РїСЂРёР»РѕР¶РµРЅРёР№ Рё headless/СЌРєСЃРїРµСЂРёРјРµРЅС‚Р°Р»СЊРЅС‹С… Next.js host-РїСЂРёР»РѕР¶РµРЅРёР№.*

[![CI](https://github.com/RustokCMS/RusToK/actions/workflows/ci.yml/badge.svg)](https://github.com/RustokCMS/RusToK/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

**[English version](README.md)** | **[РљСЂР°С‚РєР°СЏ СЃРїСЂР°РІРєР° РїРѕ РїР»Р°С‚С„РѕСЂРјРµ](PLATFORM_INFO_RU.md)**

</div>

RusToK СЃРµР№С‡Р°СЃ РїСЂРµРґСЃС‚Р°РІР»СЏРµС‚ СЃРѕР±РѕР№ Rust-first modular monolith РґР»СЏ РјСѓР»СЊС‚РёС‚РµРЅР°РЅС‚РЅС‹С… РїСЂРѕРґСѓРєС‚РѕРІ, РіРґРµ СЃРѕС‡РµС‚Р°СЋС‚СЃСЏ РєРѕРЅС‚РµРЅС‚, commerce, workflow Рё РёРЅС‚РµРіСЂР°С†РёРё. РўРµРєСѓС‰РёР№ С†РµРЅС‚СЂ РїР»Р°С‚С„РѕСЂРјС‹ вЂ” `apps/server` РєР°Рє composition root, СЃР±РѕСЂРєР° РјРѕРґСѓР»РµР№ С‡РµСЂРµР· manifest, СЃРѕР±С‹С‚РёР№РЅРѕРµ СЂР°Р·РґРµР»РµРЅРёРµ РїСѓС‚РµР№ Р·Р°РїРёСЃРё Рё С‡С‚РµРЅРёСЏ Рё РґРІРµ СЃС‚СЂР°С‚РµРіРёРё UI-host'РѕРІ: Leptos РєР°Рє РѕСЃРЅРѕРІРЅРѕР№ РёРЅС‚РµРіСЂРёСЂРѕРІР°РЅРЅС‹Р№ РїСѓС‚СЊ Рё Next.js РєР°Рє headless РёР»Рё СЌРєСЃРїРµСЂРёРјРµРЅС‚Р°Р»СЊРЅС‹Р№ РєРѕРЅС‚СѓСЂ.

<a id="table-of-contents"></a>

## РћРіР»Р°РІР»РµРЅРёРµ

- [РћР±Р·РѕСЂ](#overview)
- [Р’РѕР·РјРѕР¶РЅРѕСЃС‚Рё](#features)
- [РџСЂРѕРёР·РІРѕРґРёС‚РµР»СЊРЅРѕСЃС‚СЊ Рё СЌРєРѕРЅРѕРјРёСЏ](#performance-and-economy)
- [РџРѕС‡РµРјСѓ Rust](#why-rust)
- [AI-Native Architecture](#ai-native-architecture)
- [РЎСЂР°РІРЅРµРЅРёРµ](#comparison)
- [РЎРЅРёРјРѕРє Р°СЂС…РёС‚РµРєС‚СѓСЂС‹](#architecture-snapshot)
  - [РџСЂРёР»РѕР¶РµРЅРёСЏ](#applications)
  - [РўР°РєСЃРѕРЅРѕРјРёСЏ РјРѕРґСѓР»РµР№](#module-taxonomy)
- [РЎРёСЃС‚РµРјР° РјРѕРґСѓР»РµР№](#module-system)
- [Р‘С‹СЃС‚СЂС‹Р№ СЃС‚Р°СЂС‚](#quick-start)
- [Р”РѕРєСѓРјРµРЅС‚Р°С†РёСЏ](#documentation)
- [Р Р°Р·СЂР°Р±РѕС‚РєР°](#development)
- [РўРµРєСѓС‰РёР№ С„РѕРєСѓСЃ](#current-focus)
- [Р‘Р»Р°РіРѕРґР°СЂРЅРѕСЃС‚Рё](#acknowledgments)
- [Р›РёС†РµРЅР·РёСЏ](#license)

<a id="overview"></a>

## РћР±Р·РѕСЂ

РўРµРєСѓС‰РёРµ СЃРёР»СЊРЅС‹Рµ СЃС‚РѕСЂРѕРЅС‹ РїР»Р°С‚С„РѕСЂРјС‹:

- РЎР±РѕСЂРєР° С‡РµСЂРµР· manifest РѕС‚ [`modules.toml`](modules.toml) РґРѕ СЂР°РЅС‚Р°Р№РјР° Рё host-РїСЂРёР»РѕР¶РµРЅРёР№.
- РЇРІРЅС‹Рµ РіСЂР°РЅРёС†С‹ РјРµР¶РґСѓ `Core`, `Optional` Рё capability/support crates.
- Р“РёР±СЂРёРґРЅР°СЏ API-РјРѕРґРµР»СЊ: GraphQL РґР»СЏ РґРѕРјРµРЅРЅС‹С… РїРѕРІРµСЂС…РЅРѕСЃС‚РµР№, РѕСЂРёРµРЅС‚РёСЂРѕРІР°РЅРЅС‹С… РЅР° UI, REST РґР»СЏ operational Рё integration flows, WebSocket С‚Р°Рј, РіРґРµ РЅСѓР¶РµРЅ live runtime.
- РЎРѕР±С‹С‚РёР№РЅРѕРµ СЂР°Р·РґРµР»РµРЅРёРµ РїСѓС‚РµР№ Р·Р°РїРёСЃРё Рё С‡С‚РµРЅРёСЏ С‡РµСЂРµР· transactional outbox, `rustok-index` Рё `rustok-search`.
- Р”РІРµ СЃС‚СЂР°С‚РµРіРёРё UI-host'РѕРІ: `apps/admin` Рё `apps/storefront` РєР°Рє РёРЅС‚РµРіСЂРёСЂРѕРІР°РЅРЅС‹Рµ Leptos hosts, `apps/next-admin` Рё `apps/next-frontend` РєР°Рє headless/СЌРєСЃРїРµСЂРёРјРµРЅС‚Р°Р»СЊРЅС‹Рµ РєРѕРЅС‚СѓСЂС‹.

РљРѕСЂРЅРµРІРѕР№ README РЅР°РјРµСЂРµРЅРЅРѕ РєРѕСЂРѕС‚РєРёР№. Р•РіРѕ Р·Р°РґР°С‡Р° вЂ” РґР°С‚СЊ С‚РѕС‡РєСѓ РІС…РѕРґР° РІ СЂРµРїРѕР·РёС‚РѕСЂРёР№, Р° РЅРµ Р·Р°РјРµРЅРёС‚СЊ РїРѕР»РЅСѓСЋ Р°СЂС…РёС‚РµРєС‚СѓСЂРЅСѓСЋ СЃРїРµС†РёС„РёРєР°С†РёСЋ.

<a id="features"></a>

## Р’РѕР·РјРѕР¶РЅРѕСЃС‚Рё

### Core Platform

- РњСѓР»СЊС‚РёС‚РµРЅР°РЅС‚РЅС‹Р№ runtime СЃ tenant-aware РєРѕРЅС‚СЂР°РєС‚Р°РјРё
- Р“РёР±СЂРёРґРЅР°СЏ API-РјРѕРґРµР»СЊ СЃ GraphQL, REST Рё WebSocket С‚Р°Рј, РіРґРµ СЌС‚Рѕ РЅСѓР¶РЅРѕ
- Manifest-driven composition РјРѕРґСѓР»РµР№ Рё per-tenant enablement
- РЎРѕР±С‹С‚РёР№РЅРѕРµ СЂР°Р·РґРµР»РµРЅРёРµ РїСѓС‚РµР№ Р·Р°РїРёСЃРё Рё С‡С‚РµРЅРёСЏ С‡РµСЂРµР· transactional outbox
- Р’СЃС‚СЂРѕРµРЅРЅС‹Рµ РѕСЃРЅРѕРІС‹ РґР»СЏ Р»РѕРєР°Р»РёР·Р°С†РёРё, observability Рё RBAC

### Р РµР¶РёРјС‹ РґРµРїР»РѕСЏ

| Р РµР¶РёРј | РљР°Рє СЂР°Р±РѕС‚Р°РµС‚ | РђСѓС‚РµРЅС‚РёС„РёРєР°С†РёСЏ | РЎС†РµРЅР°СЂРёР№ |
|------|-------------|----------------|----------|
| **РњРѕРЅРѕР»РёС‚** | РЎРµСЂРІРµСЂ РїР»СЋСЃ РёРЅС‚РµРіСЂРёСЂРѕРІР°РЅРЅС‹Рµ Leptos admin/storefront hosts | РЎРµСЂРІРµСЂРЅС‹Рµ СЃРµСЃСЃРёРё Рё РѕР±С‰РёР№ runtime context | Self-hosted СЃР°Р№С‚, РІСЃС‚СЂРѕРµРЅРЅС‹Р№ backoffice Рё storefront |
| **Headless** | `apps/server` РѕС‚РґР°С‘С‚ API, Р° frontend Р¶РёРІС‘С‚ РѕС‚РґРµР»СЊРЅРѕ | OAuth2, sessions РёР»Рё СЃРјРµС€Р°РЅРЅС‹Р№ РєРѕРЅС‚СЂР°РєС‚ РІ Р·Р°РІРёСЃРёРјРѕСЃС‚Рё РѕС‚ РєР»РёРµРЅС‚Р° | РњРѕР±РёР»СЊРЅС‹Рµ РїСЂРёР»РѕР¶РµРЅРёСЏ, РІРЅРµС€РЅРёРµ С„СЂРѕРЅС‚РµРЅРґС‹, РёРЅС‚РµРіСЂР°С†РёРё |
| **РЎРјРµС€Р°РЅРЅС‹Р№** | РРЅС‚РµРіСЂРёСЂРѕРІР°РЅРЅС‹Рµ Leptos hosts Рё РІРЅРµС€РЅРёРµ РєР»РёРµРЅС‚С‹ РїРѕРІРµСЂС… РѕРґРЅРѕРіРѕ СЂР°РЅС‚Р°Р№РјР° | РћР±Р° | Р’СЃС‚СЂРѕРµРЅРЅР°СЏ Р°РґРјРёРЅРєР° РїР»СЋСЃ РІРЅРµС€РЅРёРµ РїСЂРёР»РѕР¶РµРЅРёСЏ Рё РёРЅС‚РµРіСЂР°С†РёРё |

### РњР°С‚СЂРёС†Р° РІРѕР·РјРѕР¶РЅРѕСЃС‚РµР№

| Р’РѕР·РјРѕР¶РЅРѕСЃС‚СЊ | WordPress | Shopify | Strapi | Ghost | **RusToK** |
|---|---|---|---|---|---|
| РњРѕРЅРѕР»РёС‚РЅС‹Р№ РґРµРїР»РѕР№ | РґР° | РЅРµС‚ | РЅРµС‚ | РґР° | **РґР°** |
| Headless API surface | С‡Р°СЃС‚РёС‡РЅРѕ | РґР° | РґР° | С‡Р°СЃС‚РёС‡РЅРѕ | **РґР°** |
| РЎРјРµС€Р°РЅРЅС‹Р№ integrated + headless СЂРµР¶РёРј | РєРѕСЃС‚С‹Р»Рё | С‡Р°СЃС‚РёС‡РЅРѕ | С‡Р°СЃС‚РёС‡РЅРѕ | РѕРіСЂР°РЅРёС‡РµРЅРЅРѕ | **РґР°** |
| РњСѓР»СЊС‚РёС‚РµРЅР°РЅС‚РЅС‹Р№ runtime | multisite | РѕРіСЂР°РЅРёС‡РµРЅРЅРѕ | РЅРµС‚ | РЅРµС‚ | **РЅР°С‚РёРІРЅРѕ** |
| Compile-time composition РјРѕРґСѓР»РµР№ | РЅРµС‚ | РЅРµС‚ | РЅРµС‚ | РЅРµС‚ | **РґР°** |
| Rust-first integrated UI path | РЅРµС‚ | РЅРµС‚ | РЅРµС‚ | РЅРµС‚ | **РґР°** |

### Developer Experience

- Loco.rs РєР°Рє foundation РґР»СЏ РѕР±С‰РµРіРѕ server runtime
- Rust crates РєР°Рє СЏРІРЅС‹Рµ РіСЂР°РЅРёС†С‹ РјРѕРґСѓР»РµР№
- Module-owned transport Рё UI slices РІРјРµСЃС‚Рѕ giant central app
- Р–РёРІР°СЏ РґРѕРєСѓРјРµРЅС‚Р°С†РёСЏ, РёРЅРґРµРєСЃРёСЂСѓРµРјР°СЏ РёР· `docs/index.md`

### РўРµСЃС‚РёСЂРѕРІР°РЅРёРµ Рё РєР°С‡РµСЃС‚РІРѕ

- Workspace-wide Rust test flow С‡РµСЂРµР· `cargo nextest`
- РџСЂРѕРІРµСЂРєРё manifest Рё dependency hygiene С‡РµСЂРµР· `cargo machete`
- РџР»Р°РЅС‹ РІРµСЂРёС„РёРєР°С†РёРё РїР»Р°С‚С„РѕСЂРјС‹ РґР»СЏ architecture, frontend Рё quality РєРѕРЅС‚СѓСЂРѕРІ

### РќР°Р±Р»СЋРґР°РµРјРѕСЃС‚СЊ Рё Р±РµР·РѕРїР°СЃРЅРѕСЃС‚СЊ

- Prometheus-style РјРµС‚СЂРёРєРё Рё tracing stack
- Typed RBAC Рё permission-aware runtime contracts
- Tenant-aware request context Рё channel-aware request flow
- РћР±С‰РёРµ validation Рё outbox/event-runtime guardrails

<a id="performance-and-economy"></a>

## РџСЂРѕРёР·РІРѕРґРёС‚РµР»СЊРЅРѕСЃС‚СЊ Рё СЌРєРѕРЅРѕРјРёСЏ

РўРѕС‡РЅС‹Рµ С‡РёСЃР»Р° Р·Р°РІРёСЃСЏС‚ РѕС‚ deployment profile Рё СЃРѕСЃС‚Р°РІР° РјРѕРґСѓР»РµР№, РЅРѕ РїРѕР·РёС†РёРѕРЅРёСЂРѕРІР°РЅРёРµ РїР»Р°С‚С„РѕСЂРјС‹ РїРѕ-РїСЂРµР¶РЅРµРјСѓ СЃС‚СЂРѕРёС‚СЃСЏ РІРѕРєСЂСѓРі СЌС„С„РµРєС‚РёРІРЅРѕСЃС‚Рё compiled runtime Рё РґРµРЅРѕСЂРјР°Р»РёР·РѕРІР°РЅРЅС‹С… read paths.

### Р‘РµРЅС‡РјР°СЂРєРё (СЃРёРјСѓР»РёСЂРѕРІР°РЅРЅС‹Рµ)

| РњРµС‚СЂРёРєРё | WordPress | Strapi | RusToK |
|---------|-----------|--------|--------|
| **Req/sec** | 60 | 800 | **45,000+** |
| **P99 Latency**| 450ms | 120ms | **8ms** |
| **Cold Boot** | N/A | 8.5s | **0.05s** |

<a id="why-rust"></a>

## РџРѕС‡РµРјСѓ Rust

### РџСЂРѕР±Р»РµРјС‹ СЃ С‚РµРєСѓС‰РёРјРё CMS-СЂРµС€РµРЅРёСЏРјРё

| РџСЂРѕР±Р»РµРјР° | WordPress | Node.js CMS | RusToK |
|----------|-----------|-------------|--------|
| **Runtime Errors** | Fatal errors РєСЂР°С€Р°С‚ СЃР°Р№С‚ | РќРµРѕС‚Р»РѕРІР»РµРЅРЅС‹Рµ РёСЃРєР»СЋС‡РµРЅРёСЏ | Р“Р°СЂР°РЅС‚РёРё РІСЂРµРјРµРЅРё РєРѕРјРїРёР»СЏС†РёРё |
| **Memory Leaks** | Р§Р°СЃС‚С‹Рµ СЃ РїР»Р°РіРёРЅР°РјРё | GC РїР°СѓР·С‹ Рё СЂР°Р·РґСѓРІР°РЅРёРµ РїР°РјСЏС‚Рё | РњРѕРґРµР»СЊ РІР»Р°РґРµРЅРёСЏ РїСЂРµРґРѕС‚РІСЂР°С‰Р°РµС‚ |
| **Р‘РµР·РѕРїР°СЃРЅРѕСЃС‚СЊ** | Р‘РѕР»СЊС€Р°СЏ РїРѕРІРµСЂС…РЅРѕСЃС‚СЊ Р°С‚Р°Рє С‡РµСЂРµР· РїР»Р°РіРёРЅС‹ | npm supply-chain СЂРёСЃРєРё | РЎРєРѕРјРїРёР»РёСЂРѕРІР°РЅРЅС‹Рµ Рё Р°СѓРґРёСЂСѓРµРјС‹Рµ Р·Р°РІРёСЃРёРјРѕСЃС‚Рё |
| **РњР°СЃС€С‚Р°Р±РёСЂРѕРІР°РЅРёРµ** | РўСЂРµР±СѓСЋС‚СЃСЏ РІРЅРµС€РЅРёРµ СЃР»РѕРё РєРµС€РёСЂРѕРІР°РЅРёСЏ | Р’ РѕСЃРЅРѕРІРЅРѕРј РіРѕСЂРёР·РѕРЅС‚Р°Р»СЊРЅРѕРµ | Р’РµСЂС‚РёРєР°Р»СЊРЅРѕРµ Рё РіРѕСЂРёР·РѕРЅС‚Р°Р»СЊРЅРѕРµ |

### РџСЂРµРёРјСѓС‰РµСЃС‚РІРѕ Rust

```rust
let product = Product::find_by_id(db, product_id)
    .await?
    .ok_or(Error::NotFound)?;
```

Р”Р°Р¶Рµ РїРѕСЃР»Рµ РїРµСЂРµСЂР°Р±РѕС‚РєРё Р°СЂС…РёС‚РµРєС‚СѓСЂС‹ Р±Р°Р·РѕРІР°СЏ С†РµРЅРЅРѕСЃС‚СЊ РѕСЃС‚Р°С‘С‚СЃСЏ С‚РѕР№ Р¶Рµ:

- Р±РѕР»СЊС€Рµ РѕС€РёР±РѕРє Р»РѕРІРёС‚СЃСЏ РЅР° СЌС‚Р°РїРµ РєРѕРјРїРёР»СЏС†РёРё;
- РґРѕРјРµРЅРЅС‹Рµ РєРѕРЅС‚СЂР°РєС‚С‹ РѕСЃС‚Р°СЋС‚СЃСЏ СЏРІРЅС‹РјРё РјРµР¶РґСѓ crate-Р°РјРё;
- runtime-РїРѕРІРµРґРµРЅРёРµ РїСЂРµРґСЃРєР°Р·СѓРµРјРѕ Рё РЅРµ Р·Р°РІРёСЃРёС‚ РѕС‚ РёРЅС‚РµСЂРїСЂРµС‚Р°С‚РѕСЂР°.

<a id="ai-native-architecture"></a>

## AI-Native Architecture

RusToK РїРѕ-РїСЂРµР¶РЅРµРјСѓ РѕСЂРёРµРЅС‚РёСЂРѕРІР°РЅ РЅР° agent-assisted СЂР°Р±РѕС‚Сѓ, РЅРѕ РІ РїСЂР°РєС‚РёС‡РµСЃРєРѕРј СЃРјС‹СЃР»Рµ: СЂРµРїРѕР·РёС‚РѕСЂРёР№ РѕРїРёСЂР°РµС‚СЃСЏ РЅР° СЏРІРЅС‹Рµ РєРѕРЅС‚СЂР°РєС‚С‹, РєР°СЂС‚Сѓ РґРѕРєСѓРјРµРЅС‚Р°С†РёРё, module manifests Рё РїСЂРµРґСЃРєР°Р·СѓРµРјС‹Рµ component boundaries, Р° РЅРµ РЅР° РјР°РіРёСЋ РіРµРЅРµСЂР°С‚РѕСЂРѕРІ.

РџСЂР°РєС‚РёС‡РµСЃРєРёРµ AI-facing С‚РѕС‡РєРё РІС…РѕРґР°:

- [РљР°СЂС‚Р° РґРѕРєСѓРјРµРЅС‚Р°С†РёРё](docs/index.md)
- [РЎРёСЃС‚РµРјРЅС‹Р№ РјР°РЅРёС„РµСЃС‚](RUSTOK_MANIFEST.md)
- [Р РµРµСЃС‚СЂ РјРѕРґСѓР»РµР№](docs/modules/registry.md)
- [РџСЂР°РІРёР»Р° Р°РіРµРЅС‚РѕРІ](AGENTS.md)

<a id="comparison"></a>

## РЎСЂР°РІРЅРµРЅРёРµ

### vs. WordPress + WooCommerce

| РђСЃРїРµРєС‚ | WordPress | RusToK |
|--------|-----------|--------|
| РЇР·С‹Рє | PHP 7.4+ | Rust |
| Plugin System | Runtime | Compile-time Рё manifest-driven |
| Type Safety | РќРµС‚ | РџРѕР»РЅР°СЏ |
| Multi-tenant | Multisite | РќР°С‚РёРІРЅС‹Р№ |
| API | REST | GraphQL + REST |
| Admin UI | PHP templates | Leptos host |

Р›СѓС‡С€Рµ РґР»СЏ: РєРѕРјР°РЅРґ, РєРѕС‚РѕСЂС‹Рј РЅСѓР¶РЅС‹ Р±РѕР»РµРµ СЃС‚СЂРѕРіРёРµ РєРѕРЅС‚СЂР°РєС‚С‹, С‡РµРј РґР°С‘С‚ plugin-first PHP stack.

### vs. Strapi (Node.js)

| РђСЃРїРµРєС‚ | Strapi | RusToK |
|--------|--------|--------|
| РЇР·С‹Рє | JavaScript/TypeScript | Rust |
| РњРѕРґРµР»РёСЂРѕРІР°РЅРёРµ РєРѕРЅС‚РµРЅС‚Р° | UI-based | Code- Рё module-based |
| Plugin Ecosystem | npm | crates Рё workspace modules |
| Cold Start | Р’С‹С€Рµ | РќРёР¶Рµ |

Р›СѓС‡С€Рµ РґР»СЏ: РєРѕРјР°РЅРґ, РєРѕС‚РѕСЂС‹Рј РЅСѓР¶РЅР° type safety Рё СЏРІРЅРѕРµ РІР»Р°РґРµРЅРёРµ РґРѕРјРµРЅР°РјРё.

### vs. Medusa.js (E-commerce)

| РђСЃРїРµРєС‚ | Medusa | RusToK |
|--------|--------|--------|
| Р¤РѕРєСѓСЃ | РўРѕР»СЊРєРѕ e-commerce | Commerce РїР»СЋСЃ content/community/workflow |
| РЇР·С‹Рє | TypeScript | Rust |
| РђСЂС…РёС‚РµРєС‚СѓСЂР° | Microservices encouraged | РњРѕРґСѓР»СЊРЅС‹Р№ РјРѕРЅРѕР»РёС‚ |
| Storefront | Next.js templates | Leptos host РїР»СЋСЃ Next.js companion paths |

Р›СѓС‡С€Рµ РґР»СЏ: РєРѕРјР°РЅРґ, РєРѕС‚РѕСЂС‹Рј РЅСѓР¶РЅС‹ commerce Рё non-commerce РґРѕРјРµРЅС‹ РІ РѕРґРЅРѕР№ РїР»Р°С‚С„РѕСЂРјРµ.

### vs. Directus / PayloadCMS

| РђСЃРїРµРєС‚ | Directus/Payload | RusToK |
|--------|------------------|--------|
| РџРѕРґС…РѕРґ | Database-first | Schema-first Рё module-first |
| Type Generation | Build step | РќР°С‚РёРІРЅС‹Рµ Rust types |
| Custom Logic | Hooks (JS) | Rust modules |
| Self-hosted | Р”Р° | Р”Р° |
| "Full Rust" | РќРµС‚ | Р”Р° |

Р›СѓС‡С€Рµ РґР»СЏ: РєРѕРјР°РЅРґ, СЃС‚СЂРѕСЏС‰РёС… РїР»Р°С‚С„РѕСЂРјСѓ РІРѕРєСЂСѓРі Rust-СЃС‚РµРєР°.

<a id="architecture-snapshot"></a>

## РЎРЅРёРјРѕРє Р°СЂС…РёС‚РµРєС‚СѓСЂС‹

<a id="applications"></a>

### РџСЂРёР»РѕР¶РµРЅРёСЏ

| РџСѓС‚СЊ | Р РѕР»СЊ |
|---|---|
| `apps/server` | Composition root, РѕР±С‰РёР№ HTTP/GraphQL runtime host, wiring auth/session/RBAC, event runtime, РїСЂРѕРІРµСЂРєР° manifest |
| `apps/admin` | РћСЃРЅРѕРІРЅРѕР№ Leptos admin host |
| `apps/storefront` | РћСЃРЅРѕРІРЅРѕР№ Leptos storefront host |
| `apps/next-admin` | Headless РёР»Рё СЌРєСЃРїРµСЂРёРјРµРЅС‚Р°Р»СЊРЅС‹Р№ Next.js admin host |
| `apps/next-frontend` | Headless РёР»Рё СЌРєСЃРїРµСЂРёРјРµРЅС‚Р°Р»СЊРЅС‹Р№ Next.js storefront host |

<a id="module-taxonomy"></a>

### РўР°РєСЃРѕРЅРѕРјРёСЏ РјРѕРґСѓР»РµР№

`modules.toml` вЂ” РёСЃС‚РѕС‡РЅРёРє РёСЃС‚РёРЅС‹ РїРѕ РјРѕРґСѓР»СЊРЅРѕРјСѓ СЃРѕСЃС‚Р°РІСѓ РїР»Р°С‚С„РѕСЂРјС‹.

Core-РјРѕРґСѓР»Рё:

- `auth`
- `cache`
- `channel`
- `email`
- `index`
- `search`
- `outbox`
- `tenant`
- `rbac`

Optional-РјРѕРґСѓР»Рё:

- РљРѕРЅС‚РµРЅС‚ Рё community: `content`, `blog`, `comments`, `forum`, `pages`, `media`, `workflow`
- Commerce family: `cart`, `customer`, `product`, `profiles`, `region`, `pricing`, `inventory`, `order`, `payment`, `fulfillment`, `commerce`

Р’СЃРїРѕРјРѕРіР°С‚РµР»СЊРЅС‹Рµ Рё capability-crates РЅР°С…РѕРґСЏС‚СЃСЏ РІРЅРµ С‚Р°РєСЃРѕРЅРѕРјРёРё `Core` / `Optional`:

- Shared/support: `rustok-core`, `rustok-api`, `rustok-events`, `rustok-storage`, `rustok-commerce-foundation`, `rustok-test-utils`, `rustok-telemetry`
- Capability/runtime layers: `rustok-mcp`, `alloy`, `alloy`, `flex`, `rustok-iggy`, `rustok-iggy-connector`

РљР»СЋС‡РµРІС‹Рµ РіСЂР°РЅРёС†С‹ РґРѕРјРµРЅРѕРІ:

- `rustok-content` С‚РµРїРµСЂСЊ shared helper Рё orchestration layer. Р­С‚Рѕ Р±РѕР»СЊС€Рµ РЅРµ product-facing storage РёР»Рё transport owner РґР»СЏ `blog`, `forum` Рё `pages`.
- `rustok-comments` вЂ” РѕС‚РґРµР»СЊРЅС‹Р№ generic comments module РґР»СЏ РєР»Р°СЃСЃРёС‡РµСЃРєРёС… РєРѕРјРјРµРЅС‚Р°СЂРёРµРІ РІРЅРµ forum domain.
- Commerce surface СЂР°Р·РґРµР»С‘РЅ РЅР° РїСЂРѕС„РёР»СЊРЅС‹Рµ family modules, Р° `rustok-commerce` СЂР°Р±РѕС‚Р°РµС‚ РєР°Рє umbrella/root module Рё orchestration layer.
- Channel-aware РїРѕРІРµРґРµРЅРёРµ СѓР¶Рµ РІС…РѕРґРёС‚ РІ live request/runtime pipeline С‡РµСЂРµР· `rustok-channel` Рё РѕР±С‰РёРµ request-context contracts.

<a id="module-system"></a>

## РЎРёСЃС‚РµРјР° РјРѕРґСѓР»РµР№

РўРµРєСѓС‰РёР№ РјРѕРґСѓР»СЊРЅС‹Р№ РїРѕС‚РѕРє СѓРїСЂР°РІР»СЏРµС‚СЃСЏ С‡РµСЂРµР· manifest:

```text
modules.toml
  -> build.rs РіРµРЅРµСЂРёСЂСѓРµС‚ wiring РґР»СЏ host-РїСЂРёР»РѕР¶РµРЅРёР№
  -> apps/server РїСЂРѕРІРµСЂСЏРµС‚ manifest
  -> ModuleRegistry / bootstrap СЂР°РЅС‚Р°Р№РјР°
  -> per-tenant enablement РґР»СЏ optional modules
```

Р’Р°Р¶РЅС‹Рµ РїСЂР°РІРёР»Р°:

- РќРµ СЃС‡РёС‚Р°С‚СЊ СЂСѓС‡РЅСѓСЋ СЂРµРіРёСЃС‚СЂР°С†РёСЋ РјР°СЂС€СЂСѓС‚РѕРІ РІ `app.rs` РѕСЃРЅРѕРІРЅС‹Рј СЃРїРѕСЃРѕР±РѕРј РёРЅС‚РµРіСЂР°С†РёРё РјРѕРґСѓР»РµР№.
- Host-РїСЂРёР»РѕР¶РµРЅРёСЏ РїРѕРґРєР»СЋС‡Р°СЋС‚ optional modules С‡РµСЂРµР· generated contracts, РїСЂРѕРёР·РІРѕРґРЅС‹Рµ РѕС‚ `modules.toml` Рё module manifests.
- Build composition Рё tenant enablement вЂ” СЂР°Р·РЅС‹Рµ СѓСЂРѕРІРЅРё:
  - build composition РѕРїСЂРµРґРµР»СЏРµС‚, С‡С‚Рѕ РїРѕРїР°РґР°РµС‚ РІ Р°СЂС‚РµС„Р°РєС‚;
  - tenant enablement РѕРїСЂРµРґРµР»СЏРµС‚, РєР°РєРёРµ optional modules Р°РєС‚РёРІРЅС‹ РґР»СЏ РєРѕРЅРєСЂРµС‚РЅРѕРіРѕ tenant.
- Leptos hosts СѓР¶Рµ РїРѕС‚СЂРµР±Р»СЏСЋС‚ module-owned UI packages С‡РµСЂРµР· manifest-driven wiring.
- Next.js hosts РѕСЃС‚Р°СЋС‚СЃСЏ manual/headless entry points Рё РЅРµ РґРѕР»Р¶РЅС‹ РѕРїРёСЃС‹РІР°С‚СЊСЃСЏ С‚Р°Рє, Р±СѓРґС‚Рѕ Сѓ РЅРёС… СѓР¶Рµ РµСЃС‚СЊ С‚РѕС‚ Р¶Рµ generated host contract.

РџРѕР»РЅР°СЏ РєР°СЂС‚Р° С‚РµРєСѓС‰РµРіРѕ runtime РѕРїРёСЃР°РЅР° РІ:

- [РћР±Р·РѕСЂРµ Р°СЂС…РёС‚РµРєС‚СѓСЂС‹](docs/architecture/overview.md)
- [Р РµРµСЃС‚СЂРµ РјРѕРґСѓР»РµР№](docs/modules/registry.md)
- [РРЅРґРµРєСЃРµ РјРѕРґСѓР»СЊРЅРѕР№ РґРѕРєСѓРјРµРЅС‚Р°С†РёРё](docs/modules/_index.md)
- [Р”РѕРєСѓРјРµРЅС‚Рµ РїСЂРѕ manifest Рё rebuild lifecycle](docs/modules/manifest.md)

<a id="quick-start"></a>

## Р‘С‹СЃС‚СЂС‹Р№ СЃС‚Р°СЂС‚

РђРєС‚СѓР°Р»СЊРЅРѕРµ СЂСѓРєРѕРІРѕРґСЃС‚РІРѕ Р±С‹СЃС‚СЂРѕРіРѕ СЃС‚Р°СЂС‚Р° РґР»СЏ Р»РѕРєР°Р»СЊРЅРѕР№ СЂР°Р·СЂР°Р±РѕС‚РєРё РЅР°С…РѕРґРёС‚СЃСЏ РІ [docs/guides/quickstart.md](docs/guides/quickstart.md).

РўРёРїРѕРІРѕР№ СЃС†РµРЅР°СЂРёР№:

```bash
./scripts/dev-start.sh
```

РўРµРєСѓС‰РёР№ guide РїРѕРєСЂС‹РІР°РµС‚ РїРѕР»РЅС‹Р№ Р»РѕРєР°Р»СЊРЅС‹Р№ СЃС‚РµРє:

- backend РЅР° `http://localhost:5150`
- Next.js admin РЅР° `http://localhost:3000`
- Leptos admin РЅР° `http://localhost:3001`
- Next.js storefront РЅР° `http://localhost:3100`
- Leptos storefront РЅР° `http://localhost:3101`

Р•СЃР»Рё РЅСѓР¶РµРЅ РЅРµ РєРѕСЂРЅРµРІРѕР№ РѕР±Р·РѕСЂ, Р° РєРѕРЅС‚РµРєСЃС‚ РєРѕРЅРєСЂРµС‚РЅРѕРіРѕ РїСЂРёР»РѕР¶РµРЅРёСЏ, РЅР°С‡РёРЅР°Р№С‚Рµ СЃ:

- [РґРѕРєСѓРјРµРЅС‚Р°С†РёРё apps/server](apps/server/docs/README.md)
- [РґРѕРєСѓРјРµРЅС‚Р°С†РёРё apps/admin](apps/admin/docs/README.md)
- [РґРѕРєСѓРјРµРЅС‚Р°С†РёРё apps/storefront](apps/storefront/docs/README.md)
- [РґРѕРєСѓРјРµРЅС‚Р°С†РёРё apps/next-admin](apps/next-admin/docs/README.md)
- [РґРѕРєСѓРјРµРЅС‚Р°С†РёРё apps/next-frontend](apps/next-frontend/docs/README.md)

<a id="documentation"></a>

## Р”РѕРєСѓРјРµРЅС‚Р°С†РёСЏ

РљР°РЅРѕРЅРёС‡РµСЃРєРёРµ С‚РѕС‡РєРё РІС…РѕРґР°:

- [РљР°СЂС‚Р° РґРѕРєСѓРјРµРЅС‚Р°С†РёРё](docs/index.md)
- [РћР±Р·РѕСЂ Р°СЂС…РёС‚РµРєС‚СѓСЂС‹](docs/architecture/overview.md)
- [Р РµРµСЃС‚СЂ РјРѕРґСѓР»РµР№ Рё РїСЂРёР»РѕР¶РµРЅРёР№](docs/modules/registry.md)
- [РРЅРґРµРєСЃ РјРѕРґСѓР»СЊРЅРѕР№ РґРѕРєСѓРјРµРЅС‚Р°С†РёРё](docs/modules/_index.md)
- [РЎРїСЂР°РІРѕС‡РЅС‹Р№ РїР°РєРµС‚ MCP](docs/references/mcp/README.md)
- [Р СѓРєРѕРІРѕРґСЃС‚РІРѕ РїРѕ С‚РµСЃС‚РёСЂРѕРІР°РЅРёСЋ](docs/guides/testing.md)
- [РџР»Р°РЅ СЂР°Р·РІРёС‚РёСЏ РјРѕРґСѓР»СЊРЅРѕР№ РїР»Р°С‚С„РѕСЂРјС‹](docs/modules/module-system-plan.md)
- [Р“Р»Р°РІРЅС‹Р№ РїР»Р°РЅ РІРµСЂРёС„РёРєР°С†РёРё РїР»Р°С‚С„РѕСЂРјС‹](docs/verification/PLATFORM_VERIFICATION_PLAN.md)
- [РЎРёСЃС‚РµРјРЅС‹Р№ РјР°РЅРёС„РµСЃС‚](RUSTOK_MANIFEST.md)
- [РџСЂР°РІРёР»Р° Р°РіРµРЅС‚РѕРІ](AGENTS.md)

<a id="development"></a>

## Р Р°Р·СЂР°Р±РѕС‚РєР°

Р РµРєРѕРјРµРЅРґСѓРµРјС‹Р№ РјРёРЅРёРјСѓРј РѕРєСЂСѓР¶РµРЅРёСЏ:

- Rust toolchain РёР· РєРѕРЅС„РёРіСѓСЂР°С†РёРё СЂРµРїРѕР·РёС‚РѕСЂРёСЏ
- PostgreSQL РґР»СЏ Р»РѕРєР°Р»СЊРЅРѕРіРѕ СЂР°РЅС‚Р°Р№РјР°
- Node.js РёР»Рё Bun РґР»СЏ Next.js host-РїСЂРёР»РѕР¶РµРЅРёР№
- `trunk` РґР»СЏ Leptos host-РїСЂРёР»РѕР¶РµРЅРёР№

РџРѕР»РµР·РЅС‹Рµ РєРѕРјР°РЅРґС‹:

```bash
# РїРѕР»РЅС‹Р№ Р»РѕРєР°Р»СЊРЅС‹Р№ СЃС‚РµРє
./scripts/dev-start.sh

# Rust tests
cargo nextest run --workspace --all-targets --all-features

# doc tests
cargo test --workspace --doc --all-features

# format Рё lint
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings

# dependency Рё policy checks
cargo deny check
cargo machete
```

РћР±С‰РёРµ РїСЂР°РІРёР»Р° РґР»СЏ РєРѕРЅС‚СЂРёР±СЊСЋС‚РѕСЂРѕРІ Рё Р°РіРµРЅС‚РѕРІ РѕРїРёСЃР°РЅС‹ РІ [CONTRIBUTING.md](CONTRIBUTING.md) Рё [AGENTS.md](AGENTS.md).

<a id="current-focus"></a>

## РўРµРєСѓС‰РёР№ С„РѕРєСѓСЃ

РђРєС‚СѓР°Р»СЊРЅС‹Рµ РїСЂРёРѕСЂРёС‚РµС‚С‹ РІРµРґСѓС‚СЃСЏ РІ Р¶РёРІС‹С… platform docs, Р° РЅРµ РІ РѕС‚РґРµР»СЊРЅРѕРј root roadmap-С„Р°Р№Р»Рµ:

- [РџР»Р°РЅ СЂР°Р·РІРёС‚РёСЏ РјРѕРґСѓР»СЊРЅРѕР№ РїР»Р°С‚С„РѕСЂРјС‹](docs/modules/module-system-plan.md)
- [Р“Р»Р°РІРЅС‹Р№ РїР»Р°РЅ РІРµСЂРёС„РёРєР°С†РёРё РїР»Р°С‚С„РѕСЂРјС‹](docs/verification/PLATFORM_VERIFICATION_PLAN.md)
- [РђСЂС…РёС‚РµРєС‚СѓСЂРЅС‹Рµ СЂРµС€РµРЅРёСЏ](DECISIONS/README.md)

Р’РµСЂС…РЅРµСѓСЂРѕРІРЅРµРІРѕ С‚РµРєСѓС‰РёР№ РєРѕРґРѕРІС‹Р№ С„РѕРєСѓСЃ С‚Р°РєРѕР№:

- РґРµСЂР¶Р°С‚СЊ С‡РµСЃС‚РЅС‹Рµ module boundaries РїРѕ РјРµСЂРµ СЂРѕСЃС‚Р° РїР»Р°С‚С„РѕСЂРјС‹;
- СЂР°Р·РІРёРІР°С‚СЊ module-owned transport Рё UI surfaces, РЅРµ РїСЂРµРІСЂР°С‰Р°СЏ `apps/server` РІ РґРѕРјРµРЅРЅСѓСЋ СЃРІР°Р»РєСѓ;
- СЃРѕС…СЂР°РЅСЏС‚СЊ manifest-driven composition РґР»СЏ server Рё Leptos hosts;
- СЃРёРЅС…СЂРѕРЅРёР·РёСЂРѕРІР°С‚СЊ channel-aware, multilingual Рё event-driven contracts РјРµР¶РґСѓ РґРѕРјРµРЅР°РјРё.

<a id="acknowledgments"></a>

## Р‘Р»Р°РіРѕРґР°СЂРЅРѕСЃС‚Рё

РџР»Р°С‚С„РѕСЂРјР° РѕРїРёСЂР°РµС‚СЃСЏ РЅР° С‚Р°РєРёРµ open-source РѕСЃРЅРѕРІС‹, РєР°Рє:

- Loco.rs
- Leptos
- SeaORM
- async-graphql
- Axum

<a id="license"></a>

## Р›РёС†РµРЅР·РёСЏ

RusToK СЂР°СЃРїСЂРѕСЃС‚СЂР°РЅСЏРµС‚СЃСЏ РїРѕ [Р»РёС†РµРЅР·РёРё MIT](LICENSE).

