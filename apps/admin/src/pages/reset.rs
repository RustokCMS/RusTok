use leptos::prelude::*;

use crate::components::ui::{Button, Input, LanguageToggle};
use crate::providers::locale::{translate, use_locale};

#[component]
pub fn ResetPassword() -> impl IntoView {
    let locale = use_locale();

    let (tenant, set_tenant) = create_signal(String::new());
    let (email, set_email) = create_signal(String::new());
    let (token, set_token) = create_signal(String::new());
    let (new_password, set_new_password) = create_signal(String::new());
    let (error, set_error) = create_signal(Option::<String>::None);
    let (status, set_status) = create_signal(Option::<String>::None);

    let on_request = move |_| {
        if tenant.get().is_empty() || email.get().is_empty() {
            set_error.set(Some(
                translate(locale.locale.get(), "reset.errorRequired").to_string(),
            ));
            set_status.set(None);
            return;
        }

        set_error.set(None);
        set_status.set(Some(
            translate(locale.locale.get(), "reset.requestSent").to_string(),
        ));
    };

    let on_reset = move |_| {
        if token.get().is_empty() || new_password.get().is_empty() {
            set_error.set(Some(
                translate(locale.locale.get(), "reset.tokenRequired").to_string(),
            ));
            set_status.set(None);
            return;
        }

        set_error.set(None);
        set_status.set(Some(
            translate(locale.locale.get(), "reset.updated").to_string(),
        ));
    };

    view! {
        <section class="auth-grid">
            <aside class="auth-visual">
                <span class="badge">{move || translate(locale.locale.get(), "reset.badge")}</span>
                <h1>{move || translate(locale.locale.get(), "reset.heroTitle")}</h1>
                <p>{move || translate(locale.locale.get(), "reset.heroSubtitle")}</p>
                <div class="auth-note">
                    <p>
                        <strong>{move || translate(locale.locale.get(), "reset.heroListTitle")}</strong>
                    </p>
                    <p>{move || translate(locale.locale.get(), "reset.heroListSubtitle")}</p>
                </div>
            </aside>
            <div class="auth-form">
                <div class="auth-card">
                    <div>
                        <h2>{move || translate(locale.locale.get(), "reset.title")}</h2>
                        <p>{move || translate(locale.locale.get(), "reset.subtitle")}</p>
                    </div>
                    <div class="auth-locale">
                        <span>{move || translate(locale.locale.get(), "reset.languageLabel")}</span>
                        <LanguageToggle />
                    </div>
                    <Show when=move || error.get().is_some()>
                        <div class="alert">{move || error.get().unwrap_or_default()}</div>
                    </Show>
                    <Show when=move || status.get().is_some()>
                        <div class="alert success">{move || status.get().unwrap_or_default()}</div>
                    </Show>
                    <Input
                        value=tenant
                        set_value=set_tenant
                        placeholder="demo"
                        label=move || translate(locale.locale.get(), "reset.tenantLabel").to_string()
                    />
                    <Input
                        value=email
                        set_value=set_email
                        placeholder="admin@rustok.io"
                        label=move || translate(locale.locale.get(), "reset.emailLabel").to_string()
                    />
                    <Button on_click=on_request class="w-full">
                        {move || translate(locale.locale.get(), "reset.requestSubmit")}
                    </Button>
                </div>

                <div class="auth-card">
                    <div>
                        <h3>{move || translate(locale.locale.get(), "reset.tokenTitle")}</h3>
                        <p>{move || translate(locale.locale.get(), "reset.tokenSubtitle")}</p>
                    </div>
                    <Input
                        value=token
                        set_value=set_token
                        placeholder="RESET-2024-0001"
                        label=move || translate(locale.locale.get(), "reset.tokenLabel").to_string()
                    />
                    <Input
                        value=new_password
                        set_value=set_new_password
                        placeholder="••••••••"
                        type_="password"
                        label=move || translate(locale.locale.get(), "reset.newPasswordLabel").to_string()
                    />
                    <p class="form-hint">{move || translate(locale.locale.get(), "reset.tokenHint")}</p>
                    <Button on_click=on_reset class="w-full ghost-button">
                        {move || translate(locale.locale.get(), "reset.tokenSubmit")}
                    </Button>
                    <div class="auth-links">
                        <a class="secondary-link" href="/login">
                            {move || translate(locale.locale.get(), "reset.loginLink")}
                        </a>
                        <a class="secondary-link" href="/register">
                            {move || translate(locale.locale.get(), "reset.registerLink")}
                        </a>
                    </div>
                </div>
            </div>
        </section>
    }
}
