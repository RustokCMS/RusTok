// Sidebar Navigation Component
use leptos::prelude::*;
use leptos_auth::hooks::use_current_user;
use leptos_router::components::A;

use crate::providers::locale::translate;

#[component]
pub fn Sidebar() -> impl IntoView {
    let current_user = use_current_user();

    view! {
        <aside class="w-64 bg-white border-r border-slate-200 h-screen flex flex-col shrink-0">
            // Logo & Brand
            <div class="p-5 border-b border-slate-200">
                <A href="/dashboard" class="flex items-center gap-2">
                    <div class="flex h-8 w-8 items-center justify-center rounded-lg bg-indigo-600">
                        <span class="text-white font-bold text-sm">"R"</span>
                    </div>
                    <span class="text-sm font-semibold text-slate-900">"RusTok Admin"</span>
                </A>
            </div>

            // Navigation
            <nav class="flex-1 px-3 py-4 overflow-y-auto">
                <NavGroupLabel label="Overview" />
                <NavLink href="/dashboard" icon="grid">
                    {move || translate("app.nav.dashboard")}
                </NavLink>

                <div class="pt-3">
                    <NavGroupLabel label="Management" />
                    <NavLink href="/users" icon="users">
                        {move || translate("app.nav.users")}
                    </NavLink>
                </div>

                <div class="pt-3">
                    <NavGroupLabel label="Account" />
                    <NavLink href="/profile" icon="user">
                        {move || translate("app.nav.profile")}
                    </NavLink>
                    <NavLink href="/security" icon="lock">
                        {move || translate("app.nav.security")}
                    </NavLink>
                </div>
            </nav>

            // User footer
            <div class="p-4 border-t border-slate-200">
                <div class="flex items-center gap-2">
                    <div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-indigo-100">
                        <span class="text-indigo-700 text-sm font-semibold">
                            {move || {
                                current_user.get()
                                    .and_then(|u| u.name.clone().or(Some(u.email.clone())))
                                    .and_then(|n| n.chars().next())
                                    .map(|c| c.to_uppercase().to_string())
                                    .unwrap_or_else(|| "U".to_string())
                            }}
                        </span>
                    </div>
                    <div class="grid flex-1 min-w-0 text-left text-sm leading-tight">
                        <span class="truncate font-semibold text-slate-900 text-xs">
                            {move || current_user.get().and_then(|u| u.name.clone()).unwrap_or_else(|| "User".to_string())}
                        </span>
                        <span class="truncate text-xs text-slate-500">
                            {move || current_user.get().map(|u| u.email.clone()).unwrap_or_default()}
                        </span>
                    </div>
                </div>
            </div>
        </aside>
    }
}

#[component]
fn NavGroupLabel(label: &'static str) -> impl IntoView {
    view! {
        <div class="px-3 pb-1 text-xs font-semibold text-slate-500 uppercase tracking-wider">
            {label}
        </div>
    }
}

#[component]
fn NavLink(href: &'static str, icon: &'static str, children: Children) -> impl IntoView {
    view! {
        <A
            href=href
            class="flex items-center gap-3 px-3 py-2 text-sm font-medium text-slate-700 rounded-lg hover:bg-slate-100 hover:text-slate-900 transition-colors mb-1"
            active_class="bg-indigo-50 text-indigo-700"
        >
            <NavIcon icon=icon />
            <span>{children()}</span>
        </A>
    }
}

#[component]
fn NavIcon(icon: &'static str) -> impl IntoView {
    let svg_path = match icon {
        "grid" => "M4 4h6v6H4V4zm0 10h6v6H4v-6zm10-10h6v6h-6V4zm0 10h6v6h-6v-6z",
        "users" => "M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2M23 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8z",
        "user" => "M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2M12 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8z",
        "lock" => "M19 11H5a2 2 0 0 0-2 2v7a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7a2 2 0 0 0-2-2zM7 11V7a5 5 0 0 1 10 0v4",
        _ => "M12 12m-10 0a10 10 0 1 0 20 0a10 10 0 1 0-20 0",
    };

    view! {
        <svg
            class="h-4 w-4 shrink-0"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <path d=svg_path />
        </svg>
    }
}
