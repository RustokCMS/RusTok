use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_auth::hooks::use_auth;
use leptos_router::hooks::use_navigate;

use crate::components::ui::{Button, Input, LanguageToggle};
use crate::providers::locale::translate;

#[component]
pub fn Register() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    let (tenant, set_tenant) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (name, set_name) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);
    let (status, set_status) = signal(Option::<String>::None);

    let on_submit = move |_| {
        if tenant.get().is_empty() || email.get().is_empty() || password.get().is_empty() {
            set_error.set(Some(translate("register.errorRequired").to_string()));
            set_status.set(None);
            return;
        }

        let tenant_value = tenant.get().trim().to_string();
        let email_value = email.get().trim().to_string();
        let password_value = password.get();
        let name_value = name.get().trim().to_string();
        let name_opt = if name_value.is_empty() {
            None
        } else {
            Some(name_value)
        };
        let auth = auth.clone();
        let navigate = navigate.clone();

        spawn_local(async move {
            match auth
                .sign_up(email_value, password_value, name_opt, tenant_value)
                .await
            {
                Ok(()) => {
                    set_error.set(None);
                    set_status.set(Some(translate("register.success").to_string()));
                    navigate("/dashboard", Default::default());
                }
                Err(e) => {
                    set_error.set(Some(format!("{}", e)));
                    set_status.set(None);
                }
            }
        });
    };

    view! {
        <section class="grid min-h-screen grid-cols-1 lg:grid-cols-[1.2fr_1fr]">
            <aside class="flex flex-col justify-center gap-6 bg-[radial-gradient(circle_at_top_left,#1e3a8a,#0f172a)] p-12 text-white lg:p-16">
                <span class="inline-flex w-fit items-center rounded-full bg-white/10 px-3 py-1 text-xs font-semibold text-white/80">
                    {move || translate("register.badge")}
                </span>
                <h1 class="text-4xl font-semibold">{move || translate("register.heroTitle")}</h1>
                <p class="text-lg text-white/80">{move || translate("register.heroSubtitle")}</p>
                <div class="grid gap-2">
                    <p class="text-sm font-semibold">
                        {move || translate("register.heroListTitle")}
                    </p>
                    <p class="text-sm text-white/75">
                        {move || translate("register.heroListSubtitle")}
                    </p>
                </div>
            </aside>
            <div class="flex flex-col justify-center gap-7 bg-slate-50 p-12 lg:p-20">
                <div class="flex flex-col gap-5 rounded-3xl bg-white p-8 shadow-[0_24px_60px_rgba(15,23,42,0.12)]">
                    <div>
                        <h2 class="text-2xl font-semibold">
                            {move || translate("register.title")}
                        </h2>
                        <p class="text-slate-500">
                            {move || translate("register.subtitle")}
                        </p>
                    </div>
                    <div class="flex items-center justify-between gap-3 text-sm text-slate-600">
                        <span>{move || translate("register.languageLabel")}</span>
                        <LanguageToggle />
                    </div>
                    <Show when=move || error.get().is_some()>
                        <div class="rounded-xl bg-red-100 px-4 py-2 text-sm text-red-700">
                            {move || error.get().unwrap_or_default()}
                        </div>
                    </Show>
                    <Show when=move || status.get().is_some()>
                        <div class="rounded-xl bg-emerald-100 px-4 py-2 text-sm text-emerald-700">
                            {move || status.get().unwrap_or_default()}
                        </div>
                    </Show>
                    <Input
                        value=tenant
                        set_value=set_tenant
                        placeholder="demo"
                        label=move || translate("register.tenantLabel")
                    />
                    <Input
                        value=email
                        set_value=set_email
                        placeholder="admin@rustok.io"
                        label=move || translate("register.emailLabel")
                    />
                    <Input
                        value=name
                        set_value=set_name
                        placeholder="Alex Morgan"
                        label=move || translate("register.nameLabel")
                    />
                    <Input
                        value=password
                        set_value=set_password
                        placeholder="••••••••"
                        type_="password"
                        label=move || translate("register.passwordLabel")
                    />
                    <p class="text-sm text-slate-500">
                        {move || translate("register.passwordHint")}
                    </p>
                    <Button on_click=on_submit class="w-full">
                        {move || translate("register.submit")}
                    </Button>
                    <div class="flex justify-between gap-3 text-sm">
                        <a class="text-blue-600 hover:underline" href="/login">
                            {move || translate("register.loginLink")}
                        </a>
                        <a class="text-blue-600 hover:underline" href="/reset">
                            {move || translate("register.resetLink")}
                        </a>
                    </div>
                </div>
            </div>
        </section>
    }
}
