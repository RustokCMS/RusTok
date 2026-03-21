# Р”РѕРєСѓРјРµРЅС‚Р°С†РёСЏ РїРѕ РјРѕРґСѓР»СЏРј RusToK

Р­С‚РѕС‚ РґРѕРєСѓРјРµРЅС‚ С„РёРєСЃРёСЂСѓРµС‚ С‚РµРєСѓС‰РµРµ СЃРѕСЃС‚РѕСЏРЅРёРµ РјРѕРґСѓР»СЊРЅРѕР№ Р°СЂС…РёС‚РµРєС‚СѓСЂС‹ РІ СЂРµРїРѕР·РёС‚РѕСЂРёРё:

- РєР°РєРёРµ runtime-РјРѕРґСѓР»Рё СЂРµР°Р»СЊРЅРѕ СЂРµРіРёСЃС‚СЂРёСЂСѓСЋС‚СЃСЏ РІ `ModuleRegistry`;
- РєР°РєРёРµ core/crate-Р·Р°РІРёСЃРёРјРѕСЃС‚Рё РѕР±СЏР·Р°С‚РµР»СЊРЅС‹ РґР»СЏ РїР»Р°С‚С„РѕСЂРјС‹;
- РєР°РєРёРµ capability-СЃР»РѕРё РЅРµ СЏРІР»СЏСЋС‚СЃСЏ tenant-toggle РјРѕРґСѓР»СЏРјРё.

## 1. РћР±С‰Р°СЏ РєР°СЂС‚РёРЅР°

RusToK вЂ” РјРѕРґСѓР»СЊРЅС‹Р№ РјРѕРЅРѕР»РёС‚: runtime-РјРѕРґСѓР»Рё РєРѕРјРїРёР»РёСЂСѓСЋС‚СЃСЏ РІ РѕР±С‰РёР№ Р±РёРЅР°СЂРЅРёРє Рё РїРѕРґРЅРёРјР°СЋС‚СЃСЏ С‡РµСЂРµР·
`ModuleRegistry`, Р° platform/core С„СѓРЅРєС†РёРѕРЅР°Р»СЊРЅРѕСЃС‚СЊ Рё capability-СЃР»РѕРё Р¶РёРІСѓС‚ СЂСЏРґРѕРј РІ shared crate'Р°С….

Р“РґРµ СЃРјРѕС‚СЂРµС‚СЊ РІ РєРѕРґРµ:

- runtime-СЂРµРіРёСЃС‚СЂР°С†РёСЏ РјРѕРґСѓР»РµР№: `apps/server/src/modules/mod.rs`
- СЃРёРЅС…СЂРѕРЅРёР·Р°С†РёСЏ runtime registry Рё manifest: `apps/server/src/modules/manifest.rs`
- РєРѕРЅС‚СЂР°РєС‚ РјРѕРґСѓР»СЏ Рё РІРёРґС‹ РјРѕРґСѓР»РµР№: `crates/rustok-core/src/module.rs`
- manifest СЃР±РѕСЂРєРё: `modules.toml`

## 2. Р§С‚Рѕ СЂРµР°Р»СЊРЅРѕ Р·Р°СЂРµРіРёСЃС‚СЂРёСЂРѕРІР°РЅРѕ РІ СЃРµСЂРІРµСЂРµ

### Core runtime-РјРѕРґСѓР»Рё

| Slug | Crate | РќР°Р·РЅР°С‡РµРЅРёРµ |
|---|---|---|
| `index` | `rustok-index` | РРЅРґРµРєСЃР°С†РёСЏ Рё read-model contracts |
| `tenant` | `rustok-tenant` | Tenant lifecycle Рё tenant metadata |
| `rbac` | `rustok-rbac` | RBAC lifecycle Рё authorization contracts |

### Optional runtime-РјРѕРґСѓР»Рё

| Slug | Crate | РќР°Р·РЅР°С‡РµРЅРёРµ |
|---|---|---|
| `content` | `rustok-content` | РљРѕРЅС‚РµРЅС‚РЅС‹Р№ РґРѕРјРµРЅ |
| `commerce` | `rustok-commerce` | Commerce/catalog/inventory |
| `blog` | `rustok-blog` | Р‘Р»РѕРі РїРѕРІРµСЂС… content |
| `forum` | `rustok-forum` | Р¤РѕСЂСѓРј РїРѕРІРµСЂС… content |
| `pages` | `rustok-pages` | РЎС‚СЂР°РЅРёС†С‹, РјРµРЅСЋ Рё Р±Р»РѕРєРё |
| `workflow` | `rustok-workflow` | Workflow automation Рё execution history |

