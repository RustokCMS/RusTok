use leptos::prelude::*;

use super::model::{UserRole, UserStatus};

#[component]
pub fn UserAvatar(
    #[prop(optional, into)] name: Option<String>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let initials = name
        .as_deref()
        .and_then(|n| n.chars().next())
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "U".to_string());

    view! {
        <div class=format!(
            "flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-indigo-100 {}",
            class
        )>
            <span class="text-indigo-700 text-sm font-semibold">{initials}</span>
        </div>
    }
}

#[component]
pub fn UserRoleBadge(role: UserRole) -> impl IntoView {
    let (label, class) = match role {
        UserRole::SuperAdmin => ("Super Admin", "bg-purple-100 text-purple-700"),
        UserRole::Admin => ("Admin", "bg-indigo-100 text-indigo-700"),
        UserRole::Manager => ("Manager", "bg-blue-100 text-blue-700"),
        UserRole::Customer => ("Customer", "bg-slate-100 text-slate-700"),
        UserRole::Unknown => ("Unknown", "bg-gray-100 text-gray-500"),
    };

    view! {
        <span class=format!(
            "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {}",
            class
        )>
            {label}
        </span>
    }
}

#[component]
pub fn UserStatusBadge(status: UserStatus) -> impl IntoView {
    let (label, class) = match status {
        UserStatus::Active => ("Active", "bg-emerald-100 text-emerald-700"),
        UserStatus::Inactive => ("Inactive", "bg-slate-100 text-slate-500"),
        UserStatus::Suspended => ("Suspended", "bg-rose-100 text-rose-700"),
        UserStatus::Unknown => ("Unknown", "bg-gray-100 text-gray-500"),
    };

    view! {
        <span class=format!(
            "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {}",
            class
        )>
            {label}
        </span>
    }
}
