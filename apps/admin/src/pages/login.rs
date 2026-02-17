use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_auth::hooks::use_auth;
use leptos_router::hooks::use_navigate;

use crate::components::ui::{Button, Input, LanguageToggle};
use crate::providers::locale::translate;

#[component]
pub fn Login() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    let (tenant, set_tenant) = signal(String::from("demo"));
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);

    let on_submit = move |_| {
        if tenant.get().is_empty() || email.get().is_empty() || password.get().is_empty() {
            set_error.set(Some(translate("login.errorRequired").to_string()));
            return;
        }

        let tenant_value = tenant.get().trim().to_string();
        let email_value = email.get().trim().to_string();
        let password_value = password.get();
        let auth = auth.clone();
        let navigate = navigate.clone();

        spawn_local(async move {
            match auth
                .sign_in(email_value, password_value, tenant_value)
                .await
            {
                Ok(()) => {
                    set_error.set(None);
                    navigate("/dashboard", Default::default());
                }
                Err(e) => {
                    set_error.set(Some(format!("{}", e)));
                }
            }
        });
    };

    view! {
        <section class="grid min-h-screen grid-cols-1 lg:grid-cols-[1.2fr_1fr]">
            <aside class="flex flex-col justify-center gap-6 bg-[radial-gradient(circle_at_top_left,#1e3a8a,#0f172a)] p-12 text-white lg:p-16">
                <span class="inline-flex w-fit items-center rounded-full bg-white/10 px-3 py-1 text-xs font-semibold text-white/80">
                    {move || translate("login.badge")}
                </span>
                <h1 class="text-4xl font-semibold">{move || translate("login.heroTitle")}</h1>
                <p class="text-lg text-white/80">{move || translate("login.heroSubtitle")}</p>
                <div class="grid gap-2">
                    <p class="text-sm font-semibold">
                        {move || translate("login.heroListTitle")}
                    </p>
                    <p class="text-sm text-white/75">
                        {move || translate("login.heroListSubtitle")}
                    </p>
                </div>
            </aside>
            <div class="flex flex-col justify-center gap-7 bg-slate-50 p-12 lg:p-20">
                <div class="flex flex-col gap-5 rounded-3xl bg-white p-8 shadow-[0_24px_60px_rgba(15,23,42,0.12)]">
                    <div>
                        <h2 class="text-2xl font-semibold">
                            {move || translate("login.title")}
                        </h2>
                        <p class="text-slate-500">
                            {move || translate("login.subtitle")}
                        </p>
                    </div>
                    <div class="flex items-center justify-between gap-3 text-sm text-slate-600">
                        <span>{move || translate("login.languageLabel")}</span>
                        <LanguageToggle />
                    </div>
                    <Show when=move || error.get().is_some()>
                        <div class="rounded-xl bg-red-100 px-4 py-2 text-sm text-red-700">
                            {move || error.get().unwrap_or_default()}
                        </div>
                    </Show>
                    <Input
                        value=tenant
                        set_value=set_tenant
                        placeholder="demo"
                        label=move || translate("login.tenantLabel")
                    />
                    <Input
                        value=email
                        set_value=set_email
                        placeholder="admin@rustok.io"
                        label=move || translate("login.emailLabel")
                    />
                    <Input
                        value=password
                        set_value=set_password
                        placeholder="••••••••"
                        type_="password"
                        label=move || translate("login.passwordLabel")
                    />
                    <Button on_click=on_submit class="w-full">
                        {move || translate("login.submit")}
                    </Button>
                    <div class="flex justify-between gap-3 text-sm">
                        <a class="text-blue-600 hover:underline" href="/register">
                            {move || translate("login.registerLink")}
                        </a>
                        <a class="text-blue-600 hover:underline" href="/reset">
                            {move || translate("login.resetLink")}
                        </a>
                    </div>
                </div>
            </div>
        </section>
    }
}
