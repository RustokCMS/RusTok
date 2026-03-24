mod api;
mod model;

use leptos::prelude::*;
use rustok_api::UiRouteContext;

use crate::model::{PageDetail, PageListItem, StorefrontPagesData};

#[component]
pub fn PagesView() -> impl IntoView {
    let route_context = use_context::<UiRouteContext>().unwrap_or_default();
    let selected_slug = route_context
        .query_value("slug")
        .unwrap_or("home")
        .to_string();
    let selected_locale = route_context.locale.clone();

    let pages_resource = Resource::new_blocking(
        move || (selected_slug.clone(), selected_locale.clone()),
        move |(page_slug, locale)| async move { api::fetch_storefront_pages(page_slug, locale).await },
    );

    view! {
        <section class="rounded-3xl border border-border bg-card p-8 shadow-sm">
            <div class="max-w-3xl space-y-3">
                <span class="inline-flex items-center rounded-full border border-border px-3 py-1 text-xs font-medium text-muted-foreground">
                    "pages"
                </span>
                <h2 class="text-3xl font-semibold text-card-foreground">
                    "Page-driven storefront experiences"
                </h2>
                <p class="text-sm text-muted-foreground">
                    "This module package renders real page data through the pages module GraphQL contract."
                </p>
            </div>

            <div class="mt-8">
                <Suspense fallback=|| view! {
                    <div class="space-y-4">
                        <div class="h-32 animate-pulse rounded-2xl bg-muted"></div>
                        <div class="grid gap-3 md:grid-cols-2">
                            <div class="h-24 animate-pulse rounded-2xl bg-muted"></div>
                            <div class="h-24 animate-pulse rounded-2xl bg-muted"></div>
                        </div>
                    </div>
                }>
                    {move || {
                        let pages_resource = pages_resource.clone();
                        Suspend::new(async move {
                            match pages_resource.await {
                                Ok(data) => view! { <PagesShowcase data /> }.into_any(),
                                Err(err) => view! {
                                    <div class="rounded-2xl border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
                                        {format!("Failed to load pages storefront data: {err}")}
                                    </div>
                                }.into_any(),
                            }
                        })
                    }}
                </Suspense>
            </div>
        </section>
    }
}

#[component]
fn PagesShowcase(data: StorefrontPagesData) -> impl IntoView {
    view! {
        <div class="space-y-6">
            <SelectedPageCard page=data.selected_page />
            <PublishedPagesList items=data.pages.items total=data.pages.total />
        </div>
    }
}

#[component]
fn SelectedPageCard(page: Option<PageDetail>) -> impl IntoView {
    let Some(page) = page else {
        return view! {
            <article class="rounded-2xl border border-dashed border-border p-6">
                <h3 class="text-lg font-semibold text-card-foreground">"Requested page is not published yet"</h3>
                <p class="mt-2 text-sm text-muted-foreground">
                    "Choose a page from the list below with `?slug=` or publish it from the pages admin package."
                </p>
            </article>
        }
        .into_any();
    };

    let title = page
        .translation
        .as_ref()
        .and_then(|translation| translation.title.clone())
        .unwrap_or_else(|| "Page".to_string());
    let slug = page
        .translation
        .as_ref()
        .and_then(|translation| translation.slug.clone())
        .unwrap_or_else(|| "home".to_string());
    let effective_locale = page
        .effective_locale
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let body = page
        .body
        .as_ref()
        .map(|body| summarize_content(body.content.as_str(), body.format.as_str()))
        .unwrap_or_else(|| "No body content yet.".to_string());

    view! {
        <article class="rounded-2xl border border-border bg-background p-6">
            <div class="flex flex-wrap items-center gap-2 text-xs font-medium uppercase tracking-[0.22em] text-muted-foreground">
                <span>{format!("selected slug: {slug}")}</span>
                <span>"·"</span>
                <span>{format!("locale: {effective_locale}")}</span>
            </div>
            <h3 class="mt-3 text-2xl font-semibold text-foreground">{title}</h3>
            <p class="mt-3 whitespace-pre-line text-sm leading-7 text-muted-foreground">{body}</p>
        </article>
    }
    .into_any()
}

#[component]
fn PublishedPagesList(items: Vec<PageListItem>, total: u64) -> impl IntoView {
    let route_context = use_context::<UiRouteContext>().unwrap_or_default();
    let route_segment = route_context
        .route_segment
        .unwrap_or_else(|| "pages".to_string());

    if items.is_empty() {
        return view! {
            <article class="rounded-2xl border border-dashed border-border p-6">
                <p class="text-sm text-muted-foreground">
                    "No published pages are available for storefront rendering yet."
                </p>
            </article>
        }
        .into_any();
    }

    view! {
        <div class="space-y-3">
            <div class="flex items-center justify-between gap-3">
                <h3 class="text-lg font-semibold text-card-foreground">"Published pages"</h3>
                <span class="text-sm text-muted-foreground">{format!("{total} total")}</span>
            </div>
            <div class="grid gap-3 md:grid-cols-2">
                {items
                    .into_iter()
                    .map(|page| {
                        let slug = page.slug.unwrap_or_else(|| "missing-slug".to_string());
                        let href = format!("/modules/{route_segment}?slug={slug}");
                        view! {
                            <article class="rounded-2xl border border-border bg-background p-5">
                                <div class="text-xs font-medium uppercase tracking-[0.2em] text-muted-foreground">
                                    {page.status}
                                </div>
                                <h4 class="mt-2 text-base font-semibold text-foreground">
                                    {page.title.unwrap_or_else(|| "Untitled page".to_string())}
                                </h4>
                                <a class="mt-2 inline-flex text-sm text-primary hover:underline" href=href>
                                    {format!("Open {slug}")}
                                </a>
                                <p class="mt-3 text-xs text-muted-foreground">
                                    {format!("template: {}", page.template)}
                                </p>
                            </article>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
    .into_any()
}

fn summarize_content(content: &str, format: &str) -> String {
    if format.eq_ignore_ascii_case("markdown") {
        return content.trim().to_string();
    }

    format!(
        "Stored in `{format}` format. Raw body length: {} characters.",
        content.chars().count()
    )
}
