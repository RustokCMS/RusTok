use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

use crate::features::auth::UserMenu;
use crate::shared::ui::LanguageToggle;
use crate::shared::i18n::translate;

#[derive(Clone, Copy, PartialEq)]
struct Breadcrumb {
    label_key: &'static str,
    href: Option<&'static str>,
}

#[component]
pub fn Header() -> impl IntoView {
    let location = use_location();

    let breadcrumbs = Memo::new(move |_| resolve_breadcrumbs(&location.pathname.get()));
    let title_key = Memo::new(move |_| resolve_title_key(&location.pathname.get()));

    Effect::new(move |_| {
        let title = format!("{} — {}", translate("app.brand.title"), translate(title_key.get()));
        set_document_title(&title);
    });

    view! {
        <header class="h-14 bg-background border-b border-border flex items-center justify-between px-6 shrink-0">
            <div class="flex items-center gap-2 text-sm text-muted-foreground">
                <span class="font-medium text-foreground">{translate("app.brand.short")}</span>
                <span>"/"</span>
                {{
                    let crumbs = breadcrumbs.get();
                    let last_index = crumbs.len().saturating_sub(1);
                    crumbs
                        .into_iter()
                        .enumerate()
                        .map(|(index, crumb)| {
                            let label_key = crumb.label_key;
                            let is_last = index == last_index;
                            let content = if let Some(href) = crumb.href {
                                view! { <A href=href attr:class="hover:text-foreground transition-colors">{move || translate(label_key)}</A> }.into_any()
                            } else {
                                view! { <span class="text-foreground">{move || translate(label_key)}</span> }.into_any()
                            };
                            view! {
                                <div class="flex items-center gap-2">
                                    {content}
                                    <Show when=move || !is_last>
                                        <span>"/"</span>
                                    </Show>
                                </div>
                            }
                        })
                        .collect_view()
                }}
            </div>

            <div class="flex items-center gap-3">
                <LanguageToggle />
                <UserMenu />
            </div>
        </header>
    }
}

fn resolve_breadcrumbs(pathname: &str) -> Vec<Breadcrumb> {
    match pathname {
        "/" | "/dashboard" => vec![Breadcrumb {
            label_key: "app.nav.dashboard",
            href: Some("/dashboard"),
        }],
        "/users" => vec![Breadcrumb {
            label_key: "app.nav.users",
            href: Some("/users"),
        }],
        "/profile" => vec![Breadcrumb {
            label_key: "app.nav.profile",
            href: Some("/profile"),
        }],
        "/security" => vec![Breadcrumb {
            label_key: "app.nav.security",
            href: Some("/security"),
        }],
        _ if pathname.starts_with("/users/") => vec![
            Breadcrumb {
                label_key: "app.nav.users",
                href: Some("/users"),
            },
            Breadcrumb {
                label_key: "users.detail.title",
                href: None,
            },
        ],
        _ => vec![Breadcrumb {
            label_key: "app.nav.dashboard",
            href: Some("/dashboard"),
        }],
    }
}

fn resolve_title_key(pathname: &str) -> &'static str {
    match pathname {
        "/" | "/dashboard" => "app.nav.dashboard",
        "/users" => "app.nav.users",
        "/profile" => "app.nav.profile",
        "/security" => "app.nav.security",
        _ if pathname.starts_with("/users/") => "users.detail.title",
        _ => "app.nav.dashboard",
    }
}

fn set_document_title(_title: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                document.set_title(_title);
            }
        }
    }
}
