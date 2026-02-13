use leptos::*;
use leptos_router::*;

use crate::hooks::{use_is_authenticated, use_is_loading};

#[component]
pub fn ProtectedRoute(
    children: Children,
    #[prop(optional)] redirect_path: Option<String>,
) -> impl IntoView {
    let is_authenticated = use_is_authenticated();
    let is_loading = use_is_loading();
    let redirect_to = redirect_path.unwrap_or_else(|| "/login".to_string());
    let navigate = use_navigate();

    create_effect(move |_| {
        if !is_loading.get() && !is_authenticated.get() {
            navigate(&redirect_to, Default::default());
        }
    });

    view! {
        <Show
            when=move || is_authenticated.get()
            fallback=|| view! { <div class="flex items-center justify-center min-h-screen">
                <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-gray-900"></div>
            </div> }
        >
            {children()}
        </Show>
    }
}

#[component]
pub fn GuestRoute(
    children: Children,
    #[prop(optional)] redirect_path: Option<String>,
) -> impl IntoView {
    let is_authenticated = use_is_authenticated();
    let is_loading = use_is_loading();
    let redirect_to = redirect_path.unwrap_or_else(|| "/dashboard".to_string());
    let navigate = use_navigate();

    create_effect(move |_| {
        if !is_loading.get() && is_authenticated.get() {
            navigate(&redirect_to, Default::default());
        }
    });

    view! {
        <Show
            when=move || !is_authenticated.get()
            fallback=|| view! { <div class="flex items-center justify-center min-h-screen">
                <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-gray-900"></div>
            </div> }
        >
            {children()}
        </Show>
    }
}

#[component]
pub fn RequireAuth(
    children: Children,
    #[prop(optional)] fallback: Option<View>,
) -> impl IntoView {
    let is_authenticated = use_is_authenticated();

    view! {
        <Show
            when=move || is_authenticated.get()
            fallback=move || fallback.clone().unwrap_or_else(|| view! { <p>"Please sign in to view this content"</p> }.into_view())
        >
            {children()}
        </Show>
    }
}
