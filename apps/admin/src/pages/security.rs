use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_auth::hooks::{use_auth, use_tenant, use_token};
use leptos_hook_form::FormState;
use serde::{Deserialize, Serialize};

use crate::app::providers::locale::translate;
use crate::shared::api::{request, ApiError};
use crate::shared::ui::{ui_button, ui_input};

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
    _change_password: SuccessPayload,
}

#[derive(Deserialize)]
struct SuccessPayload {
    #[allow(dead_code)]
    success: bool,
}

#[component]
pub fn security() -> impl IntoView {
    let auth = use_auth();
    let token = use_token();
    let tenant = use_tenant();

    let (current_password, set_current_password) = signal(String::new());
    let (new_password, set_new_password) = signal(String::new());
    let (form_state, set_form_state) = signal(FormState::idle());
    let (success_message, set_success_message) = signal(Option::<String>::None);

    let on_change_password = move |_| {
        if current_password.get().is_empty() || new_password.get().is_empty() {
            set_form_state.set(FormState::with_form_error(
                translate("security.passwordRequired").to_string(),
            ));
            set_success_message.set(None);
            return;
        }

        let token_value = token.get();
        let tenant_value = tenant.get();
        if token_value.is_none() {
            set_form_state.set(FormState::with_form_error(
                translate("errors.auth.unauthorized").to_string(),
            ));
            set_success_message.set(None);
            return;
        }

        let current_password_value = current_password.get();
        let new_password_value = new_password.get();

        set_form_state.set(FormState::submitting());
        set_success_message.set(None);

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
                    set_form_state.set(FormState::idle());
                    set_success_message
                        .set(Some(translate("security.passwordUpdated").to_string()));
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
                    set_form_state.set(FormState::with_form_error(message));
                    set_success_message.set(None);
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
                    <span class="inline-flex items-center rounded-full border bg-secondary px-3 py-1 text-xs font-semibold text-secondary-foreground">
                        {move || translate("security.badge")}
                    </span>
                    <h1 class="mt-2 text-2xl font-semibold text-foreground">
                        {move || translate("security.title")}
                    </h1>
                    <p class="mt-2 text-sm text-muted-foreground">
                        {move || translate("security.subtitle")}
                    </p>
                </div>
                <div class="flex flex-wrap items-center gap-3">
                    <ui_button
                        on_click=on_sign_out_all
                        class="border border-border bg-transparent text-foreground hover:bg-accent hover:text-accent-foreground"
                    >
                        {move || translate("security.signOutAll")}
                    </ui_button>
                </div>
            </header>

            <div class="grid gap-6 lg:grid-cols-2">
                <div class="grid gap-4 rounded-xl border border-border bg-card p-6 shadow-sm">
                    <h3 class="text-lg font-semibold text-card-foreground">
                        {move || translate("security.passwordTitle")}
                    </h3>
                    <p class="text-sm text-muted-foreground">
                        {move || translate("security.passwordSubtitle")}
                    </p>
                     <ui_input
                        value=current_password
                        set_value=set_current_password
                        placeholder="••••••••"
                        type_="password"
                        label=move || translate("security.currentPasswordLabel")
                    />
                    <ui_input
                        value=new_password
                        set_value=set_new_password
                        placeholder="••••••••"
                        type_="password"
                        label=move || translate("security.newPasswordLabel")
                    />
                    <p class="text-sm text-muted-foreground">
                        {move || translate("security.passwordHint")}
                    </p>
                    <ui_button
                        on_click=on_change_password
                        class="w-full"
                        disabled=Signal::derive(move || form_state.get().is_submitting)
                    >
                        {move || {
                            if form_state.get().is_submitting {
                                translate("common.updating").to_string()
                            } else {
                                translate("security.passwordSubmit").to_string()
                            }
                        }}
                    </ui_button>
                    <Show when=move || form_state.get().form_error.is_some()>
                        <div class="rounded-md bg-destructive/10 border border-destructive/20 px-4 py-2 text-sm text-destructive">
                            {move || form_state.get().form_error.unwrap_or_default()}
                        </div>
                    </Show>
                    <Show when=move || success_message.get().is_some()>
                        <div class="rounded-md bg-emerald-100 border border-emerald-200 px-4 py-2 text-sm text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-400">
                            {move || success_message.get().unwrap_or_default()}
                        </div>
                    </Show>
                </div>

                <div class="grid gap-4 rounded-xl border border-border bg-card p-6 shadow-sm">
                    <h3 class="text-lg font-semibold text-card-foreground">
                        {move || translate("security.sessionsTitle")}
                    </h3>
                    <p class="text-sm text-muted-foreground">
                        {move || translate("security.sessionsSubtitle")}
                    </p>
                    <div class="rounded-lg bg-muted px-4 py-8 text-center text-sm text-muted-foreground">
                        "Session management via GraphQL — coming soon"
                    </div>
                </div>
            </div>
        </section>
    }
}
