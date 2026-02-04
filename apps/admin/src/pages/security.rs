use leptos::prelude::*;

use crate::components::ui::{Button, Input};
use crate::providers::locale::{translate, use_locale};

struct SessionItem<'a> {
    device: &'a str,
    ip: &'a str,
    last_active_key: &'a str,
    status_key: &'a str,
}

struct LoginEvent<'a> {
    timestamp_key: &'a str,
    ip: &'a str,
    status_key: &'a str,
}

#[component]
pub fn Security() -> impl IntoView {
    let locale = use_locale();

    let (current_password, set_current_password) = create_signal(String::new());
    let (new_password, set_new_password) = create_signal(String::new());
    let (status, set_status) = create_signal(Option::<String>::None);

    let on_change_password = move |_| {
        if current_password.get().is_empty() || new_password.get().is_empty() {
            set_status.set(Some(
                translate(locale.locale.get(), "security.passwordRequired").to_string(),
            ));
            return;
        }

        set_status.set(Some(
            translate(locale.locale.get(), "security.passwordUpdated").to_string(),
        ));
    };

    let sessions = vec![
        SessionItem {
            device: "MacBook Pro · Chrome",
            ip: "91.204.12.8",
            last_active_key: "security.session.lastActiveNow",
            status_key: "security.session.active",
        },
        SessionItem {
            device: "iPhone 15 · Safari",
            ip: "31.54.102.3",
            last_active_key: "security.session.lastActiveYesterday",
            status_key: "security.session.idle",
        },
        SessionItem {
            device: "Windows · Edge",
            ip: "213.87.44.19",
            last_active_key: "security.session.lastActiveWeek",
            status_key: "security.session.inactive",
        },
    ];

    let history = vec![
        LoginEvent {
            timestamp_key: "security.history.timestamp.latest",
            ip: "91.204.12.8",
            status_key: "security.history.success",
        },
        LoginEvent {
            timestamp_key: "security.history.timestamp.prev",
            ip: "31.54.102.3",
            status_key: "security.history.success",
        },
        LoginEvent {
            timestamp_key: "security.history.timestamp.fail",
            ip: "81.23.119.52",
            status_key: "security.history.failed",
        },
    ];

    view! {
        <section class="settings-page">
            <header class="settings-header">
                <div>
                    <span class="badge">{move || translate(locale.locale.get(), "security.badge")}</span>
                    <h1>{move || translate(locale.locale.get(), "security.title")}</h1>
                    <p>{move || translate(locale.locale.get(), "security.subtitle")}</p>
                </div>
                <div class="settings-actions">
                    <Button on_click=move |_| {} class="ghost-button">
                        {move || translate(locale.locale.get(), "security.signOutAll")}
                    </Button>
                </div>
            </header>

            <div class="settings-grid">
                <div class="settings-card">
                    <h3>{move || translate(locale.locale.get(), "security.passwordTitle")}</h3>
                    <p class="section-subtitle">
                        {move || translate(locale.locale.get(), "security.passwordSubtitle")}
                    </p>
                    <Input
                        value=current_password
                        set_value=set_current_password
                        placeholder="••••••••"
                        type_="password"
                        label=move || translate(locale.locale.get(), "security.currentPasswordLabel").to_string()
                    />
                    <Input
                        value=new_password
                        set_value=set_new_password
                        placeholder="••••••••"
                        type_="password"
                        label=move || translate(locale.locale.get(), "security.newPasswordLabel").to_string()
                    />
                    <p class="form-hint">{move || translate(locale.locale.get(), "security.passwordHint")}</p>
                    <Button on_click=on_change_password class="w-full">
                        {move || translate(locale.locale.get(), "security.passwordSubmit")}
                    </Button>
                    <Show when=move || status.get().is_some()>
                        <div class="alert success">{move || status.get().unwrap_or_default()}</div>
                    </Show>
                </div>

                <div class="settings-card">
                    <h3>{move || translate(locale.locale.get(), "security.sessionsTitle")}</h3>
                    <p class="section-subtitle">
                        {move || translate(locale.locale.get(), "security.sessionsSubtitle")}
                    </p>
                    <div class="session-list">
                        {sessions
                            .iter()
                            .map(|session| {
                                view! {
                                    <div class="session-item">
                                        <div>
                                            <strong>{session.device}</strong>
                                            <p class="form-hint">
                                                {move || translate(locale.locale.get(), "security.sessionIp")} ": "
                                                {session.ip}
                                            </p>
                                        </div>
                                        <div class="session-meta">
                                            <span class="status-pill">
                                                {move || translate(locale.locale.get(), session.status_key)}
                                            </span>
                                            <span class="meta-text">
                                                {move || translate(locale.locale.get(), session.last_active_key)}
                                            </span>
                                        </div>
                                    </div>
                                }
                            })
                            .collect_view()}
                    </div>
                </div>

                <div class="settings-card">
                    <h3>{move || translate(locale.locale.get(), "security.historyTitle")}</h3>
                    <p class="section-subtitle">
                        {move || translate(locale.locale.get(), "security.historySubtitle")}
                    </p>
                    <div class="session-list">
                        {history
                            .iter()
                            .map(|event| {
                                view! {
                                    <div class="session-item">
                                        <div>
                                            <strong>
                                                {move || translate(locale.locale.get(), event.timestamp_key)}
                                            </strong>
                                            <p class="form-hint">
                                                {move || translate(locale.locale.get(), "security.sessionIp")} ": "
                                                {event.ip}
                                            </p>
                                        </div>
                                        <span class="status-pill">
                                            {move || translate(locale.locale.get(), event.status_key)}
                                        </span>
                                    </div>
                                }
                            })
                            .collect_view()}
                    </div>
                </div>
            </div>
        </section>
    }
}
