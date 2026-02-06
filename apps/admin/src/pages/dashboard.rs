use leptos::prelude::*;

use crate::components::ui::{Button, LanguageToggle};
use crate::providers::auth::use_auth;
use crate::providers::locale::{translate, use_locale};

#[component]
pub fn Dashboard() -> impl IntoView {
    let auth = use_auth();
    let locale = use_locale();

    let stats = move || {
        vec![
            (
                translate(locale.locale.get(), "app.dashboard.stats.tenants"),
                "28",
                translate(locale.locale.get(), "app.dashboard.stats.tenantsHint"),
            ),
            (
                translate(locale.locale.get(), "app.dashboard.stats.modules"),
                "12",
                translate(locale.locale.get(), "app.dashboard.stats.modulesHint"),
            ),
            (
                translate(locale.locale.get(), "app.dashboard.stats.latency"),
                "128ms",
                translate(locale.locale.get(), "app.dashboard.stats.latencyHint"),
            ),
            (
                translate(locale.locale.get(), "app.dashboard.stats.queue"),
                "7",
                translate(locale.locale.get(), "app.dashboard.stats.queueHint"),
            ),
        ]
    };

    let activity = move || {
        vec![
            (
                translate(locale.locale.get(), "app.dashboard.activity.tenant"),
                translate(locale.locale.get(), "app.dashboard.activity.tenantDetail"),
                translate(locale.locale.get(), "app.dashboard.activity.tenantTime"),
            ),
            (
                translate(locale.locale.get(), "app.dashboard.activity.module"),
                translate(locale.locale.get(), "app.dashboard.activity.moduleDetail"),
                translate(locale.locale.get(), "app.dashboard.activity.moduleTime"),
            ),
            (
                translate(locale.locale.get(), "app.dashboard.activity.security"),
                translate(locale.locale.get(), "app.dashboard.activity.securityDetail"),
                translate(locale.locale.get(), "app.dashboard.activity.securityTime"),
            ),
            (
                translate(locale.locale.get(), "app.dashboard.activity.content"),
                translate(locale.locale.get(), "app.dashboard.activity.contentDetail"),
                translate(locale.locale.get(), "app.dashboard.activity.contentTime"),
            ),
        ]
    };

    let logout = move |_| {
        auth.set_token.set(None);
        auth.set_user.set(None);
    };

    view! {
        <section class="px-10 py-8">
            <header class="mb-8 flex flex-wrap items-center justify-between gap-4">
                <div>
                    <span class="inline-flex items-center rounded-full bg-slate-200 px-3 py-1 text-xs font-semibold text-slate-600">
                        {move || translate(locale.locale.get(), "app.nav.dashboard")}
                    </span>
                    <h1 class="mt-2 text-2xl font-semibold">
                        {move || {
                            auth.user
                                .get()
                                .and_then(|user| user.name)
                                .unwrap_or_else(|| "Добро пожаловать, Админ".to_string())
                        }}
                    </h1>
                    <p class="mt-2 text-sm text-slate-500">
                        {move || translate(locale.locale.get(), "app.dashboard.subtitle")}
                    </p>
                </div>
                <div class="flex flex-wrap items-center gap-3">
                    <LanguageToggle />
                    <Button
                        on_click=logout
                        class="border border-indigo-200 bg-transparent text-blue-600 hover:bg-blue-50"
                    >
                        {move || translate(locale.locale.get(), "app.dashboard.logout")}
                    </Button>
                    <Button on_click=move |_| {}>
                        {move || translate(locale.locale.get(), "app.dashboard.createTenant")}
                    </Button>
                </div>
            </header>

            <div class="mb-8 grid gap-5 md:grid-cols-2 xl:grid-cols-4">
                {stats()
                    .iter()
                    .map(|(title, value, hint)| {
                        view! {
                            <div class="rounded-2xl bg-white p-5 shadow-[0_16px_30px_rgba(15,23,42,0.08)]">
                                <h3 class="text-sm text-slate-500">{title.clone()}</h3>
                                <strong class="text-2xl">{*value}</strong>
                                <p class="mt-2 text-sm text-slate-400">{hint.clone()}</p>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>

            <div class="grid gap-6 lg:grid-cols-[1.4fr_1fr]">
                <div class="rounded-2xl bg-white p-6 shadow-[0_18px_36px_rgba(15,23,42,0.08)]">
                    <h4 class="mb-4 text-lg font-semibold">
                        {move || translate(locale.locale.get(), "app.dashboard.activity.title")}
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
                        {move || translate(locale.locale.get(), "app.dashboard.quick.title")}
                    </h4>
                    <div class="grid gap-3">
                        <a
                            class="rounded-xl bg-slate-100 px-4 py-3 text-left text-sm font-semibold text-slate-900 transition hover:bg-slate-200"
                            href="/security"
                        >
                            {move || translate(locale.locale.get(), "app.dashboard.quick.security")}
                        </a>
                        <a
                            class="rounded-xl bg-slate-100 px-4 py-3 text-left text-sm font-semibold text-slate-900 transition hover:bg-slate-200"
                            href="/profile"
                        >
                            {move || translate(locale.locale.get(), "app.dashboard.quick.profile")}
                        </a>
                        <a
                            class="rounded-xl bg-slate-100 px-4 py-3 text-left text-sm font-semibold text-slate-900 transition hover:bg-slate-200"
                            href="/users"
                        >
                            {move || translate(locale.locale.get(), "app.dashboard.quick.users")}
                        </a>
                        <button
                            type="button"
                            class="rounded-xl bg-slate-100 px-4 py-3 text-left text-sm font-semibold text-slate-900 transition hover:bg-slate-200"
                        >
                            {move || translate(locale.locale.get(), "app.dashboard.quick.metrics")}
                        </button>
                        <button
                            type="button"
                            class="rounded-xl bg-slate-100 px-4 py-3 text-left text-sm font-semibold text-slate-900 transition hover:bg-slate-200"
                        >
                            {move || translate(locale.locale.get(), "app.dashboard.quick.roles")}
                        </button>
                    </div>
                </div>
            </div>
        </section>
    }
}
