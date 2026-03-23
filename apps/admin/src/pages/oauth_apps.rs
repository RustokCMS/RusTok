use crate::entities::oauth_app::model::OAuthApp;
use crate::features::oauth_apps::api::list_oauth_apps;
use crate::features::oauth_apps::create_app::{CreateAppForm, CreateResult};
use crate::features::oauth_apps::edit_app::EditAppForm;
use crate::features::oauth_apps::revoke_app::RevokeAppDialog;
use crate::features::oauth_apps::rotate_secret::RotateSecretDialog;
use crate::shared::ui::Button;
use crate::widgets::oauth_apps_list::OAuthAppsList;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_auth::hooks::{use_tenant, use_token};

#[derive(Clone, PartialEq)]
enum ModalState {
    None,
    CreateApp,
    EditApp(OAuthApp),
    RotateSecret(OAuthApp),
    RevokeApp(OAuthApp),
    SecretRevealed { secret: String, app: OAuthApp },
}

#[component]
pub fn OAuthAppsPage() -> impl IntoView {
    let token = use_token();
    let tenant = use_tenant();

    let (apps, set_apps) = signal(Vec::<OAuthApp>::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (refresh_counter, set_refresh_counter) = signal(0u32);
    let (modal_state, set_modal_state) = signal(ModalState::None);

    Effect::new(move |_| {
        let _ = refresh_counter.get();
        let token_value = token.get();
        let tenant_value = tenant.get();

        set_loading.set(true);
        set_error.set(None);

        spawn_local(async move {
            match list_oauth_apps(token_value, tenant_value).await {
                Ok(next_apps) => {
                    set_apps.set(next_apps);
                    set_loading.set(false);
                }
                Err(err) => {
                    set_error.set(Some(err.to_string()));
                    set_loading.set(false);
                }
            }
        });
    });

    let on_edit = Callback::new(move |app| set_modal_state.set(ModalState::EditApp(app)));
    let on_rotate = Callback::new(move |app| set_modal_state.set(ModalState::RotateSecret(app)));
    let on_revoke = Callback::new(move |app| set_modal_state.set(ModalState::RevokeApp(app)));

    let close_modal = move || set_modal_state.set(ModalState::None);

    view! {
        <div class="space-y-6">
            <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                <div>
                    <h2 class="text-2xl font-bold tracking-tight">"OAuth App Connections"</h2>
                    <p class="text-muted-foreground">
                        "Manage manual integrations, inspect manifest-managed frontends, and rotate client credentials."
                    </p>
                </div>
                <Button on_click=Callback::new(move |_| set_modal_state.set(ModalState::CreateApp))>
                    "Create New App"
                </Button>
            </div>

            <Show when=move || error.get().is_some()>
                <div class="rounded-md border border-destructive/40 bg-destructive/5 px-4 py-3 text-sm text-destructive">
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            <OAuthAppsList
                apps=apps.get()
                loading=loading.get()
                on_edit_app=on_edit
                on_rotate_secret=on_rotate
                on_revoke_app=on_revoke
            />

            <Show when=move || modal_state.get() != ModalState::None>
                <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 px-4 backdrop-blur-sm">
                    <div class="w-full max-w-2xl rounded-lg border bg-background p-6 shadow-lg">
                        {move || match modal_state.get() {
                            ModalState::CreateApp => {
                                let close = close_modal;
                                let token_value = token.get();
                                let tenant_value = tenant.get();
                                view! {
                                    <CreateAppForm
                                        token=token_value
                                        tenant=tenant_value
                                        on_success=move |result: CreateResult| {
                                            set_refresh_counter.update(|value| *value += 1);
                                            set_modal_state.set(ModalState::SecretRevealed {
                                                secret: result.client_secret,
                                                app: result.app,
                                            });
                                        }
                                        on_cancel=move || close()
                                    />
                                }
                                .into_any()
                            }
                            ModalState::EditApp(app) => {
                                let close = close_modal;
                                let token_value = token.get();
                                let tenant_value = tenant.get();
                                view! {
                                    <EditAppForm
                                        token=token_value
                                        tenant=tenant_value
                                        app=app
                                        on_success=move |_| {
                                            set_refresh_counter.update(|value| *value += 1);
                                            close();
                                        }
                                        on_cancel=move || close()
                                    />
                                }
                                .into_any()
                            }
                            ModalState::RotateSecret(app) => {
                                let close = close_modal;
                                let token_value = token.get();
                                let tenant_value = tenant.get();
                                view! {
                                    <RotateSecretDialog
                                        token=token_value
                                        tenant=tenant_value
                                        app=app
                                        on_success=move |new_secret, updated_app| {
                                            set_refresh_counter.update(|value| *value += 1);
                                            set_modal_state.set(ModalState::SecretRevealed {
                                                secret: new_secret,
                                                app: updated_app,
                                            });
                                        }
                                        on_cancel=move || close()
                                    />
                                }
                                .into_any()
                            }
                            ModalState::RevokeApp(app) => {
                                let close_for_success = close_modal;
                                let close_for_cancel = close_modal;
                                let token_value = token.get();
                                let tenant_value = tenant.get();
                                view! {
                                    <RevokeAppDialog
                                        token=token_value
                                        tenant=tenant_value
                                        app=app
                                        on_success=move || {
                                            set_refresh_counter.update(|value| *value += 1);
                                            close_for_success();
                                        }
                                        on_cancel=move || close_for_cancel()
                                    />
                                }
                                .into_any()
                            }
                            ModalState::SecretRevealed { secret, app } => {
                                let close = close_modal;
                                let title = if app.auto_created {
                                    "Client secret rotated."
                                } else {
                                    "Client secret generated."
                                };

                                view! {
                                    <div class="space-y-4">
                                        <h3 class="text-lg font-medium text-green-600">{title}</h3>
                                        <p class="text-sm">
                                            "Store this secret safely. It will not be shown again."
                                        </p>

                                        <div class="break-all rounded border bg-slate-100 p-3 font-mono text-sm">
                                            {secret}
                                        </div>

                                        <Button class="w-full" on_click=Callback::new(move |_| close())>
                                            "I have saved it"
                                        </Button>
                                    </div>
                                }
                                .into_any()
                            }
                            ModalState::None => view! { <div></div> }.into_any(),
                        }}
                    </div>
                </div>
            </Show>
        </div>
    }
}
