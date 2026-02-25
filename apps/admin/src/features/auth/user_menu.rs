use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;

use leptos_auth::hooks::{use_auth, use_current_user};

#[component]
pub fn UserMenu() -> impl IntoView {
    let auth = use_auth();
    let current_user = use_current_user();

    let (open, set_open) = signal(false);

    let handle_logout = Callback::new(move |_| {
        let auth = auth.clone();
        spawn_local(async move {
            let _ = auth.sign_out().await;
        });
    });

    let toggle_menu = move |_| {
        set_open.update(|v| *v = !*v);
    };

    view! {
        <div class="relative">
            <button
                on:click=toggle_menu
                class="flex items-center gap-2 p-2 hover:bg-gray-100 rounded-lg transition-colors"
            >
                <div class="w-8 h-8 bg-gradient-to-br from-blue-500 to-purple-600 rounded-full flex items-center justify-center">
                    <span class="text-white text-sm font-semibold">
                        {move || {
                            current_user
                                .get()
                                .and_then(|u| u.name.clone())
                                .and_then(|n| n.chars().next())
                                .map(|c| c.to_string())
                                .unwrap_or_else(|| "U".to_string())
                        }}
                    </span>
                </div>
                <div class="text-left hidden md:block">
                    <p class="text-sm font-medium text-gray-900">
                        {move || {
                            current_user
                                .get()
                                .and_then(|u| u.name.clone())
                                .unwrap_or_else(|| "User".to_string())
                        }}
                    </p>
                    <p class="text-xs text-gray-500">
                        {move || {
                            current_user
                                .get()
                                .map(|u| u.role.clone())
                                .unwrap_or_else(|| "user".to_string())
                        }}
                    </p>
                </div>
                <span class="text-gray-400 text-sm">
                    {move || if open.get() { "▲" } else { "▼" }}
                </span>
            </button>

            <Show when=move || open.get()>
                <div class="absolute right-0 mt-2 w-56 bg-white rounded-lg shadow-lg border border-gray-200 py-1 z-50">
                    <div class="px-4 py-3 border-b border-gray-200">
                        <p class="text-sm font-medium text-gray-900">
                            {move || {
                                current_user
                                    .get()
                                    .and_then(|u| u.name.clone())
                                    .unwrap_or_else(|| "User".to_string())
                            }}
                        </p>
                        <p class="text-xs text-gray-500 truncate">
                            {move || {
                                current_user
                                    .get()
                                    .map(|u| u.email.clone())
                                    .unwrap_or_else(|| "user@example.com".to_string())
                            }}
                        </p>
                    </div>

                    <div class="py-1">
                        <DropdownLink href="/profile" icon="👤">
                            "Profile"
                        </DropdownLink>
                        <DropdownLink href="/security" icon="🔒">
                            "Security"
                        </DropdownLink>
                    </div>

                    <div class="border-t border-gray-200 py-1">
                        <button
                            on:click=move |ev| handle_logout.run(ev)
                            class="w-full flex items-center gap-3 px-4 py-2 text-sm text-red-600 hover:bg-red-50 transition-colors"
                        >
                            <span>"🚪"</span>
                            <span>"Sign Out"</span>
                        </button>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn DropdownLink(href: &'static str, icon: &'static str, children: Children) -> impl IntoView {
    view! {
        <A
            href=href
            attr:class="flex items-center gap-3 px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 transition-colors"
        >
            <span>{icon}</span>
            <span>{children()}</span>
        </A>
    }
}
