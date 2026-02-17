use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_auth::hooks::{use_auth, use_tenant, use_token};
use serde::{Deserialize, Serialize};

use crate::api::{request, ApiError};
use crate::components::ui::{Button, Input};
use crate::providers::locale::translate;

// GraphQL mutation for changing password
const CHANGE_PASSWORD_MUTATION: &str = r#"
mutation ChangePassword($input: ChangePasswordInput!) {
    changePassword(input: $input) {
        success
    }
}
"#;

#[derive(Serialize)]
struct ChangePasswordVariables {
    input: ChangePasswordInput,
}

#[derive(Serialize)]
struct ChangePasswordInput {
    #[serde(rename = "currentPassword")]
    current_password: String,
    #[serde(rename = "newPassword")]
    new_password: String,
}

#[derive(Deserialize)]
struct ChangePasswordResponse {
    #[serde(rename = "changePassword")]
    change_password: SuccessPayload,
}

#[derive(Deserialize)]
struct SuccessPayload {
    #[allow(dead_code)]
    success: bool,
}

#[component]
pub fn Security() -> impl IntoView {
    let auth = use_auth();
    let token = use_token();
    let tenant = use_tenant();

    let (current_password, set_current_password) = signal(String::new());
    let (new_password, set_new_password) = signal(String::new());
    let (status, set_status) = signal(Option::<String>::None);
    let (error, set_error) = signal(Option::<String>::None);

    let on_change_password = move |_| {
        if current_password.get().is_empty() || new_password.get().is_empty() {
            set_error.set(Some(translate("security.passwordRequired").to_string()));
            set_status.set(None);
            return;
        }

        let token_value = token.get();
        let tenant_value = tenant.get();
        if token_value.is_none() {
            set_error.set(Some(translate("errors.auth.unauthorized").to_string()));
            set_status.set(None);
            return;
        }

        let current_password_value = current_password.get();
        let new_password_value = new_password.get();

        spawn_local(async move {
            let result = request::<ChangePasswordVariables, ChangePasswordResponse>(
                CHANGE_PASSWORD_MUTATION,
                ChangePasswordVariables {
                    input: ChangePasswordInput {
                        current_password: current_password_value,
                        new_password: new_password_value,
                    },
                },
                token_value,
                tenant_value,
            )
            .await;

            match result {
                Ok(_) => {
                    set_error.set(None);
                    set_status.set(Some(translate("security.passwordUpdated").to_string()));
                    set_current_password.set(String::new());
                    set_new_password.set(String::new());
                }
                Err(err) => {
                    let message = match err {
                        ApiError::Unauthorized => translate("errors.auth.unauthorized").to_string(),
                        ApiError::Http(_) => translate("errors.http").to_string(),
                        ApiError::Network => translate("errors.network").to_string(),
                        ApiError::Graphql(_) => translate("errors.unknown").to_string(),
                    };
                    set_error.set(Some(message));
                    set_status.set(None);
                }
            }
        });
    };

    let on_sign_out_all = move |_: leptos::ev::MouseEvent| {
        let auth = auth.clone();
        spawn_local(async move {
            let _ = auth.sign_out().await;
        });
    };

    view! {
        <section class="px-10 py-8">
            <header class="mb-6 flex flex-wrap items-start justify-between gap-4">
                <div>
                    <span class="inline-flex items-center rounded-full bg-slate-200 px-3 py-1 text-xs font-semibold text-slate-600">
                        {move || translate("security.badge")}
                    </span>
                    <h1 class="mt-2 text-2xl font-semibold">
                        {move || translate("security.title")}
                    </h1>
                    <p class="mt-2 text-sm text-slate-500">
                        {move || translate("security.subtitle")}
                    </p>
                </div>
                <div class="flex flex-wrap items-center gap-3">
                    <Button
                        on_click=on_sign_out_all
                        class="border border-indigo-200 bg-transparent text-blue-600 hover:bg-blue-50"
                    >
                        {move || translate("security.signOutAll")}
                    </Button>
                </div>
            </header>

            <div class="grid gap-6 lg:grid-cols-2">
                <div class="grid gap-4 rounded-2xl bg-white p-6 shadow-[0_18px_36px_rgba(15,23,42,0.08)]">
                    <h3 class="text-lg font-semibold">
                        {move || translate("security.passwordTitle")}
                    </h3>
                    <p class="text-sm text-slate-500">
                        {move || translate("security.passwordSubtitle")}
                    </p>
                    <Input
                        value=current_password
                        set_value=set_current_password
                        placeholder="••••••••"
                        type_="password"
                        label=move || translate("security.currentPasswordLabel")
                    />
                    <Input
                        value=new_password
                        set_value=set_new_password
                        placeholder="••••••••"
                        type_="password"
                        label=move || translate("security.newPasswordLabel")
                    />
                    <p class="text-sm text-slate-500">
                        {move || translate("security.passwordHint")}
                    </p>
                    <Button on_click=on_change_password class="w-full">
                        {move || translate("security.passwordSubmit")}
                    </Button>
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
                </div>

                <div class="grid gap-4 rounded-2xl bg-white p-6 shadow-[0_18px_36px_rgba(15,23,42,0.08)]">
                    <h3 class="text-lg font-semibold">
                        {move || translate("security.sessionsTitle")}
                    </h3>
                    <p class="text-sm text-slate-500">
                        {move || translate("security.sessionsSubtitle")}
                    </p>
                    <div class="rounded-xl bg-slate-50 px-4 py-8 text-center text-sm text-slate-500">
                        "Session management via GraphQL — coming soon"
                    </div>
                </div>
            </div>
        </section>
    }
}
