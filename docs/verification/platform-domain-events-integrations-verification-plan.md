# РџР»Р°РЅ РІРµСЂРёС„РёРєР°С†РёРё РїР»Р°С‚С„РѕСЂРјС‹: СЃРѕР±С‹С‚РёСЏ, РґРѕРјРµРЅРЅС‹Рµ РјРѕРґСѓР»Рё Рё РёРЅС‚РµРіСЂР°С†РёРё

- **РЎС‚Р°С‚СѓСЃ:** РђРєС‚СѓР°Р»РёР·РёСЂРѕРІР°РЅРЅС‹Р№ РґРµС‚Р°Р»СЊРЅС‹Р№ С‡РµРєР»РёСЃС‚
- **РљРѕРЅС‚СѓСЂ:** Event flow, outbox/runtime transport, РґРѕРјРµРЅРЅС‹Рµ РјРѕРґСѓР»Рё, РјРѕРґСѓР»СЊРЅС‹Рµ Р·Р°РІРёСЃРёРјРѕСЃС‚Рё, РјРµР¶РєРѕРЅС‚СѓСЂРЅС‹Рµ РёРЅС‚РµРіСЂР°С†РёРё
- **РџСЂРёРјРµС‡Р°РЅРёРµ:** API- Рё UI-РїРѕРІРµСЂС…РЅРѕСЃС‚Рё РїСЂРѕРІРµСЂСЏСЋС‚СЃСЏ РІ РѕС‚РґРµР»СЊРЅС‹С… РїР»Р°РЅР°С…, РЅРѕ РёРЅС‚РµРіСЂР°С†РёРѕРЅРЅС‹Рµ СЃРєР»РµР№РєРё РѕСЃС‚Р°СЋС‚СЃСЏ Р·РґРµСЃСЊ.

---

## Р¤Р°Р·Р° 6: РЎРѕР±С‹С‚РёР№РЅР°СЏ СЃРёСЃС‚РµРјР°

### 6.1 Event runtime

**Р¤Р°Р№Р»С‹:**
- `apps/server/src/services/event_transport_factory.rs`
- `apps/server/src/services/event_bus.rs`
- `crates/rustok-outbox/`
- `crates/rustok-iggy/`

- [ ] РџРѕРґС‚РІРµСЂР¶РґРµРЅРѕ, С‡С‚Рѕ server bootstrap РїРѕРґРЅРёРјР°РµС‚ Р°РєС‚СѓР°Р»СЊРЅС‹Р№ event runtime.
- [ ] РџРѕРґС‚РІРµСЂР¶РґРµРЅРѕ, С‡С‚Рѕ РїРѕРґРґРµСЂР¶РёРІР°РµРјС‹Рµ transport modes СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓСЋС‚ РєРѕРґСѓ Рё settings.
- [ ] `outbox` РѕСЃС‚Р°С‘С‚СЃСЏ production-first transport path С‚Р°Рј, РіРґРµ СЌС‚Рѕ РѕР¶РёРґР°РµС‚СЃСЏ Р°СЂС…РёС‚РµРєС‚СѓСЂРѕР№.
- [ ] Iggy transport РЅРµ РґСЂРµР№С„СѓРµС‚ РѕС‚ С‚РµРєСѓС‰РёС… contracts Рё feature gates.

### 6.2 Transactional publish path

- [ ] Write-path РґРѕРјРµРЅРЅС‹С… СЃРµСЂРІРёСЃРѕРІ РїСѓР±Р»РёРєСѓРµС‚ СЃРѕР±С‹С‚РёСЏ С‡РµСЂРµР· transactional mechanism.
- [ ] РќРµС‚ РєСЂРёС‚РёС‡РЅС‹С… publish-after-commit СЃС†РµРЅР°СЂРёРµРІ РІ content/commerce/blog/forum/pages/workflow.
- [ ] Event envelope Рё retry metadata СЃРѕРІРїР°РґР°СЋС‚ СЃ С‚РµРєСѓС‰РёРј contract layer.

