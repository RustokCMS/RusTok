mod api;
mod model;

use leptos::prelude::*;
use rustok_api::UiRouteContext;

use crate::model::{ProductDetail, ProductListItem, StorefrontCommerceData};

#[component]
pub fn CommerceView() -> impl IntoView {
    let route_context = use_context::<UiRouteContext>().unwrap_or_default();
    let selected_handle = route_context.query_value("handle").map(ToOwned::to_owned);
    let selected_locale = route_context.locale.clone();

    let resource = Resource::new_blocking(
        move || (selected_handle.clone(), selected_locale.clone()),
        move |(handle, locale)| async move { api::fetch_storefront_commerce(handle, locale).await },
    );

    view! {
        <section class="rounded-[2rem] border border-border bg-card p-8 shadow-sm">
            <div class="max-w-3xl space-y-3">
                <span class="inline-flex items-center rounded-full border border-border px-3 py-1 text-xs font-medium uppercase tracking-[0.2em] text-muted-foreground">"commerce"</span>
                <h2 class="text-3xl font-semibold text-card-foreground">"Published catalog from the module package"</h2>
                <p class="text-sm text-muted-foreground">"The storefront now reads published products through the module-owned GraphQL contract with no commerce-specific host wiring."</p>
            </div>
            <div class="mt-8">
                <Suspense fallback=|| view! { <div class="space-y-4"><div class="h-48 animate-pulse rounded-3xl bg-muted"></div><div class="grid gap-3 md:grid-cols-3"><div class="h-28 animate-pulse rounded-2xl bg-muted"></div><div class="h-28 animate-pulse rounded-2xl bg-muted"></div><div class="h-28 animate-pulse rounded-2xl bg-muted"></div></div></div> }>
                    {move || {
                        let resource = resource.clone();
                        Suspend::new(async move {
                            match resource.await {
                                Ok(data) => view! { <CommerceShowcase data /> }.into_any(),
                                Err(err) => view! { <div class="rounded-2xl border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">{format!("Failed to load commerce storefront data: {err}")}</div> }.into_any(),
                            }
                        })
                    }}
                </Suspense>
            </div>
        </section>
    }
}

#[component]
fn CommerceShowcase(data: StorefrontCommerceData) -> impl IntoView {
    view! {
        <div class="grid gap-6 xl:grid-cols-[minmax(0,1.1fr)_minmax(0,0.9fr)]">
            <SelectedProductCard product=data.selected_product />
            <CatalogRail items=data.products.items total=data.products.total />
        </div>
    }
}

#[component]
fn SelectedProductCard(product: Option<ProductDetail>) -> impl IntoView {
    let Some(product) = product else {
        return view! { <article class="rounded-3xl border border-dashed border-border p-8"><h3 class="text-lg font-semibold text-card-foreground">"No published product selected"</h3><p class="mt-2 text-sm text-muted-foreground">"Publish a product from the commerce admin package or open one with `?handle=`."</p></article> }.into_any();
    };

    let translation = product.translations.first().cloned();
    let variant = product.variants.first().cloned();
    let title = translation
        .as_ref()
        .map(|item| item.title.clone())
        .unwrap_or_else(|| "Untitled product".to_string());
    let description = translation
        .and_then(|item| item.description)
        .unwrap_or_else(|| "No localized merchandising copy yet.".to_string());
    let price = variant
        .as_ref()
        .and_then(|item| item.prices.first())
        .map(|item| {
            if let Some(compare) = &item.compare_at_amount {
                format!(
                    "{} {} (compare-at {})",
                    item.currency_code, item.amount, compare
                )
            } else {
                format!("{} {}", item.currency_code, item.amount)
            }
        })
        .unwrap_or_else(|| "No pricing yet".to_string());
    let inventory = variant
        .as_ref()
        .map(|item| item.inventory_quantity)
        .unwrap_or(0);

    view! {
        <article class="rounded-3xl border border-border bg-background p-8">
            <div class="flex flex-wrap items-center gap-2 text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">
                <span>{product.product_type.unwrap_or_else(|| "catalog".to_string())}</span>
                <span>"•"</span>
                <span>{product.vendor.unwrap_or_else(|| "independent label".to_string())}</span>
                <span>"•"</span>
                <span>{product.published_at.unwrap_or_else(|| "scheduled later".to_string())}</span>
            </div>
            <h3 class="mt-4 text-3xl font-semibold text-foreground">{title}</h3>
            <p class="mt-4 text-sm leading-7 text-muted-foreground">{description}</p>
            <div class="mt-6 grid gap-3 md:grid-cols-2">
                <MetricCard title="Price" value=price />
                <MetricCard title="Inventory" value=inventory.to_string() />
            </div>
        </article>
    }.into_any()
}

#[component]
fn CatalogRail(items: Vec<ProductListItem>, total: u64) -> impl IntoView {
    let route_context = use_context::<UiRouteContext>().unwrap_or_default();
    let route_segment = route_context
        .route_segment
        .unwrap_or_else(|| "commerce".to_string());

    if items.is_empty() {
        return view! { <article class="rounded-3xl border border-dashed border-border p-8 text-center text-sm text-muted-foreground">"No published products are available yet."</article> }.into_any();
    }

    view! {
        <div class="space-y-4">
            <div class="flex items-center justify-between gap-3">
                <h3 class="text-lg font-semibold text-card-foreground">"Published products"</h3>
                <span class="text-sm text-muted-foreground">{format!("{total} total")}</span>
            </div>
            <div class="space-y-3">
                {items.into_iter().map(|product| {
                    let href = format!("/modules/{route_segment}?handle={}", product.handle);
                    view! {
                        <article class="rounded-2xl border border-border bg-background p-5">
                            <div class="text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">{product.product_type.unwrap_or_else(|| "catalog".to_string())}</div>
                            <h4 class="mt-2 text-base font-semibold text-card-foreground">{product.title}</h4>
                            <p class="mt-2 text-sm text-muted-foreground">{product.vendor.unwrap_or_else(|| "Independent label".to_string())}</p>
                            <div class="mt-4 flex items-center justify-between gap-3">
                                <span class="text-xs text-muted-foreground">{product.published_at.unwrap_or(product.created_at)}</span>
                                <a class="inline-flex text-sm font-medium text-primary hover:underline" href=href>"Open"</a>
                            </div>
                        </article>
                    }
                }).collect_view()}
            </div>
        </div>
    }.into_any()
}

#[component]
fn MetricCard(title: &'static str, value: String) -> impl IntoView {
    view! { <article class="rounded-2xl border border-border bg-card p-4"><div class="text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">{title}</div><div class="mt-2 text-lg font-semibold text-card-foreground">{value}</div></article> }
}
