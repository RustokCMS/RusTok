# РџР»Р°РЅ РІРµСЂРёС„РёРєР°С†РёРё РїР»Р°С‚С„РѕСЂРјС‹: РєР°С‡РµСЃС‚РІРѕ, СЌРєСЃРїР»СѓР°С‚Р°С†РёСЏ Рё release-readiness

- **РЎС‚Р°С‚СѓСЃ:** РђРєС‚СѓР°Р»РёР·РёСЂРѕРІР°РЅРЅС‹Р№ РґРµС‚Р°Р»СЊРЅС‹Р№ С‡РµРєР»РёСЃС‚
- **РљРѕРЅС‚СѓСЂ:** РўРµСЃС‚С‹, observability, РґРѕРєСѓРјРµРЅС‚Р°С†РёСЏ, CI/CD, Р±РµР·РѕРїР°СЃРЅРѕСЃС‚СЊ, РєР°С‡РµСЃС‚РІРѕ РєРѕРґР°, correctness

---

## Р¤Р°Р·Р° 14: РўРµСЃС‚РѕРІРѕРµ РїРѕРєСЂС‹С‚РёРµ

### 14.1 Workspace test strategy

- [ ] `cargo nextest run --workspace --all-targets --all-features` РїСЂРѕС…РѕРґРёС‚ РёР»Рё known-failures СЏРІРЅРѕ Р·Р°РґРѕРєСѓРјРµРЅС‚РёСЂРѕРІР°РЅС‹.
- [ ] `cargo test --workspace --lib` РїСЂРѕС…РѕРґРёС‚ РґР»СЏ library-level regression baseline.
- [ ] `cargo test --workspace --doc --all-features` РїСЂРѕС…РѕРґРёС‚ РґР»СЏ doc-test baseline.
- [ ] РќРµС‚ С„Р»РµР№РєРѕРІС‹С… С‚РµСЃС‚РѕРІ, Р·Р°РІРёСЃСЏС‰РёС… РѕС‚ РїРѕСЂСЏРґРєР° РІС‹РїРѕР»РЅРµРЅРёСЏ.

### 14.2 Server integration tests

**РџСѓС‚СЊ:** `apps/server/tests/`

- [ ] `commerce_openapi_contract.rs`
- [ ] `library_api_smoke.rs`
- [ ] `module_lifecycle.rs`
- [ ] `tenant_cache_stampede_test.rs`

### 14.3 Crate integration / contract tests

- [ ] `rustok-core/tests/*`
- [ ] `rustok-events/tests/*`
- [ ] `rustok-content/tests/*`
- [ ] `rustok-commerce/tests/*`
- [ ] `rustok-blog/tests/*`
- [ ] `rustok-forum/tests/*`
- [ ] `rustok-pages/tests/*`
- [ ] `rustok-index/tests/*`
- [ ] `rustok-rbac/tests/*`
- [ ] `rustok-tenant/tests/*`
- [ ] `rustok-outbox/tests/*`
- [ ] `rustok-telemetry/tests/*`
- [ ] `rustok-iggy/tests/*`
- [ ] `rustok-iggy-connector/tests/*`
- [ ] `rustok-mcp/tests/*`
- [ ] `rustok-test-utils/tests/*`

### 14.4 Property, security Рё benchmark РїСЂРѕРІРµСЂРєРё

- [ ] Property-based tests РїРѕ state-machine/validation СЃС†РµРЅР°СЂРёСЏРј РѕСЃС‚Р°СЋС‚СЃСЏ Р°РєС‚СѓР°Р»СЊРЅС‹РјРё.
- [ ] Security regression tests СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓСЋС‚ С‚РµРєСѓС‰РµРјСѓ auth/RBAC/input-validation contract.
- [ ] Bench suite РІ `benches/` РїРѕ-РїСЂРµР¶РЅРµРјСѓ СЃРѕР±РёСЂР°РµС‚СЃСЏ Рё СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓРµС‚ С‚РµРєСѓС‰РёРј hot paths.

---

