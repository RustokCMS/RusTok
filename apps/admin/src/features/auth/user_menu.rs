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
                class="flex items-center gap-2 p-2 hover:bg-accent rounded-lg transition-colors"
            >
                <div class="w-8 h-8 bg-primary rounded-full flex items-center justify-center">
                    <span class="text-primary-foreground text-sm font-semibold">
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
                    <p class="text-sm font-medium text-foreground">
                        {move || {
                            current_user
                                .get()
                                .and_then(|u| u.name.clone())
                                .unwrap_or_else(|| "User".to_string())
                        }}
                    </p>
                    <p class="text-xs text-muted-foreground">
                        {move || {
                            current_user
                                .get()
                                .map(|u| u.role.clone())
                                .unwrap_or_else(|| "user".to_string())
                        }}
                    </p>
                </div>
                <span class="text-muted-foreground text-sm">
                    {move || if open.get() { "▲" } else { "▼" }}
                </span>
            </button>

            <Show when=move || open.get()>
                <div class="absolute right-0 mt-2 w-56 bg-popover rounded-lg shadow-md border border-border py-1 z-50">
                    <div class="px-4 py-3 border-b border-border">
                        <p class="text-sm font-medium text-popover-foreground">
                            {move || {
                                current_user
                                    .get()
                                    .and_then(|u| u.name.clone())
                                    .unwrap_or_else(|| "User".to_string())
                            }}
                        </p>
                        <p class="text-xs text-muted-foreground truncate">
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

                    <div class="border-t border-border py-1">
                        <button
                            on:click=move |ev| handle_logout.run(ev)
                            class="w-full flex items-center gap-3 px-4 py-2 text-sm text-destructive hover:bg-destructive/10 transition-colors"
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
            attr:class="flex items-center gap-3 px-4 py-2 text-sm text-popover-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
        >
            <span>{icon}</span>
            <span>{children()}</span>
        </A>
    }
}
