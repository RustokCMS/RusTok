use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use leptos::{component, view, IntoView};
use leptos::prelude::{CollectView, ClassAttribute, ElementChild, render_to_string};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct LocaleStrings {
    hero_title: &'static str,
    hero_subtitle: &'static str,
    cta_primary: &'static str,
    cta_secondary: &'static str,
    featured_title: &'static str,
    featured_subtitle: &'static str,
    story_title: &'static str,
    story_body: &'static str,
    newsletter_title: &'static str,
    newsletter_body: &'static str,
    newsletter_cta: &'static str,
    newsletter_placeholder: &'static str,
    newsletter_note: &'static str,
    cta_view: &'static str,
    nav_home: &'static str,
    nav_catalog: &'static str,
    nav_about: &'static str,
    nav_contact: &'static str,
    nav_language: &'static str,
    footer_tagline: &'static str,
    badge_new: &'static str,
    stat_ssr_title: &'static str,
    stat_ssr_value: &'static str,
    stat_ssr_desc: &'static str,
    stat_i18n_title: &'static str,
    stat_i18n_value: &'static str,
    stat_i18n_desc: &'static str,
    stat_modules_title: &'static str,
    stat_modules_value: &'static str,
    stat_modules_desc: &'static str,
}

#[derive(Clone, Debug)]
struct ProductCardData {
    title: &'static str,
    description: &'static str,
    price: &'static str,
    badge: Option<&'static str>,
}

fn locale_strings(locale: &str) -> LocaleStrings {
    match locale {
        "ru" => LocaleStrings {
            hero_title: "Витрина RusToK",
            hero_subtitle: "SSR-платформа на Rust для быстрой коммерции и красивых витрин.",
            cta_primary: "Открыть каталог",
            cta_secondary: "О платформе",
            featured_title: "Избранные коллекции",
            featured_subtitle: "Заполните карточки товарами из Commerce-модуля и настройте промо.",
            story_title: "Бренд, которому доверяют",
            story_body: "RusToK помогает собрать надежную витрину: события, поисковые индексы, многоязычность.",
            newsletter_title: "Подписка на обновления",
            newsletter_body: "Подключайте рассылки и промо-кампании прямо из ядра.",
            newsletter_cta: "Подписаться",
            newsletter_placeholder: "email@rustok.local",
            newsletter_note: "Никакого спама. Отписка в любой момент.",
            cta_view: "Смотреть",
            nav_home: "Главная",
            nav_catalog: "Каталог",
            nav_about: "О нас",
            nav_contact: "Контакты",
            nav_language: "Язык",
            footer_tagline: "RusToK — модульная платформа для современной коммерции.",
            badge_new: "Новинка",
            stat_ssr_title: "SSR",
            stat_ssr_value: "Быстро",
            stat_ssr_desc: "Первые миллисекунды важны",
            stat_i18n_title: "i18n",
            stat_i18n_value: "2+",
            stat_i18n_desc: "Готово для глобальных рынков",
            stat_modules_title: "Модули",
            stat_modules_value: "8",
            stat_modules_desc: "Коммерция, контент, поиск",
        },
        _ => LocaleStrings {
            hero_title: "RusToK Storefront",
            hero_subtitle: "SSR-first Rust commerce with fast discovery and clean design.",
            cta_primary: "Browse catalog",
            cta_secondary: "Platform overview",
            featured_title: "Featured collections",
            featured_subtitle: "Plug Commerce data in and curate promotions in minutes.",
            story_title: "A brand you can trust",
            story_body: "RusToK ships with events, search indexes, and multilingual support.",
            newsletter_title: "Stay in the loop",
            newsletter_body: "Connect newsletters and promotions from the core.",
            newsletter_cta: "Subscribe",
            newsletter_placeholder: "email@rustok.local",
            newsletter_note: "No spam. Opt-out anytime.",
            cta_view: "View",
            nav_home: "Home",
            nav_catalog: "Catalog",
            nav_about: "About",
            nav_contact: "Contact",
            nav_language: "Language",
            footer_tagline: "RusToK — the modular platform for modern commerce.",
            badge_new: "New",
            stat_ssr_title: "SSR",
            stat_ssr_value: "Fast",
            stat_ssr_desc: "First paint in milliseconds",
            stat_i18n_title: "i18n",
            stat_i18n_value: "2+",
            stat_i18n_desc: "Ready for global stores",
            stat_modules_title: "Modules",
            stat_modules_value: "8",
            stat_modules_desc: "Commerce, content, search",
        },
    }
}

