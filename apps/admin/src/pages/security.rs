use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_auth::hooks::{use_auth, use_tenant, use_token};
use leptos_hook_form::FormState;
use serde::{Deserialize, Serialize};

use crate::shared::api::{request, ApiError};
use crate::shared::ui::{Button, Input, PageHeader};
use crate::{t_string, use_i18n};

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

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ChangePasswordResponse {
    #[serde(rename = "changePassword")]
    _change_password: SuccessPayload,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct SuccessPayload {
    #[allow(dead_code)]
    success: bool,
}

#[server(prefix = "/api/fn", endpoint = "admin/change-password")]
async fn change_password_native(
    token: String,
    tenant: String,
    current_password: String,
    new_password: String,
) -> Result<SuccessPayload, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};

        #[derive(Deserialize)]
        struct RestStatusResponse {
            #[allow(dead_code)]
            status: String,
        }

        let client = reqwest::Client::new();
        let response = client
            .post(format!(
                "{}/api/auth/change-password",
                crate::shared::api::api_base_url()
            ))
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .header(CONTENT_TYPE, "application/json")
            .header("X-Tenant-ID", tenant)
            .json(&serde_json::json!({
                "current_password": current_password,
                "new_password": new_password,
            }))
            .send()
            .await
            .map_err(ServerFnError::new)?;

        if !response.status().is_success() {
            return Err(ServerFnError::new(
                crate::shared::api::extract_http_error(response).await,
            ));
        }

        let _ = response
            .json::<RestStatusResponse>()
            .await
            .map_err(ServerFnError::new)?;

        Ok(SuccessPayload { success: true })
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (token, tenant, current_password, new_password);
        Err(ServerFnError::new(
            "admin/change-password requires the `ssr` feature",
        ))
    }
}

#[component]
pub fn Security() -> impl IntoView {
    let i18n = use_i18n();
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
                t_string!(i18n, security.passwordRequired).to_string(),
            ));
            return;
        }

        let token_value = token.get();
        let tenant_value = tenant.get();
        if token_value.is_none() {
            set_form_state.set(FormState::with_form_error(
                t_string!(i18n, errors.auth.unauthorized).to_string(),
            ));
            return;
        }

        let current_password_value = current_password.get();
        let new_password_value = new_password.get();

        set_form_state.set(FormState::submitting());
        set_success_message.set(None);

        spawn_local(async move {
            let native_result = change_password_native(
                token_value.clone().unwrap_or_default(),
                tenant_value.clone().unwrap_or_default(),
                current_password_value.clone(),
                new_password_value.clone(),
            )
            .await;

            let result = match native_result {
                Ok(payload) => Ok(ChangePasswordResponse {
                    _change_password: payload,
                }),
                Err(_) => {
                    request::<ChangePasswordVariables, ChangePasswordResponse>(
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
                    .await
                }
            };

            match result {
                Ok(_) => {
                    set_form_state.set(FormState::idle());
                    set_success_message
                        .set(Some(t_string!(i18n, security.passwordUpdated).to_string()));
                    set_current_password.set(String::new());
                    set_new_password.set(String::new());
                }
                Err(err) => {
                    let message = match err {
                        ApiError::Unauthorized => {
                            t_string!(i18n, errors.auth.unauthorized).to_string()
                        }
                        ApiError::Http(_) => t_string!(i18n, errors.http).to_string(),
                        ApiError::Network => t_string!(i18n, errors.network).to_string(),
                        ApiError::Graphql(_) => t_string!(i18n, errors.unknown).to_string(),
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
        <section class="flex flex-1 flex-col p-4 md:px-6">
            <PageHeader
                title=move || t_string!(i18n, security.title).to_string()
                subtitle=move || t_string!(i18n, security.subtitle).to_string()
                eyebrow=move || t_string!(i18n, security.badge).to_string()
                actions=view! {
                    <Button
                        on_click=on_sign_out_all
                        class="border border-border bg-transparent text-foreground hover:bg-accent hover:text-accent-foreground"
                    >
                        {move || t_string!(i18n, security.signOutAll)}
                    </Button>
                }
                .into_any()
            />

            <div class="grid gap-6 lg:grid-cols-2">
                <div class="grid gap-4 rounded-xl border border-border bg-card p-6 shadow-sm">
                    <h3 class="text-lg font-semibold text-card-foreground">
                        {move || t_string!(i18n, security.passwordTitle)}
                    </h3>
                    <p class="text-sm text-muted-foreground">
                        {move || t_string!(i18n, security.passwordSubtitle)}
                    </p>
                     <Input
                        value=current_password
                        set_value=set_current_password
                        placeholder="••••••••"
                        type_="password"
                        label=move || t_string!(i18n, security.currentPasswordLabel)
                    />
                    <Input
                        value=new_password
                        set_value=set_new_password
                        placeholder="••••••••"
                        type_="password"
                        label=move || t_string!(i18n, security.newPasswordLabel)
                    />
                    <p class="text-sm text-muted-foreground">
                        {move || t_string!(i18n, security.passwordHint)}
                    </p>
                    <Button on_click=on_change_password class="w-full">
                        {move || t_string!(i18n, security.passwordSubmit)}
                    </Button>
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
                        {move || t_string!(i18n, security.sessionsTitle)}
                    </h3>
                    <p class="text-sm text-muted-foreground">
                        {move || t_string!(i18n, security.sessionsSubtitle)}
                    </p>
                    <div class="rounded-lg bg-muted px-4 py-8 text-center text-sm text-muted-foreground">
                        "Session management via GraphQL — coming soon"
                    </div>
                </div>
            </div>
        </section>
    }
}
