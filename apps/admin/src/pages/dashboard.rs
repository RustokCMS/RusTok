use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_auth::hooks::{use_auth, use_current_user};

use crate::components::ui::{Button, LanguageToggle, PageHeader, StatsCard};
use crate::providers::locale::translate;

#[component]
pub fn Dashboard() -> impl IntoView {
    let auth = use_auth();
    let current_user = use_current_user();

    let stats = move || {
        vec![
            (
                translate("app.dashboard.stats.tenants"),
                "28",
                translate("app.dashboard.stats.tenantsHint"),
                view! {
                    <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
                    </svg>
                }.into_any()
            ),
            (
                translate("app.dashboard.stats.modules"),
                "12",
                translate("app.dashboard.stats.modulesHint"),
                view! {
                    <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
                    </svg>
                }.into_any()
            ),
            (
                translate("app.dashboard.stats.latency"),
                "128ms",
                translate("app.dashboard.stats.latencyHint"),
                view! {
                    <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                    </svg>
                }.into_any()
            ),
            (
                translate("app.dashboard.stats.queue"),
                "7",
                translate("app.dashboard.stats.queueHint"),
                view! {
                    <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
                    </svg>
                }.into_any()
            ),
        ]
    };

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

            <div class="mb-8 grid gap-5 md:grid-cols-2 xl:grid-cols-4">
                {stats()
                    .into_iter()
                    .map(|(title, value, hint, icon)| {
                        view! {
                            <StatsCard
                                title=title
                                value=value
                                icon=icon
                                trend=hint
                                class="transition-all hover:scale-[1.02]"
                            />
                        }
                    })
                    .collect_view()}
            </div>

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
                        <a
                            class="rounded-xl bg-slate-100 px-4 py-3 text-left text-sm font-semibold text-slate-900 transition hover:bg-slate-200"
                            href="/security"
                        >
                            {move || translate("app.dashboard.quick.security")}
                        </a>
                        <a
                            class="rounded-xl bg-slate-100 px-4 py-3 text-left text-sm font-semibold text-slate-900 transition hover:bg-slate-200"
                            href="/profile"
                        >
                            {move || translate("app.dashboard.quick.profile")}
                        </a>
                        <a
                            class="rounded-xl bg-slate-100 px-4 py-3 text-left text-sm font-semibold text-slate-900 transition hover:bg-slate-200"
                            href="/users"
                        >
                            {move || translate("app.dashboard.quick.users")}
                        </a>
                    </div>
                </div>
            </div>
        </section>
    }
}
