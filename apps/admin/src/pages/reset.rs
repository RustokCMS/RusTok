use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_auth::hooks::use_tenant;

use crate::app::providers::locale::translate;
use crate::shared::ui::{Button, Input, LanguageToggle};

#[component]
pub fn ResetPassword() -> impl IntoView {
    let tenant_signal = use_tenant();

    let initial_tenant = tenant_signal.get().unwrap_or_default();
    let (tenant, set_tenant) = signal(initial_tenant);
    let (email, set_email) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);
    let (status, set_status) = signal(Option::<String>::None);

    let on_request = move |_| {
        if tenant.get().is_empty() || email.get().is_empty() {
            set_error.set(Some(translate("reset.errorRequired").to_string()));
            set_status.set(None);
            return;
        }

        let tenant_value = tenant.get().trim().to_string();
        let email_value = email.get().trim().to_string();

        spawn_local(async move {
            match leptos_auth::api::forgot_password(email_value, tenant_value).await {
                Ok(message) => {
                    set_error.set(None);
                    set_status.set(Some(message));
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
            <aside class="flex flex-col justify-center gap-6 bg-primary p-12 text-primary-foreground lg:p-16">
                <span class="inline-flex w-fit items-center rounded-full bg-primary-foreground/10 px-3 py-1 text-xs font-semibold text-primary-foreground/80">
                    {move || translate("reset.badge")}
                </span>
                <h1 class="text-4xl font-semibold">{move || translate("reset.heroTitle")}</h1>
                <p class="text-lg text-primary-foreground/80">{move || translate("reset.heroSubtitle")}</p>
                <div class="grid gap-2">
                    <p class="text-sm font-semibold">
                        {move || translate("reset.heroListTitle")}
                    </p>
                    <p class="text-sm text-primary-foreground/75">
                        {move || translate("reset.heroListSubtitle")}
                    </p>
                </div>
            </aside>
            <div class="flex flex-col justify-center gap-7 bg-background p-12 lg:p-20">
                <div class="flex flex-col gap-5 rounded-xl border border-border bg-card p-8 shadow-md">
                    <div>
                        <h2 class="text-2xl font-semibold text-card-foreground">
                            {move || translate("reset.title")}
                        </h2>
                        <p class="text-muted-foreground">
                            {move || translate("reset.subtitle")}
                        </p>
                    </div>
                    <div class="flex items-center justify-between gap-3 text-sm text-muted-foreground">
                        <span>{move || translate("reset.languageLabel")}</span>
                        <LanguageToggle />
                    </div>
                    <Show when=move || error.get().is_some()>
                        <div class="rounded-md bg-destructive/10 border border-destructive/20 px-4 py-2 text-sm text-destructive">
                            {move || error.get().unwrap_or_default()}
                        </div>
                    </Show>
                    <Show when=move || status.get().is_some()>
                        <div class="rounded-md bg-emerald-100 border border-emerald-200 px-4 py-2 text-sm text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-400">
                            {move || status.get().unwrap_or_default()}
                        </div>
                    </Show>
                    <Input value=tenant set_value=set_tenant placeholder="demo" label=move || translate("reset.tenantLabel") />
                    <Input value=email set_value=set_email placeholder="admin@rustok.io" label=move || translate("reset.emailLabel") />
                    <Button on_click=on_request class="w-full">{move || translate("reset.requestSubmit")}</Button>
                    <div class="flex justify-between gap-3 text-sm">
                        <a class="text-primary hover:underline underline-offset-4" href="/login">
                            {move || translate("reset.loginLink")}
                        </a>
                        <a class="text-primary hover:underline underline-offset-4" href="/register">
                            {move || translate("reset.registerLink")}
                        </a>
                    </div>
                </div>
            </div>
        </section>
    }
}
