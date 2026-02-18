use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_auth::hooks::{use_auth, use_current_user, use_tenant, use_token};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::api::queries::DASHBOARD_STATS_QUERY;
use crate::api::request;
use crate::components::ui::{Button, LanguageToggle, PageHeader, StatsCard};
use crate::modules::{components_for_slot, AdminSlot};
use crate::providers::locale::translate;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct DashboardStatsResponse {
    #[serde(rename = "dashboardStats")]
    dashboard_stats: Option<DashboardStats>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct DashboardStats {
    #[serde(rename = "totalTenants")]
    total_tenants: i64,
    #[serde(rename = "totalModules")]
    total_modules: i64,
    #[serde(rename = "avgLatencyMs")]
    avg_latency_ms: i64,
    #[serde(rename = "queueDepth")]
    queue_depth: i64,
}

#[component]
pub fn Dashboard() -> impl IntoView {
    let auth = use_auth();
    let current_user = use_current_user();
    let token = use_token();
    let tenant = use_tenant();

    let dashboard_stats = Resource::new(
        move || (token.get(), tenant.get()),
        move |(token_value, tenant_value)| async move {
            request::<_, DashboardStatsResponse>(
                DASHBOARD_STATS_QUERY,
                json!({}),
                token_value,
                tenant_value,
            )
            .await
        },
    );

    let activity = move || {
        vec![
            (
                translate("app.dashboard.activity.tenant"),
                translate("app.dashboard.activity.tenantDetail"),
                translate("app.dashboard.activity.tenantTime"),
            ),
            (
                translate("app.dashboard.activity.module"),
                translate("app.dashboard.activity.moduleDetail"),
                translate("app.dashboard.activity.moduleTime"),
            ),
            (
                translate("app.dashboard.activity.security"),
                translate("app.dashboard.activity.securityDetail"),
                translate("app.dashboard.activity.securityTime"),
            ),
            (
                translate("app.dashboard.activity.content"),
                translate("app.dashboard.activity.contentDetail"),
                translate("app.dashboard.activity.contentTime"),
            ),
        ]
    };

    let logout = move |_| {
        let auth = auth.clone();
        spawn_local(async move {
            let _ = auth.sign_out().await;
        });
    };

    let title = current_user
        .get()
        .and_then(|user| user.name)
        .unwrap_or_else(|| "Dashboard".to_string());

    let module_sections = components_for_slot(AdminSlot::DashboardSection);

    view! {
        <section class="px-10 py-8">
            <PageHeader
                title=title
                eyebrow=translate("app.nav.dashboard")
                subtitle=translate("app.dashboard.subtitle")
                actions=view! {
                    <LanguageToggle />
                    <Button
                        on_click=logout
                        class="border border-indigo-200 bg-transparent text-blue-600 hover:bg-blue-50"
                    >
                        {move || translate("app.dashboard.logout")}
                    </Button>
                    <Button on_click=move |_| {}>
                        {move || translate("app.dashboard.createTenant")}
                    </Button>
                }
                .into_any()
            />

            <Suspense
                fallback=move || view! {
                    <div class="mb-8 grid gap-5 md:grid-cols-2 xl:grid-cols-4">
                        {(0..4)
                            .map(|_| {
                                view! { <div class="h-32 animate-pulse rounded-2xl bg-slate-100"></div> }
                            })
                            .collect_view()}
                    </div>
                }
            >
                {move || {
                    let stats = dashboard_stats
                        .get()
                        .and_then(|res| res.ok())
                        .and_then(|res| res.dashboard_stats)
                        .map(|stats| {
                            vec![
                                (
                                    translate("app.dashboard.stats.tenants"),
                                    stats.total_tenants.to_string(),
                                    translate("app.dashboard.stats.tenantsHint"),
                                ),
                                (
                                    translate("app.dashboard.stats.modules"),
                                    stats.total_modules.to_string(),
                                    translate("app.dashboard.stats.modulesHint"),
                                ),
                                (
                                    translate("app.dashboard.stats.latency"),
                                    format!("{}ms", stats.avg_latency_ms),
                                    translate("app.dashboard.stats.latencyHint"),
                                ),
                                (
                                    translate("app.dashboard.stats.queue"),
                                    stats.queue_depth.to_string(),
                                    translate("app.dashboard.stats.queueHint"),
                                ),
                            ]
                        })
                        .unwrap_or_else(|| {
                            vec![
                                (translate("app.dashboard.stats.tenants"), "—".to_string(), translate("app.dashboard.stats.tenantsHint")),
                                (translate("app.dashboard.stats.modules"), "—".to_string(), translate("app.dashboard.stats.modulesHint")),
                                (translate("app.dashboard.stats.latency"), "—".to_string(), translate("app.dashboard.stats.latencyHint")),
                                (translate("app.dashboard.stats.queue"), "—".to_string(), translate("app.dashboard.stats.queueHint")),
                            ]
                        });

                    view! {
                        <div class="mb-8 grid gap-5 md:grid-cols-2 xl:grid-cols-4">
                            {stats
                                .into_iter()
                                .map(|(title, value, hint)| {
                                    view! {
                                        <StatsCard
                                            title=title
                                            value=value
                                            icon=view! { <span class="text-slate-400">"•"</span> }.into_any()
                                            trend=hint
                                            class="transition-all hover:scale-[1.02]"
                                        />
                                    }
                                })
                                .collect_view()}
                        </div>
                    }
                }}
            </Suspense>

            <div class="grid gap-6 lg:grid-cols-[1.4fr_1fr]">
                <div class="rounded-2xl bg-white p-6 shadow-[0_18px_36px_rgba(15,23,42,0.08)]">
                    <h4 class="mb-4 text-lg font-semibold">
                        {move || translate("app.dashboard.activity.title")}
                    </h4>
                    {activity()
                        .iter()
                        .map(|(title, detail, time)| {
                            view! {
                                <div class="flex items-center justify-between border-b border-slate-200 py-3 last:border-b-0">
                                    <div>
                                        <strong>{title.clone()}</strong>
                                        <p class="mt-1 text-sm text-slate-500">{detail.clone()}</p>
                                    </div>
                                    <span class="inline-flex items-center rounded-full bg-slate-200 px-3 py-1 text-xs font-semibold text-slate-600">
                                        {time.clone()}
                                    </span>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
                <div class="rounded-2xl bg-white p-6 shadow-[0_18px_36px_rgba(15,23,42,0.08)]">
                    <h4 class="mb-4 text-lg font-semibold">
                        {move || translate("app.dashboard.quick.title")}
                    </h4>
                    <div class="grid gap-3">
                        <a class="rounded-xl bg-slate-100 px-4 py-3 text-left text-sm font-semibold text-slate-900 transition hover:bg-slate-200" href="/security">
                            {move || translate("app.dashboard.quick.security")}
                        </a>
                        <a class="rounded-xl bg-slate-100 px-4 py-3 text-left text-sm font-semibold text-slate-900 transition hover:bg-slate-200" href="/profile">
                            {move || translate("app.dashboard.quick.profile")}
                        </a>
                        <a class="rounded-xl bg-slate-100 px-4 py-3 text-left text-sm font-semibold text-slate-900 transition hover:bg-slate-200" href="/users">
                            {move || translate("app.dashboard.quick.users")}
                        </a>
                    </div>
                </div>
            </div>

            <div class="mt-8 grid gap-6 lg:grid-cols-2">
                {module_sections
                    .into_iter()
                    .map(|module| (module.render)())
                    .collect_view()}
            </div>

        </section>
    }
}