### 6.3 Read-side / consumers

- [ ] `rustok-index` Рё РґСЂСѓРіРёРµ consumers РїРѕРґРїРёСЃС‹РІР°СЋС‚СЃСЏ РЅР° Р°РєС‚СѓР°Р»СЊРЅС‹Рµ СЃРѕР±С‹С‚РёСЏ.
- [ ] Error handling РІ consumers РЅРµ Р»РѕРјР°РµС‚ batch/runtime loop.
- [ ] Backlog, retries Рё failed/DLQ semantics СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓСЋС‚ С‚РµРєСѓС‰РµРјСѓ РєРѕРґСѓ.

### 6.4 Coverage РґРѕРјРµРЅРЅС‹С… СЃРѕР±С‹С‚РёР№

- [ ] Content: creation/update/publish/archive/delete СЃС†РµРЅР°СЂРёРё РѕС‚СЂР°Р¶РµРЅС‹ РІ С‚РµРєСѓС‰РµРј event vocabulary.
- [ ] Commerce: product/variant/inventory/price СЃС†РµРЅР°СЂРёРё РѕС‚СЂР°Р¶РµРЅС‹ РІ С‚РµРєСѓС‰РµРј event vocabulary.
- [ ] Blog/Forum: wrapper-РјРѕРґСѓР»Рё РЅРµ С‚РµСЂСЏСЋС‚ РјРѕРґСѓР»СЊ-СЃРїРµС†РёС„РёС‡РЅС‹Рµ СЃРѕР±С‹С‚РёСЏ.
- [ ] Pages: РєРѕСЂСЂРµРєС‚РЅРѕ РёСЃРїРѕР»СЊР·СѓСЋС‚ content/node event path.
- [ ] Media Рё Workflow event surfaces РѕС‚СЂР°Р¶РµРЅС‹ С‚Р°Рј, РіРґРµ РѕРЅРё СѓР¶Рµ СЂРµР°Р»РёР·РѕРІР°РЅС‹ РІ РєРѕРґРµ.
- [ ] Р”Р»СЏ РЅРµ СЂРµР°Р»РёР·РѕРІР°РЅРЅС‹С… product areas РЅРµС‚ СѓСЃС‚Р°СЂРµРІС€РёС… РѕР±РµС‰Р°РЅРёР№ РІ РїР»Р°РЅРµ.

---

## Р¤Р°Р·Р° 7: Р”РѕРјРµРЅРЅС‹Рµ РјРѕРґСѓР»Рё

### 7.1 `rustok-content`

- [ ] Entities, DTOs, GraphQL/REST adapters Рё `NodeService` СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓСЋС‚ С‚РµРєСѓС‰РµРјСѓ РєРѕРґСѓ.
- [ ] State machine, translations, bodies Рё tenant scoping РѕС‚СЂР°Р¶РµРЅС‹ РєРѕСЂСЂРµРєС‚РЅРѕ.
- [ ] РњРёРіСЂР°С†РёРё С‡РµСЂРµР· `apps/server/migration` Рё/РёР»Рё shared migration path Р·Р°РґРѕРєСѓРјРµРЅС‚РёСЂРѕРІР°РЅС‹ С‡РµСЃС‚РЅРѕ.

### 7.2 `rustok-commerce`

- [ ] Product/variant/inventory/pricing surfaces СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓСЋС‚ С‚РµРєСѓС‰РµРјСѓ РЅР°Р±РѕСЂСѓ СЃРµСЂРІРёСЃРѕРІ.
- [ ] DTO validation Рё state machine checks РѕС‚СЂР°Р¶РµРЅС‹ Р±РµР· СѓСЃС‚Р°СЂРµРІС€РёС… РґРѕРїСѓС‰РµРЅРёР№.
- [ ] Order-related РѕР¶РёРґР°РЅРёСЏ РЅРµ РѕРїРµСЂРµР¶Р°СЋС‚ С„Р°РєС‚РёС‡РµСЃРєСѓСЋ СЂРµР°Р»РёР·Р°С†РёСЋ.