fn featured_products(locale: &str) -> Vec<ProductCardData> {
    match locale {
        "ru" => vec![
            ProductCardData {
                title: "Смарт-аксессуары",
                description: "Функциональные гаджеты для повседневной жизни.",
                price: "от 4 990 ₽",
                badge: Some("Лидер продаж"),
            },
            ProductCardData {
                title: "Экологичная коллекция",
                description: "Натуральные материалы и премиальная отделка.",
                price: "от 2 490 ₽",
                badge: None,
            },
            ProductCardData {
                title: "Городская классика",
                description: "Минималистичные силуэты для любого сезона.",
                price: "от 6 500 ₽",
                badge: Some("Новинка"),
            },
        ],
        _ => vec![
            ProductCardData {
                title: "Smart accessories",
                description: "Functional gadgets for everyday life.",
                price: "from $89",
                badge: Some("Best seller"),
            },
            ProductCardData {
                title: "Eco collection",
                description: "Natural materials with premium finishing.",
                price: "from $49",
                badge: None,
            },
            ProductCardData {
                title: "City essentials",
                description: "Minimal silhouettes for every season.",
                price: "from $129",
                badge: Some("New"),
            },
        ],
    }
}

#[component]
fn ProductCard(
    product: ProductCardData,
    badge_new: &'static str,
    cta_view: &'static str,
) -> impl IntoView {
    let badge = product.badge.unwrap_or(badge_new);
    view! {
        <div class="card bg-base-100 shadow-xl">
            <figure class="bg-base-200">
                <div class="h-40 w-full bg-gradient-to-br from-primary/20 to-secondary/20" />
            </figure>
            <div class="card-body">
                <div class="flex items-start justify-between gap-2">
                    <h3 class="card-title">{product.title}</h3>
                    <span class="badge badge-secondary">{badge}</span>
                </div>
                <p class="opacity-70">{product.description}</p>
                <div class="card-actions items-center justify-between">
                    <span class="text-lg font-semibold">{product.price}</span>
                    <button class="btn btn-primary btn-sm">{cta_view}</button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn StorefrontShell(locale: String) -> impl IntoView {
    let strings = locale_strings(locale.as_str());
    let products = featured_products(locale.as_str());
    view! {
        <div class="min-h-screen bg-base-200 text-base-content">
            <header class="navbar bg-base-100 shadow">
                <div class="container-app flex w-full items-center">
                    <div class="navbar-start">
                        <a class="btn btn-ghost text-xl" href="/">{ "RusToK" }</a>
                    </div>
                    <div class="navbar-center hidden lg:flex">
                        <ul class="menu menu-horizontal gap-2">
                            <li><a href="#home">{strings.nav_home}</a></li>
                            <li><a href="#catalog">{strings.nav_catalog}</a></li>
                            <li><a href="#about">{strings.nav_about}</a></li>
                            <li><a href="#contact">{strings.nav_contact}</a></li>
                        </ul>
                    </div>
                    <div class="navbar-end">
                        <details class="dropdown dropdown-end">
                            <summary class="btn btn-ghost btn-sm">{strings.nav_language}</summary>
                            <ul class="menu dropdown-content z-[1] w-32 rounded-box bg-base-100 p-2 shadow">
                                <li><a href="/?lang=en">"English"</a></li>
                                <li><a href="/?lang=ru">"Русский"</a></li>
                            </ul>
                        </details>
                        <a class="btn btn-primary btn-sm ml-2" href="#catalog">{strings.cta_primary}</a>
                    </div>
                </div>
            </header>
            <main class="space-y-20">
                <section id="home" class="hero bg-base-100">
                    <div class="container-app hero-content flex-col gap-10 py-16 lg:flex-row lg:justify-between">
                        <div class="max-w-xl space-y-6">
                            <h1 class="text-4xl font-bold lg:text-5xl">{strings.hero_title}</h1>
                            <p class="text-lg opacity-80">{strings.hero_subtitle}</p>
                            <div class="flex flex-wrap gap-3">
                                <a class="btn btn-primary" href="#catalog">{strings.cta_primary}</a>
                                <a class="btn btn-ghost" href="#about">{strings.cta_secondary}</a>
                            </div>
                        </div>
                        <div class="stats stats-vertical bg-base-200 shadow lg:stats-horizontal">
                            <div class="stat">
                                <div class="stat-title">{strings.stat_ssr_title}</div>
                                <div class="stat-value">{strings.stat_ssr_value}</div>
                                <div class="stat-desc">{strings.stat_ssr_desc}</div>
                            </div>
                            <div class="stat">
                                <div class="stat-title">{strings.stat_i18n_title}</div>
                                <div class="stat-value">{strings.stat_i18n_value}</div>
                                <div class="stat-desc">{strings.stat_i18n_desc}</div>
                            </div>
                            <div class="stat">
                                <div class="stat-title">{strings.stat_modules_title}</div>
                                <div class="stat-value">{strings.stat_modules_value}</div>
                                <div class="stat-desc">{strings.stat_modules_desc}</div>
                            </div>
                        </div>
                    </div>
                </section>

                <section id="catalog" class="container-app">
                    <div class="mx-auto max-w-6xl space-y-6">
                        <div>
                            <h2 class="text-3xl font-bold">{strings.featured_title}</h2>
                            <p class="opacity-70">{strings.featured_subtitle}</p>
                        </div>
                        <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
                            {products
                                .into_iter()
                                .map(|product| {
                                    view! {
                                        <ProductCard
                                            product=product
                                            badge_new=strings.badge_new
                                            cta_view=strings.cta_view
                                        />
                                    }
                                })
                                .collect_view()}
                        </div>
                    </div>
                </section>

                <section id="about" class="container-app">
                    <div class="mx-auto grid max-w-6xl gap-6 lg:grid-cols-2">
                        <div class="card bg-base-100 shadow">
                            <div class="card-body space-y-3">
                                <h3 class="card-title">{strings.story_title}</h3>
                                <p class="opacity-70">{strings.story_body}</p>
                                <div class="flex gap-2">
                                    <span class="badge badge-outline">"GraphQL"</span>
                                    <span class="badge badge-outline">"Events"</span>
                                    <span class="badge badge-outline">"Search"</span>
                                </div>
                            </div>
                        </div>
                        <div class="card bg-base-100 shadow">
                            <div class="card-body space-y-4">
                                <h3 class="card-title">{strings.newsletter_title}</h3>
                                <p class="opacity-70">{strings.newsletter_body}</p>
                                <div class="join w-full">
                                    <input
                                        class="input join-item w-full"
                                        placeholder=strings.newsletter_placeholder
                                    />
                                    <button class="btn btn-secondary join-item">{strings.newsletter_cta}</button>
                                </div>
                                <span class="text-xs opacity-60">{strings.newsletter_note}</span>
                            </div>
                        </div>
                    </div>
                </section>
            </main>
            <footer id="contact" class="footer footer-center bg-base-300 p-8 text-base-content">
                <div class="space-y-2">
                    <p class="text-sm">{strings.footer_tagline}</p>
                    <div class="flex gap-4">
                        <span class="badge badge-primary">"SSR"</span>
                        <span class="badge badge-secondary">"Tailwind"</span>
                        <span class="badge badge-outline">"DaisyUI"</span>
                    </div>
                </div>
            </footer>
        </div>
    }
}

fn render_shell(locale: &str) -> String {
    let locale_owned = locale.to_string();
    let app_html = render_to_string(move || {
        let locale = locale_owned.clone();
        view! { <StorefrontShell locale=locale /> }
    });
    format!(
        r#"<!DOCTYPE html>
<html lang="{locale}" data-theme="rustok">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>RusToK Storefront</title>
  <link rel="stylesheet" href="/assets/app.css" />
  <script src="https://cdn.tailwindcss.com"></script>
  <link
    href="https://cdn.jsdelivr.net/npm/daisyui@4.12.10/dist/full.min.css"
    rel="stylesheet"
    type="text/css"
  />
</head>
<body class="bg-base-200">
  <div id="app">{app_html}</div>
</body>
</html>"#,
        locale = locale,
        app_html = app_html
    )
}

fn static_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static")
}

async fn css_handler() -> impl IntoResponse {
    let css_path = static_dir().join("app.css");
    match tokio::fs::read(css_path).await {
        Ok(contents) => ([(header::CONTENT_TYPE, "text/css")], contents).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route(
            "/",
            get(
                |axum::extract::Query(params): axum::extract::Query<
                    std::collections::HashMap<String, String>,
                >| async move {
                    let locale = params
                        .get("lang")
                        .map(|value| value.to_lowercase())
                        .unwrap_or_else(|| "en".to_string());
                    render_shell(locale.as_str())
                },
            ),
        )
        .route("/assets/app.css", get(css_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3100")
        .await
        .expect("bind storefront");
    axum::serve(listener, app).await.expect("serve storefront");
}