## Р¤Р°Р·Р° 15: Observability Рё РѕРїРµСЂР°С†РёРѕРЅРЅР°СЏ РіРѕС‚РѕРІРЅРѕСЃС‚СЊ

### 15.1 Metrics, health, tracing

- [ ] `/metrics` РѕС‚СЂР°Р¶Р°РµС‚ С‚РµРєСѓС‰РёР№ РЅР°Р±РѕСЂ Prometheus metrics.
- [ ] `/health`, `/health/live`, `/health/ready`, `/health/runtime`, `/health/modules` РѕС‚СЂР°Р¶Р°СЋС‚ С‚РµРєСѓС‰РёР№ health contract.
- [ ] Tracing / OTEL wiring СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓРµС‚ С‚РµРєСѓС‰РµРјСѓ server bootstrap.
- [ ] GraphQL observability extension Рё build progress subscription РЅРµ СЂР°СЃС…РѕРґСЏС‚СЃСЏ СЃ runtime.

### 15.2 Grafana / Prometheus / Compose

- [ ] Datasource config РІ `grafana/` СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓРµС‚ С‚РµРєСѓС‰РµРјСѓ СЃС‚РµРєСѓ.
- [ ] Dashboards РЅРµ СЃСЃС‹Р»Р°СЋС‚СЃСЏ РЅР° РЅРµСЃСѓС‰РµСЃС‚РІСѓСЋС‰РёРµ datasource/uid.
- [ ] `docker-compose.yml`, `docker-compose.full-dev.yml`, `docker-compose.observability.yml` РѕС‚СЂР°Р¶Р°СЋС‚ С‚РµРєСѓС‰РёР№ dev/runtime stack.

### 15.3 Runtime operational paths

- [ ] Outbox backlog / retries / failed events РѕС‚СЂР°Р¶РµРЅС‹ РєР°Рє operational concern.
- [ ] MCP / OAuth / build pipeline operational endpoints РѕС‚СЂР°Р¶РµРЅС‹ РІ runbooks Рё verification notes.
- [ ] Cache invalidation, Redis optionality Рё degraded-mode scenarios РЅРµ СЂР°СЃС…РѕРґСЏС‚СЃСЏ СЃ РєРѕРґРѕРј.

---

## Р¤Р°Р·Р° 16: РЎРёРЅС…СЂРѕРЅРёР·Р°С†РёСЏ РґРѕРєСѓРјРµРЅС‚Р°С†РёРё СЃ РєРѕРґРѕРј

### 16.1 `docs/architecture/`

- [ ] `overview.md`
- [ ] `api.md`
- [ ] `database.md`
- [ ] `dataloader.md`
- [ ] `diagram.md`
- [ ] `event-flow-contract.md`
- [ ] `channels.md`
- [ ] `i18n.md`
- [ ] `matryoshka.md`
- [ ] `modules.md`
- [ ] `performance-baseline.md`
- [ ] `principles.md`
- [ ] `routing.md`

### 16.2 `docs/guides/`

- [ ] `quickstart.md`
- [ ] `observability-quickstart.md`
- [ ] `testing.md`
- [ ] `testing-integration.md`
- [ ] `testing-property.md`
- [ ] `testing-oauth2-rfc.md`
- [ ] `error-handling.md`
- [ ] `input-validation.md`
- [ ] `rate-limiting.md`
- [ ] `security-audit.md`
- [ ] `module-metrics.md`
- [ ] `metrics.md`
- [ ] `runtime-guardrails.md`
- [ ] `tracing.md`
- [ ] `scheduler.md`

### 16.3 `docs/modules/` Рё С†РµРЅС‚СЂР°Р»РёР·РѕРІР°РЅРЅС‹Рµ СЂРµРµСЃС‚СЂС‹

- [ ] `registry.md`
- [ ] `crates-registry.md`
- [ ] `manifest.md`
- [ ] `_index.md`
- [ ] module-system / UI package РґРѕРєСѓРјРµРЅС‚С‹ РЅРµ СЂР°СЃС…РѕРґСЏС‚СЃСЏ СЃ С‚РµРєСѓС‰РёРј РєРѕРґРѕРј