### 7.3 `rustok-blog`

- [ ] `BlogModule` РѕСЃС‚Р°С‘С‚СЃСЏ wrapper-РјРѕРґСѓР»РµРј РїРѕРІРµСЂС… content.
- [ ] Post/category/comment/tag surfaces СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓСЋС‚ С‚РµРєСѓС‰РµРјСѓ РєРѕРґСѓ.
- [ ] i18n, state machine Рё event publishing path Р°РєС‚СѓР°Р»СЊРЅС‹.

### 7.4 `rustok-forum`

- [ ] Topic/reply/category/moderation surfaces СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓСЋС‚ С‚РµРєСѓС‰РµРјСѓ РєРѕРґСѓ.
- [ ] Wrapper-Р»РѕРіРёРєР° РїРѕРІРµСЂС… content Р·Р°РґРѕРєСѓРјРµРЅС‚РёСЂРѕРІР°РЅР° РєРѕСЂСЂРµРєС‚РЅРѕ.
- [ ] Permissions Рё СЃРѕР±С‹С‚РёСЏ РЅРµ СЂР°СЃС…РѕРґСЏС‚СЃСЏ СЃ module contract.

### 7.5 `rustok-pages`

- [ ] `pages -> content` РѕС‚СЂР°Р¶РµРЅРѕ РєР°Рє runtime dependency.
- [ ] `PageService`, blocks, menus Рё node-backed persistence Р·Р°РґРѕРєСѓРјРµРЅС‚РёСЂРѕРІР°РЅС‹ РєРѕСЂСЂРµРєС‚РЅРѕ.
- [ ] Module-owned admin/storefront surfaces СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓСЋС‚ РєРѕРґСѓ.

### 7.6 `rustok-media`

- [ ] `MediaModule` РІРєР»СЋС‡С‘РЅ РІ optional modules Рё РѕС‚СЂР°Р¶С‘РЅ РІ РїР»Р°РЅРµ.
- [ ] Entities, DTOs, GraphQL surface Рё `MediaService` СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓСЋС‚ С‚РµРєСѓС‰РµРјСѓ РєРѕРґСѓ.
- [ ] Storage integration Рё localized metadata РѕС‚СЂР°Р¶РµРЅС‹ Р±РµР· СѓСЃС‚Р°СЂРµРІС€РёС… РїСЂРµРґРїРѕР»РѕР¶РµРЅРёР№.

### 7.7 `rustok-workflow`

- [ ] `WorkflowModule` РІРєР»СЋС‡С‘РЅ РІ optional modules Рё РѕС‚СЂР°Р¶С‘РЅ РІ РїР»Р°РЅРµ.
- [ ] Entities, GraphQL/REST surfaces, engine, trigger handler Рё built-in steps СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓСЋС‚ С‚РµРєСѓС‰РµРјСѓ РєРѕРґСѓ.
- [ ] Workflow РЅРµ РѕРїРёСЃР°РЅ РєР°Рє runtime dependency Alloy, РµСЃР»Рё РІ РєРѕРґРµ С‚Р°РєРѕР№ Р·Р°РІРёСЃРёРјРѕСЃС‚Рё РЅРµС‚.

### 7.8 `rustok-index`

- [ ] `IndexModule` Рё search/read-model contract СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓСЋС‚ С‚РµРєСѓС‰РµРјСѓ РєРѕРґСѓ.
- [ ] Content/Product indexers Рё search engine wiring Р·Р°РґРѕРєСѓРјРµРЅС‚РёСЂРѕРІР°РЅС‹ РєРѕСЂСЂРµРєС‚РЅРѕ.

### 7.9 `rustok-rbac` Рё `rustok-tenant`

