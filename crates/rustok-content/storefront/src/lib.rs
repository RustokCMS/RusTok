mod api;
mod model;

use leptos::prelude::*;
use rustok_api::UiRouteContext;

use crate::model::{NodeDetail, NodeListItem, StorefrontContentData};

#[component]
pub fn ContentView() -> impl IntoView {
    let route_context = use_context::<UiRouteContext>().unwrap_or_default();
    let selected_id = route_context.query_value("id").map(str::to_string);
    let selected_kind = route_context.query_value("kind").map(str::to_string);
    let selected_locale = route_context.locale.clone();

    let content_resource = Resource::new_blocking(
        move || {
            (
                selected_id.clone(),
                selected_kind.clone(),
                selected_locale.clone(),
            )
        },
        move |(node_id, kind, locale)| async move {
            api::fetch_storefront_content(node_id, kind, locale).await
        },
    );

    view! {
        <section class="rounded-3xl border border-border bg-card p-8 shadow-sm">
            <span class="inline-flex items-center rounded-full border border-border px-3 py-1 text-xs font-medium text-muted-foreground">
                "content"
            </span>
            <h2 class="mt-3 text-3xl font-semibold text-card-foreground">
                "Published content rendered from the module package"
            </h2>
            <p class="mt-3 text-sm text-muted-foreground">
                "This storefront surface reads published nodes through the content module GraphQL contract."
            </p>

            <div class="mt-8">
                <Suspense fallback=|| view! {
                    <div class="space-y-4">
                        <div class="h-40 animate-pulse rounded-2xl bg-muted"></div>
                        <div class="grid gap-3 md:grid-cols-2">
                            <div class="h-28 animate-pulse rounded-2xl bg-muted"></div>
                            <div class="h-28 animate-pulse rounded-2xl bg-muted"></div>
                        </div>
                    </div>
                }>
                    {move || {
                        let content_resource = content_resource.clone();
                        Suspend::new(async move {
                            match content_resource.await {
                                Ok(data) => view! { <ContentShowcase data /> }.into_any(),
                                Err(err) => view! {
                                    <div class="rounded-2xl border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
                                        {format!("Failed to load content storefront data: {err}")}
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
fn ContentShowcase(data: StorefrontContentData) -> impl IntoView {
    view! {
        <div class="space-y-6">
            <SelectedNodeCard node=data.selected_node />
            <PublishedNodesList
                items=data.nodes.items
                total=data.nodes.total
                selected_id=data.selected_id
                selected_kind=data.selected_kind
            />
        </div>
    }
}

#[component]
fn SelectedNodeCard(node: Option<NodeDetail>) -> impl IntoView {
    let Some(node) = node else {
        return view! {
            <article class="rounded-2xl border border-dashed border-border p-6">
                <h3 class="text-lg font-semibold text-card-foreground">"Pick a published node"</h3>
                <p class="mt-2 text-sm text-muted-foreground">
                    "Choose a node from the list below or narrow the showcase with `?kind=`."
                </p>
            </article>
        }
        .into_any();
    };

    let translation = node.translation.clone();
    let title = translation
        .as_ref()
        .and_then(|value| value.title.clone())
        .unwrap_or_else(|| "Content node".to_string());
    let excerpt = translation
        .as_ref()
        .and_then(|value| value.excerpt.clone())
        .unwrap_or_else(|| "No excerpt yet.".to_string());
    let slug = translation
        .as_ref()
        .and_then(|value| value.slug.clone())
        .unwrap_or_else(|| "missing-slug".to_string());
    let effective_locale = node
        .effective_locale
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let published_at = node
        .published_at
        .clone()
        .unwrap_or_else(|| "Unscheduled".to_string());
    let body = node
        .body
        .as_ref()
        .and_then(|value| value.body.clone())
        .map(|content| {
            summarize_content(
                content.as_str(),
                node.body
                    .as_ref()
                    .map(|value| value.format.as_str())
                    .unwrap_or("markdown"),
            )
        })
        .unwrap_or_else(|| "No body content yet.".to_string());

    view! {
        <article class="rounded-2xl border border-border bg-background p-6">
            <div class="flex flex-wrap items-center gap-2 text-xs font-medium uppercase tracking-[0.22em] text-muted-foreground">
                <span>{format!("kind: {}", node.kind)}</span>
                <span>"·"</span>
                <span>{format!("slug: {slug}")}</span>
                <span>"·"</span>
                <span>{format!("locale: {effective_locale}")}</span>
                <span>"·"</span>
                <span>{format!("published: {published_at}")}</span>
            </div>
            <h3 class="mt-3 text-2xl font-semibold text-foreground">{title}</h3>
            <p class="mt-3 text-sm text-muted-foreground">{excerpt}</p>
            <p class="mt-4 whitespace-pre-line text-sm leading-7 text-muted-foreground">{body}</p>
        </article>
    }
    .into_any()
}

#[component]
fn PublishedNodesList(
    items: Vec<NodeListItem>,
    total: u64,
    selected_id: Option<String>,
    selected_kind: Option<String>,
) -> impl IntoView {
    let route_context = use_context::<UiRouteContext>().unwrap_or_default();
    let route_segment = route_context
        .route_segment
        .unwrap_or_else(|| "content".to_string());

    if items.is_empty() {
        return view! {
            <article class="rounded-2xl border border-dashed border-border p-6">
                <p class="text-sm text-muted-foreground">
                    "No published nodes are available for this storefront filter yet."
                </p>
            </article>
        }
        .into_any();
    }

    view! {
        <div class="space-y-3">
            <div class="flex items-center justify-between gap-3">
                <h3 class="text-lg font-semibold text-card-foreground">"Published nodes"</h3>
                <span class="text-sm text-muted-foreground">{format!("{total} total")}</span>
            </div>
            <div class="grid gap-3 md:grid-cols-2">
                {items
                    .into_iter()
                    .map(|node| {
                        let is_selected = selected_id.as_deref() == Some(node.id.as_str());
                        let href = content_href(
                            route_segment.as_str(),
                            node.id.as_str(),
                            selected_kind.as_deref(),
                        );
                        view! {
                            <article class=move || format!(
                                "rounded-2xl border p-5 {}",
                                if is_selected {
                                    "border-primary bg-primary/5"
                                } else {
                                    "border-border bg-background"
                                }
                            )>
                                <div class="text-xs font-medium uppercase tracking-[0.2em] text-muted-foreground">
                                    {format!("{} · {}", node.kind, node.effective_locale)}
                                </div>
                                <h4 class="mt-2 text-base font-semibold text-foreground">
                                    {node.title.clone().unwrap_or_else(|| "Untitled node".to_string())}
                                </h4>
                                <p class="mt-2 text-sm text-muted-foreground">
                                    {node.excerpt.clone().unwrap_or_else(|| "No excerpt yet.".to_string())}
                                </p>
                                <a class="mt-3 inline-flex text-sm text-primary hover:underline" href=href>
                                    "Open node"
                                </a>
                                <p class="mt-3 text-xs text-muted-foreground">
                                    {format!(
                                        "slug: {}{}",
                                        node.slug.unwrap_or_else(|| "missing-slug".to_string()),
                                        node.published_at
                                            .map(|value| format!(" | published: {value}"))
                                            .unwrap_or_default(),
                                    )}
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

fn content_href(route_segment: &str, node_id: &str, kind: Option<&str>) -> String {
    match kind {
        Some(kind_value) if !kind_value.is_empty() => {
            format!("/modules/{route_segment}?id={node_id}&kind={kind_value}")
        }
        _ => format!("/modules/{route_segment}?id={node_id}"),
    }
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