### 16.4 Р›РѕРєР°Р»СЊРЅР°СЏ РґРѕРєСѓРјРµРЅС‚Р°С†РёСЏ crate-РѕРІ Рё РїСЂРёР»РѕР¶РµРЅРёР№

- [ ] Shared library crates: `rustok-core`, `rustok-events`, `rustok-api`, `rustok-storage`, `rustok-test-utils`
- [ ] Core modules: `rustok-auth`, `rustok-cache`, `rustok-email`, `rustok-index`, `rustok-outbox`, `rustok-tenant`, `rustok-rbac`
- [ ] Optional modules: `rustok-content`, `rustok-commerce`, `rustok-blog`, `rustok-forum`, `rustok-pages`, `rustok-media`, `rustok-workflow`
- [ ] Capability / support crates: `rustok-telemetry`, `rustok-iggy`, `rustok-iggy-connector`, `alloy`, `alloy`, `rustok-mcp`, `flex`
- [ ] Apps: `apps/server`, `apps/admin`, `apps/storefront`, `apps/next-admin`, `apps/next-frontend`

### 16.5 Root Рё verification docs

- [ ] `README.md`
- [ ] `PLATFORM_INFO_RU.md`
- [ ] `RUSTOK_MANIFEST.md`
- [ ] `AGENTS.md`
- [ ] `CONTRIBUTING.md`
- [ ] `CHANGELOG.md`
- [ ] `docs/index.md`
- [ ] `docs/verification/README.md`
- [ ] master/detailed verification-РїР»Р°РЅС‹ СЃРѕРіР»Р°СЃРѕРІР°РЅС‹ РјРµР¶РґСѓ СЃРѕР±РѕР№

### 16.6 ADR

- [ ] Р’СЃРµ СЂРµС€РµРЅРёСЏ РІ `DECISIONS/` СЃРѕРіР»Р°СЃРѕРІР°РЅС‹ СЃ С‚РµРєСѓС‰РёРј РєРѕРґРѕРј Рё СЃС‚Р°С‚СѓСЃР°РјРё.
- [ ] ADR РїРѕ events / module bundles / auth lifecycle / Alloy / MCP / shared API layer РѕСЃС‚Р°СЋС‚СЃСЏ СЂРµР»РµРІР°РЅС‚РЅС‹РјРё.

---

## Р¤Р°Р·Р° 17: CI/CD Рё DevOps

### 17.1 GitHub workflows

- [ ] `.github/workflows/ci.yml` РѕС‚СЂР°Р¶Р°РµС‚ С‚РµРєСѓС‰РёРµ РѕР±СЏР·Р°С‚РµР»СЊРЅС‹Рµ РїСЂРѕРІРµСЂРєРё.
- [ ] `.github/workflows/dependencies.yml` РЅРµ РґСЂРµР№С„СѓРµС‚ РѕС‚ dependency-management СЃС‚СЂР°С‚РµРіРёРё.
- [ ] `cargo-nextest` РёСЃРїРѕР»СЊР·СѓРµС‚СЃСЏ РєР°Рє РѕСЃРЅРѕРІРЅРѕР№ Rust test runner РІ CI, Р° doc-tests РѕСЃС‚Р°СЋС‚СЃСЏ РѕС‚РґРµР»СЊРЅРѕР№ РїСЂРѕРІРµСЂРєРѕР№.
- [ ] `cargo-machete` РїРѕРґРєР»СЋС‡С‘РЅ РєР°Рє manifest-hygiene СЃРёРіРЅР°Р» Рё РЅРµ РєРѕРЅС„Р»РёРєС‚СѓРµС‚ СЃ `cargo udeps`.
- [ ] Verification docs РЅРµ РѕРїРёСЃС‹РІР°СЋС‚ С€Р°РіРё, РєРѕС‚РѕСЂС‹С… Р±РѕР»СЊС€Рµ РЅРµС‚ РІ CI.

### 17.2 Verify scripts Рё operational automation