- [ ] `rustok-rbac` РѕРїРёСЃР°РЅ РєР°Рє `Core` module СЃ relation/policy/runtime resolvers.
- [ ] `rustok-tenant` РѕРїРёСЃР°РЅ РєР°Рє `Core` module СЃ CRUD + tenant_modules lifecycle.
- [ ] Migration ownership РґР»СЏ СЌС‚РёС… РјРѕРґСѓР»РµР№ Р·Р°РґРѕРєСѓРјРµРЅС‚РёСЂРѕРІР°РЅ С‡РµСЃС‚РЅРѕ.

### 7.10 Alloy Рё РґСЂСѓРіРёРµ capability-crate'С‹

- [ ] `alloy` Рё `alloy` РЅРµ РѕРїРёСЃР°РЅС‹ РєР°Рє РѕР±С‹С‡РЅС‹Рµ tenant-toggle РґРѕРјРµРЅРЅС‹Рµ РјРѕРґСѓР»Рё.
- [ ] Capability boundaries Рё СЃРІСЏР·СЊ СЃ workflow/MCP РѕС‚СЂР°Р¶РµРЅС‹ Р±РµР· СЃРјРµС€РµРЅРёСЏ СЃ taxonomy platform modules.

---

## Р¤Р°Р·Р° 13: РРЅС‚РµРіСЂР°С†РёРѕРЅРЅС‹Рµ СЃРІСЏР·Рё

### 13.1 Module dependency contract

- [ ] Manifest dependencies Рё runtime dependencies СЃРѕРІРїР°РґР°СЋС‚.
- [ ] `blog/forum/pages -> content` РїСЂРѕРІРµСЂСЏСЋС‚СЃСЏ РєР°Рє build-time Рё runtime РёРЅРІР°СЂРёР°РЅС‚.
- [ ] Optional modules РЅРµ РѕР±С…РѕРґСЏС‚ dependency checks С‡РµСЂРµР· host-РїСЂРёР»РѕР¶РµРЅРёСЏ.

### 13.2 Write -> Event -> Read model

- [ ] Content/commerce/blog/forum/pages СЃС†РµРЅР°СЂРёРё РїСЂРѕС…РѕРґСЏС‚ РїСѓС‚СЊ write -> event -> index/read-side Р±РµР· СЂР°Р·СЂС‹РІР°.
- [ ] Workflow/event trigger path РЅРµ СЂР°СЃС…РѕРґРёС‚СЃСЏ СЃ С‚РµРєСѓС‰РёРј engine runtime.
- [ ] Build/event hub Рё GraphQL subscription path СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓСЋС‚ С‚РµРєСѓС‰РµРјСѓ server runtime.

### 13.3 Host apps Рё module-owned surfaces

- [ ] Leptos Admin РёСЃРїРѕР»СЊР·СѓРµС‚ module-owned admin pages С‡РµСЂРµР· `/modules/:module_slug` Рё `/*module_path`.
- [ ] Leptos Storefront РёСЃРїРѕР»СЊР·СѓРµС‚ module-owned page registrations Рё slot injections.
- [ ] Next.js/other hosts РЅРµ РґРѕРєСѓРјРµРЅС‚РёСЂРѕРІР°РЅС‹ РєР°Рє РїРѕС‚СЂРµР±РёС‚РµР»Рё С‚РµС… surfaces, РєРѕС‚РѕСЂС‹С… РІ РєРѕРґРµ РµС‰С‘ РЅРµС‚.

### 13.4 Build, manifest Рё lifecycle integration

- [ ] Manifest diff -> build request -> build progress path СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓРµС‚ С‚РµРєСѓС‰РёРј `BuildService` / GraphQL subscription / event hub РєРѕРЅС‚СЂР°РєС‚Р°Рј.
- [ ] Tenant module lifecycle Рё build pipeline РЅРµ РѕРїРёСЃР°РЅС‹ РєР°Рє РЅРµР·Р°РІРёСЃРёРјС‹Рµ, РµСЃР»Рё РІ РєРѕРґРµ РѕРЅРё СЃРІСЏР·Р°РЅС‹.

