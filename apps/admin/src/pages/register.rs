use leptos::prelude::*;

use crate::components::ui::{Button, Input, LanguageToggle};
use crate::providers::locale::{translate, use_locale};

#[component]
pub fn Register() -> impl IntoView {
    let locale = use_locale();

    let (tenant, set_tenant) = create_signal(String::new());
    let (email, set_email) = create_signal(String::new());
    let (name, set_name) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (invite_token, set_invite_token) = create_signal(String::new());
    let (verification_email, set_verification_email) = create_signal(String::new());
    let (error, set_error) = create_signal(Option::<String>::None);
    let (status, set_status) = create_signal(Option::<String>::None);

    let on_submit = move |_| {
        if tenant.get().is_empty() || email.get().is_empty() || password.get().is_empty() {
            set_error.set(Some(
                translate(locale.locale.get(), "register.errorRequired").to_string(),
            ));
            set_status.set(None);
            return;
        }

        set_error.set(None);
        set_status.set(Some(
            translate(locale.locale.get(), "register.success").to_string(),
        ));
    };

    let on_accept_invite = move |_| {
        if invite_token.get().is_empty() {
            set_error.set(Some(
                translate(locale.locale.get(), "register.inviteRequired").to_string(),
            ));
            set_status.set(None);
            return;
        }

        set_error.set(None);
        set_status.set(Some(
            translate(locale.locale.get(), "register.inviteAccepted").to_string(),
        ));
    };

    let on_resend_verification = move |_| {
        if verification_email.get().is_empty() {
            set_error.set(Some(
                translate(locale.locale.get(), "register.verifyRequired").to_string(),
            ));
            set_status.set(None);
            return;
        }

        set_error.set(None);
        set_status.set(Some(
            translate(locale.locale.get(), "register.verifySent").to_string(),
        ));
    };

    view! {
        <section class="auth-grid">
            <aside class="auth-visual">
                <span class="badge">{move || translate(locale.locale.get(), "register.badge")}</span>
                <h1>{move || translate(locale.locale.get(), "register.heroTitle")}</h1>
                <p>{move || translate(locale.locale.get(), "register.heroSubtitle")}</p>
                <div class="auth-note">
                    <p>
                        <strong>{move || translate(locale.locale.get(), "register.heroListTitle")}</strong>
                    </p>
                    <p>{move || translate(locale.locale.get(), "register.heroListSubtitle")}</p>
                </div>
            </aside>
            <div class="auth-form">
                <div class="auth-card">
                    <div>
                        <h2>{move || translate(locale.locale.get(), "register.title")}</h2>
                        <p>{move || translate(locale.locale.get(), "register.subtitle")}</p>
                    </div>
                    <div class="auth-locale">
                        <span>{move || translate(locale.locale.get(), "register.languageLabel")}</span>
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
                        label=move || translate(locale.locale.get(), "register.tenantLabel").to_string()
                    />
                    <Input
                        value=email
                        set_value=set_email
                        placeholder="admin@rustok.io"
                        label=move || translate(locale.locale.get(), "register.emailLabel").to_string()
                    />
                    <Input
                        value=name
                        set_value=set_name
                        placeholder="Alex Morgan"
                        label=move || translate(locale.locale.get(), "register.nameLabel").to_string()
                    />
                    <Input
                        value=password
                        set_value=set_password
                        placeholder="••••••••"
                        type_="password"
                        label=move || translate(locale.locale.get(), "register.passwordLabel").to_string()
                    />
                    <p class="form-hint">{move || translate(locale.locale.get(), "register.passwordHint")}</p>
                    <Button on_click=on_submit class="w-full">
                        {move || translate(locale.locale.get(), "register.submit")}
                    </Button>
                    <div class="auth-links">
                        <a class="secondary-link" href="/login">
                            {move || translate(locale.locale.get(), "register.loginLink")}
                        </a>
                        <a class="secondary-link" href="/reset">
                            {move || translate(locale.locale.get(), "register.resetLink")}
                        </a>
                    </div>
                </div>

                <div class="auth-card">
                    <div>
                        <h3>{move || translate(locale.locale.get(), "register.inviteTitle")}</h3>
                        <p>{move || translate(locale.locale.get(), "register.inviteSubtitle")}</p>
                    </div>
                    <Input
                        value=invite_token
                        set_value=set_invite_token
                        placeholder="INVITE-2024-0001"
                        label=move || translate(locale.locale.get(), "register.inviteLabel").to_string()
                    />
                    <Button on_click=on_accept_invite class="w-full ghost-button">
                        {move || translate(locale.locale.get(), "register.inviteSubmit")}
                    </Button>
                </div>

                <div class="auth-card">
                    <div>
                        <h3>{move || translate(locale.locale.get(), "register.verifyTitle")}</h3>
                        <p>{move || translate(locale.locale.get(), "register.verifySubtitle")}</p>
                    </div>
                    <Input
                        value=verification_email
                        set_value=set_verification_email
                        placeholder="admin@rustok.io"
                        label=move || translate(locale.locale.get(), "register.verifyLabel").to_string()
                    />
                    <Button on_click=on_resend_verification class="w-full ghost-button">
                        {move || translate(locale.locale.get(), "register.verifySubmit")}
                    </Button>
                </div>
            </div>
        </section>
    }
}