- [ ] `scripts/verify/README.md` СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓРµС‚ С‚РµРєСѓС‰РµРјСѓ РЅР°Р±РѕСЂСѓ verification-РїР»Р°РЅРѕРІ.
- [ ] РЎРєСЂРёРїС‚С‹ РІ `scripts/verify/` РѕС‚СЂР°Р¶Р°СЋС‚ Р°РєС‚СѓР°Р»СЊРЅС‹Рµ checks Рё РЅРµ СЃСЃС‹Р»Р°СЋС‚СЃСЏ РЅР° СѓРґР°Р»С‘РЅРЅС‹Рµ РґРѕРєСѓРјРµРЅС‚С‹/РєРѕРјР°РЅРґС‹.
- [ ] Build/release helper scripts СЃРѕРіР»Р°СЃРѕРІР°РЅС‹ СЃ current manifest/build flow.

### 17.3 РџР»Р°РЅ РІРЅРµРґСЂРµРЅРёСЏ tooling-РїР°РєРµС‚Р° РєР°С‡РµСЃС‚РІР°

- [x] РЈСЃС‚Р°РЅРѕРІРёС‚СЊ Р»РѕРєР°Р»СЊРЅРѕ РёРЅСЃС‚СЂСѓРјРµРЅС‚С‹ С‚РµРєСѓС‰РµРіРѕ СЌС‚Р°РїР°: `cargo-nextest` Рё `cargo-machete`.
- [x] РСЃРїРѕР»СЊР·РѕРІР°С‚СЊ `cargo-nextest` РєР°Рє РѕСЃРЅРѕРІРЅРѕР№ Rust test runner РІ CI Рё РІ Р»РѕРєР°Р»СЊРЅРѕРј Р±Р°Р·РѕРІРѕРј workflow.
- [x] РћСЃС‚Р°РІРёС‚СЊ `cargo test --workspace --doc --all-features` РєР°Рє РѕС‚РґРµР»СЊРЅС‹Р№ doc-test baseline, Р° РЅРµ РїС‹С‚Р°С‚СЊСЃСЏ СЃРєСЂС‹С‚Рѕ Р·Р°РјРµРЅРёС‚СЊ РµРіРѕ.
- [x] РџРѕРґРєР»СЋС‡РёС‚СЊ `cargo-machete` РєР°Рє advisory manifest-hygiene check Р±РµР· Р·Р°РјРµРЅС‹ `cargo udeps`.
- [x] Р”Р»СЏ Windows-local `cargo nextest --workspace --all-targets --all-features` РІСЂРµРјРµРЅРЅРѕ РїРѕРґРјРµРЅРёС‚СЊ published `iggy_common 0.9.0` С‡РµСЂРµР· `patch.crates-io`, РїРѕС‚РѕРјСѓ С‡С‚Рѕ РІ РѕРїСѓР±Р»РёРєРѕРІР°РЅРЅРѕР№ crate РЅРµРІРµСЂРЅС‹Р№ `cfg` РІРѕРєСЂСѓРі `posix_fadvise`; РїРѕСЃР»Рµ РІС‹С…РѕРґР° upstream-СЂРµР»РёР·Р° СЃ С„РёРєСЃРѕРј РІРµСЂРЅСѓС‚СЊ Р·Р°РІРёСЃРёРјРѕСЃС‚СЊ РѕР±СЂР°С‚РЅРѕ РЅР° crates.io Рё СѓРґР°Р»РёС‚СЊ Р»РѕРєР°Р»СЊРЅС‹Р№ patch.
- [ ] РЎР»РµРґСѓСЋС‰РёР№ СЌС‚Р°Рї РІРЅРµРґСЂРµРЅРёСЏ: `Semgrep`, `GraphQL Inspector`, `knip`.
- [ ] РЎР»РµРґСѓСЋС‰РёР№ РїРѕСЃР»Рµ РЅРµРіРѕ СЌС‚Р°Рї: `Playwright` + `axe-core`, `GraphQL Code Generator`.
- [ ] РћС‚Р»РѕР¶РµРЅРЅС‹Р№ СЌС‚Р°Рї: `testcontainers`, `cargo-mutants`, `Stryker`.
- [ ] РРЅСЃС‚СЂСѓРјРµРЅС‚С‹ Р»РѕРєР°Р»СЊРЅРѕР№ РґРёР°РіРЅРѕСЃС‚РёРєРё Рё performance-Р°РЅР°Р»РёР·Р° РґРµСЂР¶Р°С‚СЊ РІРЅРµ blocking CI gate: `tokio-console`, `cargo-flamegraph`.
- [ ] РўРѕС‡РµС‡РЅС‹Рµ deep-check РёРЅСЃС‚СЂСѓРјРµРЅС‚С‹ РѕСЃС‚Р°РІР»СЏС‚СЊ РЅР° РїРѕР·РґРЅРёР№ Р°РґСЂРµСЃРЅС‹Р№ rollout РґР»СЏ РєСЂРёС‚РёС‡РЅС‹С… РјРѕРґСѓР»РµР№: `loom`, `miri`, `cargo-fuzz`.
- [ ] РќРµ Р·Р°РјРµРЅСЏС‚СЊ РЅР° СЌС‚РѕРј СЌС‚Р°РїРµ СЃСѓС‰РµСЃС‚РІСѓСЋС‰РёРµ `cargo audit`, `cargo deny`, `cargo udeps`, `cargo llvm-cov`, `scripts/verify/*` Рё `Makefile`; РЅРѕРІС‹Рµ РёРЅСЃС‚СЂСѓРјРµРЅС‚С‹ РґРѕР»Р¶РЅС‹ СѓСЃРёР»РёРІР°С‚СЊ С‚РµРєСѓС‰РёР№ baseline, Р° РЅРµ РґСѓР±Р»РёСЂРѕРІР°С‚СЊ РµРіРѕ Р±РµР· РґРѕРєР°Р·Р°РЅРЅРѕР№ РїРѕР»СЊР·С‹.

