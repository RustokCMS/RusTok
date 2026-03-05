use leptos::prelude::*;
use leptos_auth::components::ProtectedRoute;
use leptos_auth::context::AuthProvider;
use leptos_router::components::{ParentRoute, Route, Router, Routes};
use leptos_router::path;

use crate::pages::{
    dashboard::dashboard, login::login, modules::modules, not_found::not_found, profile::profile,
    register::register, reset::reset_password, security::security, user_details::user_details,
    users::users,
};
use crate::widgets::app_shell::app_layout;

use super::providers::locale::provide_locale_context;

#[component]
pub fn app() -> impl IntoView {
    provide_locale_context();

    view! {
        <AuthProvider>
            <Router>
                <Routes fallback=|| view! { {not_found()} }>
                    <Route path=path!("/login") view=login />
                    <Route path=path!("/register") view=register />
                    <Route path=path!("/reset") view=reset_password />

                    <ParentRoute path=path!("") view=ProtectedRoute>
                        <ParentRoute path=path!("") view=app_layout>
                            <Route path=path!("/dashboard") view=dashboard />
                            <Route path=path!("/profile") view=profile />
                            <Route path=path!("/security") view=security />
                            <Route path=path!("/modules") view=modules />
                            <Route path=path!("/users") view=users />
                            <Route path=path!("/users/:id") view=user_details />
                            <Route path=path!("") view=dashboard />
                        </ParentRoute>
                    </ParentRoute>

                    <Route path=path!("/*") view=not_found />
                </Routes>
            </Router>
        </AuthProvider>
    }
}
