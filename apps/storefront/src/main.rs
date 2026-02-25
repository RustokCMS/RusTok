mod modules;

use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
// GlobalAttributes enables id= usage in view! macros.
use leptos::prelude::{ClassAttribute, CollectView, ElementChild, GlobalAttributes, RenderHtml};
use leptos::{component, view, IntoView};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::modules::{components_for_slot, StorefrontSlot};

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
        <div class="rounded-xl border border-border bg-card shadow">
            <div class="h-40 w-full rounded-t-xl bg-gradient-to-br from-primary/10 to-secondary" />
            <div class="p-5 space-y-3">
                <div class="flex items-start justify-between gap-2">
                    <h3 class="text-base font-semibold text-card-foreground">{product.title}</h3>
                    <span class="inline-flex items-center rounded-full bg-secondary px-2.5 py-0.5 text-xs font-medium text-secondary-foreground">{badge}</span>
                </div>
                <p class="text-sm text-muted-foreground">{product.description}</p>
                <div class="flex items-center justify-between pt-1">
                    <span class="text-lg font-semibold text-foreground">{product.price}</span>
                    <button class="inline-flex items-center justify-center rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground shadow hover:bg-primary/90 transition-colors">
                        {cta_view}
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn StorefrontShell(locale: String) -> impl IntoView {
    let strings = locale_strings(locale.as_str());
    let products = featured_products(locale.as_str());
    let module_sections = components_for_slot(StorefrontSlot::HomeAfterHero);
    view! {
        <div class="min-h-screen bg-background text-foreground">
            <header class="sticky top-0 z-40 border-b border-border bg-background/95 backdrop-blur">
                <div class="container-app flex h-14 w-full items-center">
                    <div class="flex-1">
                        <a class="text-xl font-bold text-foreground hover:text-primary transition-colors" href="/">
                            "RusToK"
                        </a>
                    </div>
                    <nav class="hidden lg:flex items-center gap-6">
                        <a class="text-sm text-muted-foreground hover:text-foreground transition-colors" href="#home">{strings.nav_home}</a>
                        <a class="text-sm text-muted-foreground hover:text-foreground transition-colors" href="#catalog">{strings.nav_catalog}</a>
                        <a class="text-sm text-muted-foreground hover:text-foreground transition-colors" href="#about">{strings.nav_about}</a>
                        <a class="text-sm text-muted-foreground hover:text-foreground transition-colors" href="#contact">{strings.nav_contact}</a>
                    </nav>
                    <div class="flex items-center gap-3 ml-6">
                        <div class="relative">
                            <details class="group">
                                <summary class="inline-flex items-center gap-1 rounded-md border border-input bg-background px-3 py-1.5 text-sm text-foreground cursor-pointer hover:bg-accent hover:text-accent-foreground transition-colors list-none">
                                    {strings.nav_language}
                                </summary>
                                <ul class="absolute right-0 mt-1 w-32 rounded-md border border-border bg-popover p-1 shadow-md z-50">
                                    <li>
                                        <a class="block rounded px-3 py-1.5 text-sm text-popover-foreground hover:bg-accent hover:text-accent-foreground transition-colors" href="/?lang=en">
                                            "English"
                                        </a>
                                    </li>
                                    <li>
                                        <a class="block rounded px-3 py-1.5 text-sm text-popover-foreground hover:bg-accent hover:text-accent-foreground transition-colors" href="/?lang=ru">
                                            "Русский"
                                        </a>
                                    </li>
                                </ul>
                            </details>
                        </div>
                        <a class="inline-flex items-center justify-center rounded-md bg-primary px-4 py-1.5 text-sm font-medium text-primary-foreground shadow hover:bg-primary/90 transition-colors" href="#catalog">
                            {strings.cta_primary}
                        </a>
                    </div>
                </div>
            </header>

            <main class="space-y-20">
                <section id="home" class="bg-background">
                    <div class="container-app flex flex-col gap-10 py-16 lg:flex-row lg:items-center lg:justify-between">
                        <div class="max-w-xl space-y-6">
                            <h1 class="text-4xl font-bold text-foreground lg:text-5xl">
                                {strings.hero_title}
                            </h1>
                            <p class="text-lg text-muted-foreground">{strings.hero_subtitle}</p>
                            <div class="flex flex-wrap gap-3">
                                <a class="inline-flex items-center justify-center rounded-md bg-primary px-5 py-2.5 text-sm font-medium text-primary-foreground shadow hover:bg-primary/90 transition-colors" href="#catalog">
                                    {strings.cta_primary}
                                </a>
                                <a class="inline-flex items-center justify-center rounded-md border border-input bg-background px-5 py-2.5 text-sm font-medium text-foreground shadow-sm hover:bg-accent hover:text-accent-foreground transition-colors" href="#about">
                                    {strings.cta_secondary}
                                </a>
                            </div>
                        </div>
                        <div class="grid grid-cols-1 divide-y divide-border rounded-xl border border-border bg-card shadow sm:grid-cols-3 sm:divide-x sm:divide-y-0 lg:w-auto">
                            <div class="p-6 text-center">
                                <p class="text-sm text-muted-foreground">{strings.stat_ssr_title}</p>
                                <p class="mt-1 text-2xl font-bold text-foreground">{strings.stat_ssr_value}</p>
                                <p class="mt-1 text-xs text-muted-foreground">{strings.stat_ssr_desc}</p>
                            </div>
                            <div class="p-6 text-center">
                                <p class="text-sm text-muted-foreground">{strings.stat_i18n_title}</p>
                                <p class="mt-1 text-2xl font-bold text-foreground">{strings.stat_i18n_value}</p>
                                <p class="mt-1 text-xs text-muted-foreground">{strings.stat_i18n_desc}</p>
                            </div>
                            <div class="p-6 text-center">
                                <p class="text-sm text-muted-foreground">{strings.stat_modules_title}</p>
                                <p class="mt-1 text-2xl font-bold text-foreground">{strings.stat_modules_value}</p>
                                <p class="mt-1 text-xs text-muted-foreground">{strings.stat_modules_desc}</p>
                            </div>
                        </div>
                    </div>
                </section>

                {module_sections
                    .into_iter()
                    .map(|module| (module.render)())
                    .collect_view()}

                <section id="catalog" class="container-app">
                    <div class="mx-auto max-w-6xl space-y-6">
                        <div>
                            <h2 class="text-3xl font-bold text-foreground">{strings.featured_title}</h2>
                            <p class="mt-1 text-muted-foreground">{strings.featured_subtitle}</p>
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
                        <div class="rounded-xl border border-border bg-card p-6 shadow space-y-3">
                            <h3 class="text-lg font-semibold text-card-foreground">{strings.story_title}</h3>
                            <p class="text-sm text-muted-foreground">{strings.story_body}</p>
                            <div class="flex gap-2 flex-wrap">
                                <span class="inline-flex items-center rounded-full border border-border px-2.5 py-0.5 text-xs font-medium text-foreground">"GraphQL"</span>
                                <span class="inline-flex items-center rounded-full border border-border px-2.5 py-0.5 text-xs font-medium text-foreground">"Events"</span>
                                <span class="inline-flex items-center rounded-full border border-border px-2.5 py-0.5 text-xs font-medium text-foreground">"Search"</span>
                            </div>
                        </div>
                        <div class="rounded-xl border border-border bg-card p-6 shadow space-y-4">
                            <h3 class="text-lg font-semibold text-card-foreground">{strings.newsletter_title}</h3>
                            <p class="text-sm text-muted-foreground">{strings.newsletter_body}</p>
                            <div class="flex gap-2">
                                <input
                                    class="flex-1 rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
                                    placeholder=strings.newsletter_placeholder
                                />
                                <button class="inline-flex items-center justify-center rounded-md bg-secondary px-4 py-2 text-sm font-medium text-secondary-foreground shadow-sm hover:bg-secondary/80 transition-colors">
                                    {strings.newsletter_cta}
                                </button>
                            </div>
                            <span class="text-xs text-muted-foreground">{strings.newsletter_note}</span>
                        </div>
                    </div>
                </section>
            </main>

            <footer id="contact" class="mt-20 border-t border-border bg-muted/40 px-4 py-10">
                <div class="container-app space-y-3 text-center">
                    <p class="text-sm text-muted-foreground">{strings.footer_tagline}</p>
                    <div class="flex justify-center gap-3">
                        <span class="inline-flex items-center rounded-full bg-primary px-2.5 py-0.5 text-xs font-medium text-primary-foreground">"SSR"</span>
                        <span class="inline-flex items-center rounded-full bg-secondary px-2.5 py-0.5 text-xs font-medium text-secondary-foreground">"Tailwind"</span>
                        <span class="inline-flex items-center rounded-full border border-border px-2.5 py-0.5 text-xs font-medium text-foreground">"shadcn"</span>
                    </div>
                </div>
            </footer>
        </div>
    }
}

fn render_shell(locale: &str) -> String {
    let locale_owned = locale.to_string();
    let app_html = {
        let locale = locale_owned.clone();
        view! { <StorefrontShell locale=locale /> }.to_html()
    };
    format!(
        r#"<!DOCTYPE html>
<html lang="{locale}">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>RusToK Storefront</title>
  <link rel="stylesheet" href="/assets/app.css" />
</head>
<body>
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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    modules::init_modules();

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

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3100").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