---

## Р¤Р°Р·Р° 18: Р‘РµР·РѕРїР°СЃРЅРѕСЃС‚СЊ

### 18.1 Auth Рё session security

- [ ] Rate limiting РЅР° auth endpoints СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓРµС‚ С‚РµРєСѓС‰РµРјСѓ middleware.
- [ ] Session revocation, password reset, verification Рё invite flows СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓСЋС‚ С‚РµРєСѓС‰РµРјСѓ РєРѕРґСѓ.
- [ ] OAuth browser/session flow РѕС‚СЂР°Р¶С‘РЅ РєРѕСЂСЂРµРєС‚РЅРѕ.

### 18.2 Authorization security

- [ ] RBAC coverage СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓРµС‚ С‚РµРєСѓС‰РёРј REST/GraphQL/capability surfaces.
- [ ] РќРµС‚ role-based bypass РІРјРµСЃС‚Рѕ typed permissions.
- [ ] Flex / MCP / scripts / workflows РЅРµ РѕР±С…РѕРґСЏС‚ РѕР±С‰РёР№ authorization model.

### 18.3 Tenant boundary security

- [ ] Tenant isolation РІ data access, cache Рё module lifecycle СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓРµС‚ С‚РµРєСѓС‰РµРјСѓ РєРѕРґСѓ.
- [ ] РќРµС‚ public/tenant-crossing endpoints, РЅРµ РѕС‚СЂР°Р¶С‘РЅРЅС‹С… РІ РґРѕРєСѓРјРµРЅС‚Р°С†РёРё.

### 18.4 Input, dependency Рё secret hygiene

- [ ] Validation flows СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓСЋС‚ С‚РµРєСѓС‰РёРј DTO/services.
- [ ] Secret handling Рё env contract РѕС‚СЂР°Р¶РµРЅС‹ С‡РµСЃС‚РЅРѕ.
- [ ] Dependency risk checks СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓСЋС‚ С‚РµРєСѓС‰РµРјСѓ toolchain/CI practice.

---

## Р¤Р°Р·Р° 19: РђРЅС‚РёРїР°С‚С‚РµСЂРЅС‹ Рё РєР°С‡РµСЃС‚РІРѕ РєРѕРґР°