## 3. РћР±СЏР·Р°С‚РµР»СЊРЅС‹Р№ platform/core СЃР»РѕР№

Р­С‚Рё crate'С‹ СЏРІР»СЏСЋС‚СЃСЏ С‡Р°СЃС‚СЊСЋ РѕР±СЏР·Р°С‚РµР»СЊРЅРѕРіРѕ Р±Р°Р·РёСЃР° РїР»Р°С‚С„РѕСЂРјС‹:

| Crate | Р РѕР»СЊ |
|---|---|
| `rustok-core` | Р‘Р°Р·РѕРІС‹Рµ РїР»Р°С‚С„РѕСЂРјРµРЅРЅС‹Рµ РєРѕРЅС‚СЂР°РєС‚С‹ |
| `rustok-api` | РћР±С‰РёР№ web/API СЃР»РѕР№ РґР»СЏ transport-Р°РґР°РїС‚РµСЂРѕРІ |
| `rustok-outbox` | Transactional delivery СЃРѕР±С‹С‚РёР№ |
| `rustok-events` | РљР°РЅРѕРЅРёС‡РµСЃРєРёР№ import point РґР»СЏ event contracts |
| `rustok-telemetry` | Observability bootstrap |
| `rustok-cache` | Cache/runtime infra |
| `rustok-storage` | Storage backend contracts |
| `rustok-iggy` + `rustok-iggy-connector` | Streaming transport |
| `rustok-mcp` | MCP integration surface |

## 4. Alloy: РЅРµ РјРѕРґСѓР»СЊ, Р° capability

Alloy Р±РѕР»СЊС€Рµ РЅРµ С‚СЂР°РєС‚СѓРµС‚СЃСЏ РєР°Рє optional runtime-РјРѕРґСѓР»СЊ.

РџСЂР°РІРёР»СЊРЅР°СЏ Р°СЂС…РёС‚РµРєС‚СѓСЂРЅР°СЏ РјРѕРґРµР»СЊ:

- `alloy-scripting` вЂ” module-agnostic runtime/engine crate;
- `alloy` вЂ” transport-shell для Alloy management/API surface;
- Alloy РЅРµ СЂРµРіРёСЃС‚СЂРёСЂСѓРµС‚СЃСЏ РІ `ModuleRegistry`;
- Alloy РЅРµ СѓС‡Р°СЃС‚РІСѓРµС‚ РІ tenant module lifecycle;
- `workflow` РјРѕР¶РµС‚ РёСЃРїРѕР»СЊР·РѕРІР°С‚СЊ Alloy РєР°Рє capability РґР»СЏ С€Р°РіР° `alloy_script`, РЅРѕ РЅРµ РєР°Рє runtime dependency;
- РІРЅРµС€РЅСЏСЏ РёРЅС‚РµРіСЂР°С†РёРѕРЅРЅР°СЏ РїРѕРІРµСЂС…РЅРѕСЃС‚СЊ Alloy РЅР°С…РѕРґРёС‚СЃСЏ СЂСЏРґРѕРј СЃ `rustok-mcp`.

## 5. UI composition policy РґР»СЏ optional-РјРѕРґСѓР»РµР№

Р”Р»СЏ `ModuleKind::Optional` UI-СЃР»РѕР№ РЅРµ РґРѕР»Р¶РµРЅ Р¶РёС‚СЊ РІ `apps/*` РєР°Рє Р¶С‘СЃС‚РєРѕ РїСЂРёС€РёС‚Р°СЏ Р»РѕРіРёРєР°.
Р­РєСЂР°РЅС‹, РјРµРЅСЋ Рё РјР°СЂС€СЂСѓС‚С‹ РґРѕР»Р¶РЅС‹ РїРѕСЃС‚Р°РІР»СЏС‚СЊСЃСЏ СЃР°РјРёРјРё РјРѕРґСѓР»СЊРЅС‹РјРё РїР°РєРµС‚Р°РјРё/crate'Р°РјРё.

РСЃРєР»СЋС‡РµРЅРёРµ: platform/core Рё capability-СЃР»РѕРё РЅРµ РѕР±СЏР·Р°РЅС‹ СЃР»РµРґРѕРІР°С‚СЊ СЌС‚РѕРјСѓ РїСЂР°РІРёР»Сѓ, РµСЃР»Рё РїРѕ СЃРІРѕРµР№
РїСЂРёСЂРѕРґРµ РѕРЅРё СЏРІР»СЏСЋС‚СЃСЏ server/runtime orchestration, Р° РЅРµ tenant-РјРѕРґСѓР»РµРј.

