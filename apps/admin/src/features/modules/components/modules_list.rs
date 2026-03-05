use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_auth::hooks::{use_tenant, use_token};

use crate::entities::module::ModuleInfo;
use crate::features::modules::api;
use crate::shared::ui::ui_success_message;
use leptos_hook_form::FormState;

#[component]
pub fn modules_list(modules: Vec<ModuleInfo>) -> impl IntoView {
    let (module_list, set_module_list) = signal(modules);
    let (loading_slug, set_loading_slug) = signal::<Option<String>>(None);
    let (form_state, set_form_state) = signal(FormState::idle());
    let (success_message, set_success_message) = signal::<Option<String>>(None);

    let token = use_token();
    let tenant = use_tenant();

    let on_toggle = Callback::new(move |(slug, enabled): (String, bool)| {
        let slug_clone = slug.clone();
        set_loading_slug.set(Some(slug.clone()));
        set_form_state.set(FormState::idle());
        set_success_message.set(None);

        let token_val = token.get();
        let tenant_val = tenant.get();

        spawn_local(async move {
            set_form_state.set(FormState::submitting());

            match api::toggle_module(slug_clone.clone(), enabled, token_val, tenant_val).await {
                Ok(result) => {
                    set_module_list.update(|modules| {
                        if let Some(m) = modules.iter_mut().find(|m| m.module_slug == slug_clone) {
                            m.enabled = result.enabled;
                        }
                    });
                    let status = if result.enabled {
                        translate("modules.toast.enabled")
                    } else {
                        translate("modules.toast.disabled")
                    };
                    set_success_message.set(Some(status));
                    set_form_state.set(FormState::idle());
                }
                Err(err) => {
                    set_form_state.set(FormState::with_form_error(format!("{}", err)));
                }
            }
            set_loading_slug.set(None);
        });
    });

    let core_modules = move || {
        module_list
            .get()
            .into_iter()
            .filter(|m| m.is_core())
            .collect::<Vec<_>>()
    };

    let optional_modules = move || {
        module_list
            .get()
            .into_iter()
            .filter(|m| !m.is_core())
            .collect::<Vec<_>>()
    };

    view! {
        <div class="space-y-8">
            <Show when=move || form_state.get().form_error.is_some()>
                <div class="rounded-lg border border-destructive/50 bg-destructive/10 px-4 py-3 text-sm text-destructive">
                    {move || form_state.get().form_error.unwrap_or_default()}
                </div>
            </Show>

            <Show when=move || success_message.get().is_some()>
                <ui_success_message message=success_message.get().unwrap_or_default() />
            </Show>

            // Core modules
            <div class="space-y-3">
                <div class="flex items-center gap-2">
                    <svg class="h-5 w-5 text-muted-foreground" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
                    </svg>
                    <h3 class="text-lg font-semibold text-foreground">{translate("modules.section.core")}</h3>
                    <span class="inline-flex items-center rounded-full bg-secondary px-2.5 py-0.5 text-xs font-semibold text-secondary-foreground">
                        {translate("modules.always_active")}
                    </span>
                </div>
                <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                    {move || core_modules().into_iter().map(|module| {
                        let slug = module.module_slug.clone();
                        let is_loading = Signal::derive(move || loading_slug.get().as_deref() == Some(&slug));
                        view! {
                            {module_card(
                                module,
                                is_loading,
                                None,
                            )}
                        }
                    }).collect_view()}
                </div>
            </div>

            // Optional modules
            <div class="space-y-3">
                <div class="flex items-center gap-2">
                    <svg class="h-5 w-5 text-muted-foreground" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
                    </svg>
                    <h3 class="text-lg font-semibold text-foreground">{translate("modules.section.optional")}</h3>
                </div>
                <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                    {move || optional_modules().into_iter().map(|module| {
                        let slug = module.module_slug.clone();
                        let is_loading = Signal::derive(move || loading_slug.get().as_deref() == Some(&slug));
                        view! {
                            {module_card(
                                module,
                                is_loading,
                                Some(on_toggle),
                            )}
                        }
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}