### 19.1 Authorization / module / event antipatterns

- [ ] РќРµС‚ hardcoded role checks РІ server authorization path.
- [ ] РќРµС‚ publish-after-commit РІ РґРѕРјРµРЅРЅС‹С… write-path.
- [ ] Runtime modules РїСѓР±Р»РёРєСѓСЋС‚ permissions/dependencies С‡РµСЂРµР· module contracts.
- [ ] Host apps РЅРµ СЃРѕРґРµСЂР¶Р°С‚ РґРѕР»РіРѕРІСЂРµРјРµРЅРЅС‹Рµ bypass-СЂРµР°Р»РёР·Р°С†РёРё РІРјРµСЃС‚Рѕ library/module contracts.

### 19.2 API / DTO / service quality

- [ ] DTO validation Рё error mapping СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓСЋС‚ current service layer.
- [ ] GraphQL/REST parity РЅРµ Р·Р°РґРѕРєСѓРјРµРЅС‚РёСЂРѕРІР°РЅР° С‚Р°Рј, РіРґРµ РµС‘ С„Р°РєС‚РёС‡РµСЃРєРё РЅРµС‚.
- [ ] Capability crates РЅРµ СЃРјРµС€Р°РЅС‹ СЃ platform modules РІ РґРѕРєСѓРјРµРЅС‚Р°С†РёРё Рё РєРѕРґРµ.

### 19.3 Frontend / library quality

- [ ] Module-owned UI surfaces РЅРµ РґСѓР±Р»РёСЂСѓСЋС‚СЃСЏ Р±РµСЃСЃРёСЃС‚РµРјРЅРѕ РІ host apps.
- [ ] Leptos/Next/shared packages РґРѕРєСѓРјРµРЅС‚РёСЂРѕРІР°РЅС‹ РїРѕ С„Р°РєС‚РёС‡РµСЃРєРѕРјСѓ public contract.

---

## Р¤Р°Р·Р° 20: РџСЂР°РІРёР»СЊРЅРѕСЃС‚СЊ РєРѕРґР° Рё correctness

### 20.1 Type / serialization / migration correctness

- [ ] Type model Рё permission vocabulary СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓСЋС‚ С‚РµРєСѓС‰РµРјСѓ РєРѕРґСѓ.
- [ ] Serialization contracts РґР»СЏ REST/GraphQL/OAuth/MCP РЅРµ СЂР°СЃС…РѕРґСЏС‚СЃСЏ СЃ С‚РµРєСѓС‰РёРјРё DTOs.
- [ ] РњРёРіСЂР°С†РёРё СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓСЋС‚ С‚РµРєСѓС‰РёРј entity/service РѕР¶РёРґР°РЅРёСЏРј.

### 20.2 Concurrency / retry / runtime correctness

- [ ] Retry/backoff logic СЃРѕРѕС‚РІРµС‚СЃС‚РІСѓРµС‚ С‚РµРєСѓС‰РёРј async runtime patterns.
- [ ] Event consumers Рё schedulers РЅРµ СЃРѕРґРµСЂР¶Р°С‚ РѕС‡РµРІРёРґРЅС‹С… busy-loop / lag-handling regressions.
- [ ] Workflow, outbox, cache invalidation Рё build progress flows РЅРµ СЂР°СЃС…РѕРґСЏС‚СЃСЏ СЃ runtime behavior.

### 20.3 Boundary correctness

- [ ] Module registry, manifest, host apps, shared libraries Рё capability layers СЃРѕРіР»Р°СЃРѕРІР°РЅС‹ РєР°Рє РµРґРёРЅР°СЏ СЃРёСЃС‚РµРјР°.
- [ ] Alloy/MCP/workflow/flex boundaries Р·Р°РґРѕРєСѓРјРµРЅС‚РёСЂРѕРІР°РЅС‹ Рё СЂРµР°Р»РёР·РѕРІР°РЅС‹ Р±РµР· Р°СЂС…РёС‚РµРєС‚СѓСЂРЅРѕР№ РїСѓС‚Р°РЅРёС†С‹.

