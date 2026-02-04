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
        <section class="dashboard">
            <header class="dashboard-header">
                <div>
                    <span class="badge">{move || translate(locale.locale.get(), "app.nav.dashboard")}</span>
                    <h1>
                        {move || {
                            auth.user
                                .get()
                                .and_then(|user| user.name)
                                .unwrap_or_else(|| "Добро пожаловать, Админ".to_string())
                        }}
                    </h1>
                    <p style="margin:8px 0 0; color:#64748b;">
                        {move || translate(locale.locale.get(), "app.dashboard.subtitle")}
                    </p>
                </div>
                <div class="dashboard-actions">
                    <LanguageToggle />
                    <Button on_click=logout class="ghost-button">
                        {move || translate(locale.locale.get(), "app.dashboard.logout")}
                    </Button>
                    <Button on_click=move |_| {}>
                        {move || translate(locale.locale.get(), "app.dashboard.createTenant")}
                    </Button>
                </div>
            </header>

            <div class="stats-grid">
                {stats()
                    .iter()
                    .map(|(title, value, hint)| {
                        view! {
                            <div class="stat-card">
                                <h3>{*title}</h3>
                                <strong>{*value}</strong>
                                <p style="margin:8px 0 0; color:#94a3b8;">{*hint}</p>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>

            <div class="dashboard-panels">
                <div class="panel">
                    <h4>{move || translate(locale.locale.get(), "app.dashboard.activity.title")}</h4>
                    {activity()
                        .iter()
                        .map(|(title, detail, time)| {
                            view! {
                                <div class="activity-item">
                                    <div>
                                        <strong>{*title}</strong>
                                        <p style="margin:4px 0 0; color:#64748b;">{*detail}</p>
                                    </div>
                                    <span class="badge">{*time}</span>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
                <div class="panel">
                    <h4>{move || translate(locale.locale.get(), "app.dashboard.quick.title")}</h4>
                    <div class="quick-actions">
                        <a href="/security">
                            {move || translate(locale.locale.get(), "app.dashboard.quick.security")}
                        </a>
                        <a href="/profile">
                            {move || translate(locale.locale.get(), "app.dashboard.quick.profile")}
                        </a>
                        <a href="/users">
                            {move || translate(locale.locale.get(), "app.dashboard.quick.users")}
                        </a>
                        <button type="button">
                            {move || translate(locale.locale.get(), "app.dashboard.quick.metrics")}
                        </button>
                        <button type="button">
                            {move || translate(locale.locale.get(), "app.dashboard.quick.roles")}
                        </button>
                    </div>
                </div>
            </div>
        </section>
    }
}
