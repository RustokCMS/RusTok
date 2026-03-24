mod api;
mod model;

use leptos::prelude::*;
use rustok_api::UiRouteContext;

use crate::model::{BlogPostDetail, BlogPostListItem, StorefrontBlogData};

#[component]
pub fn BlogView() -> impl IntoView {
    let route_context = use_context::<UiRouteContext>().unwrap_or_default();
    let selected_slug = route_context
        .query_value("slug")
        .unwrap_or("latest")
        .to_string();
    let selected_locale = route_context.locale.clone();

    let posts_resource = Resource::new_blocking(
        move || (selected_slug.clone(), selected_locale.clone()),
        move |(post_slug, locale)| async move { api::fetch_storefront_blog(post_slug, locale).await },
    );

    view! {
        <section class="rounded-3xl border border-border bg-card p-8 shadow-sm">
            <div class="max-w-3xl space-y-3">
                <span class="inline-flex items-center rounded-full border border-border px-3 py-1 text-xs font-medium text-muted-foreground">
                    "blog"
                </span>
                <h2 class="text-3xl font-semibold text-card-foreground">
                    "Stories published from the module package"
                </h2>
                <p class="text-sm text-muted-foreground">
                    "This storefront surface reads blog data through GraphQL with no host-specific blog wiring."
                </p>
            </div>

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
                        let posts_resource = posts_resource.clone();
                        Suspend::new(async move {
                            match posts_resource.await {
                                Ok(data) => view! { <BlogShowcase data /> }.into_any(),
                                Err(err) => view! {
                                    <div class="rounded-2xl border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
                                        {format!("Failed to load blog storefront data: {err}")}
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
fn BlogShowcase(data: StorefrontBlogData) -> impl IntoView {
    view! {
        <div class="space-y-6">
            <SelectedPostCard post=data.selected_post />
            <PublishedPostsList items=data.posts.items total=data.posts.total />
        </div>
    }
}

#[component]
fn SelectedPostCard(post: Option<BlogPostDetail>) -> impl IntoView {
    let Some(post) = post else {
        return view! {
            <article class="rounded-2xl border border-dashed border-border p-6">
                <h3 class="text-lg font-semibold text-card-foreground">"Pick a published post"</h3>
                <p class="mt-2 text-sm text-muted-foreground">
                    "Open a post from the list below with `?slug=` or publish one from the blog admin package."
                </p>
            </article>
        }
        .into_any();
    };

    let title = post.title;
    let effective_locale = post.effective_locale;
    let slug = post.slug.unwrap_or_else(|| "missing-slug".to_string());
    let excerpt = post
        .excerpt
        .unwrap_or_else(|| "No excerpt yet.".to_string());
    let published_at = post
        .published_at
        .unwrap_or_else(|| "Unscheduled".to_string());
    let tags = post.tags;
    let body_format = post.body_format;
    let body = post
        .body
        .map(|body| summarize_content(body.as_str(), body_format.as_str()))
        .unwrap_or_else(|| "No body content yet.".to_string());

    view! {
        <article class="rounded-2xl border border-border bg-background p-6">
            <div class="flex flex-wrap items-center gap-2 text-xs font-medium uppercase tracking-[0.22em] text-muted-foreground">
                <span>{format!("slug: {slug}")}</span>
                <span>"В·"</span>
                <span>{format!("locale: {effective_locale}")}</span>
                <span>"В·"</span>
                <span>{format!("published: {published_at}")}</span>
            </div>
            <h3 class="mt-3 text-2xl font-semibold text-foreground">{title}</h3>
            <p class="mt-3 text-sm text-muted-foreground">{excerpt}</p>
            <p class="mt-4 whitespace-pre-line text-sm leading-7 text-muted-foreground">{body}</p>
            {if tags.is_empty() {
                view! { <></> }.into_any()
            } else {
                view! {
                    <div class="mt-5 flex flex-wrap gap-2">
                        {tags
                            .into_iter()
                            .map(|tag| {
                                view! {
                                    <span class="inline-flex rounded-full border border-border px-3 py-1 text-xs text-muted-foreground">
                                        {tag}
                                    </span>
                                }
                            })
                            .collect_view()}
                    </div>
                }
                .into_any()
            }}
        </article>
    }
    .into_any()
}

#[component]
fn PublishedPostsList(items: Vec<BlogPostListItem>, total: u64) -> impl IntoView {
    let route_context = use_context::<UiRouteContext>().unwrap_or_default();
    let route_segment = route_context
        .route_segment
        .unwrap_or_else(|| "blog".to_string());

    if items.is_empty() {
        return view! {
            <article class="rounded-2xl border border-dashed border-border p-6">
                <p class="text-sm text-muted-foreground">
                    "No published blog posts are available for storefront rendering yet."
                </p>
            </article>
        }
        .into_any();
    }

    view! {
        <div class="space-y-3">
            <div class="flex items-center justify-between gap-3">
                <h3 class="text-lg font-semibold text-card-foreground">"Published posts"</h3>
                <span class="text-sm text-muted-foreground">{format!("{total} total")}</span>
            </div>
            <div class="grid gap-3 md:grid-cols-2">
                {items
                    .into_iter()
                    .map(|post| {
                        let route_segment = route_segment.clone();
                        let slug = post.slug.unwrap_or_else(|| "missing-slug".to_string());
                        let href = format!("/modules/{route_segment}?slug={slug}");
                        view! {
                            <article class="rounded-2xl border border-border bg-background p-5">
                                <div class="text-xs font-medium uppercase tracking-[0.2em] text-muted-foreground">
                                    {post.status}
                                </div>
                                <h4 class="mt-2 text-base font-semibold text-foreground">{post.title}</h4>
                                <p class="mt-2 text-sm text-muted-foreground">
                                    {post.excerpt.unwrap_or_else(|| "No excerpt yet.".to_string())}
                                </p>
                                <a class="mt-3 inline-flex text-sm text-primary hover:underline" href=href>
                                    {format!("Open {slug}")}
                                </a>
                                <p class="mt-3 text-xs text-muted-foreground">
                                    {format!("locale: {}", post.effective_locale)}
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
