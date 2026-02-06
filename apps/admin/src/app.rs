use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::hooks::use_navigate;
use leptos_router::path;

use crate::pages::{
    dashboard::Dashboard, login::Login, not_found::NotFound, profile::Profile, register::Register,
    reset::ResetPassword, security::Security, user_details::UserDetails, users::Users,
};
use crate::providers::auth::{provide_auth_context, use_auth};
use crate::providers::locale::provide_locale_context;

#[component]
pub fn App() -> impl IntoView {
    provide_auth_context();
    provide_locale_context();

    view! {
        <Router>
            <main class="min-h-screen bg-slate-100 text-slate-900 font-sans">
                <script src="https://cdn.tailwindcss.com"></script>
                <Routes fallback=|| view! { <NotFound /> }>
                    <Route path=path!("/login") view=Login />
                    <Route path=path!("/register") view=Register />
                    <Route path=path!("/reset") view=ResetPassword />
                    <Route path=path!("/dashboard") view=ProtectedDashboard />
                    <Route path=path!("/profile") view=ProtectedProfile />
                    <Route path=path!("/security") view=ProtectedSecurity />
                    <Route path=path!("/users") view=ProtectedUsers />
                    <Route path=path!("/users/:id") view=ProtectedUserDetails />
                    <Route path=path!("") view=ProtectedDashboard />
                    <Route path=path!("/*") view=NotFound />
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn ProtectedDashboard() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    Effect::new(move |_| {
        if auth.token.get().is_none() {
            navigate("/login", Default::default());
        }
    });

    view! {
        <Show
            when=move || auth.token.get().is_some()
            fallback=|| view! { <Login /> }
        >
            <Dashboard />
        </Show>
    }
}

#[component]
fn ProtectedUsers() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    Effect::new(move |_| {
        if auth.token.get().is_none() {
            navigate("/login", Default::default());
        }
    });

    view! {
        <Show
            when=move || auth.token.get().is_some()
            fallback=|| view! { <Login /> }
        >
            <Users />
        </Show>
    }
}

#[component]
fn ProtectedUserDetails() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    Effect::new(move |_| {
        if auth.token.get().is_none() {
            navigate("/login", Default::default());
        }
    });

    view! {
        <Show
            when=move || auth.token.get().is_some()
            fallback=|| view! { <Login /> }
        >
            <UserDetails />
        </Show>
    }
}

#[component]
fn ProtectedProfile() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    Effect::new(move |_| {
        if auth.token.get().is_none() {
            navigate("/login", Default::default());
        }
    });

    view! {
        <Show
            when=move || auth.token.get().is_some()
            fallback=|| view! { <Login /> }
        >
            <Profile />
        </Show>
    }
}

#[component]
fn ProtectedSecurity() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    Effect::new(move |_| {
        if auth.token.get().is_none() {
            navigate("/login", Default::default());
        }
    });

    view! {
        <Show
            when=move || auth.token.get().is_some()
            fallback=|| view! { <Login /> }
        >
            <Security />
        </Show>
    }
}
