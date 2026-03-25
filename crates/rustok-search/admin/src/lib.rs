mod api;
mod model;

use leptos::ev::{MouseEvent, SubmitEvent};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use rustok_api::UiRouteContext;

use crate::model::{
    LaggingSearchDocumentPayload, SearchAdminBootstrap, SearchDiagnosticsPayload, SearchFacetGroup,
    SearchPreviewFilters, SearchPreviewPayload,
};

#[component]
pub fn SearchAdmin() -> impl IntoView {
    let route_context = use_context::<UiRouteContext>().unwrap_or_default();
    let route_segment = route_context
        .route_segment
        .clone()
        .unwrap_or_else(|| "search".to_string());
    let initial_query = route_context.query_value("q").unwrap_or("").to_string();
    let initial_locale = route_context.locale.clone();
    let on_playground = route_context.subpath_matches("playground");
    let on_diagnostics = route_context.subpath_matches("analytics");
    let on_dictionaries = route_context.subpath_matches("dictionaries");

    let token = leptos_auth::hooks::use_token();
    let tenant = leptos_auth::hooks::use_tenant();

    let (query, set_query) = signal(initial_query);
    let (entity_types, set_entity_types) = signal(String::new());
    let (source_modules, set_source_modules) = signal(String::new());
    let (statuses, set_statuses) = signal(String::new());
    let (preview, set_preview) = signal(Option::<SearchPreviewPayload>::None);
    let (preview_error, set_preview_error) = signal(Option::<String>::None);
    let (refresh_nonce, set_refresh_nonce) = signal(0_u64);
    let (rebuild_busy, set_rebuild_busy) = signal(false);
    let (rebuild_feedback, set_rebuild_feedback) = signal(Option::<String>::None);
    let (rebuild_target_type, set_rebuild_target_type) = signal("search".to_string());
    let (rebuild_target_id, set_rebuild_target_id) = signal(String::new());
    let (busy, set_busy) = signal(false);

    let bootstrap = Resource::new(
        move || (token.get(), tenant.get(), refresh_nonce.get()),
        move |(token_value, tenant_value, _)| async move {
            api::fetch_bootstrap(token_value, tenant_value).await
        },
    );
    let lagging_documents = Resource::new(
        move || (token.get(), tenant.get(), refresh_nonce.get()),
        move |(token_value, tenant_value, _)| async move {
            api::fetch_lagging_documents(token_value, tenant_value, Some(25)).await
        },
    );

    let run_preview = Callback::new(move |ev: SubmitEvent| {
        ev.prevent_default();
        set_preview_error.set(None);
        set_busy.set(true);
        let filters = SearchPreviewFilters {
            entity_types: parse_csv(entity_types.get_untracked()),
            source_modules: parse_csv(source_modules.get_untracked()),
            statuses: parse_csv(statuses.get_untracked()),
        };
        spawn_local({
            let token_value = token.get_untracked();
            let tenant_value = tenant.get_untracked();
            let query_value = query.get_untracked();
            let locale_value = initial_locale.clone();
            async move {
                match api::fetch_search_preview(
                    token_value,
                    tenant_value,
                    query_value,
                    locale_value,
                    filters,
                )
                .await
                {
                    Ok(result) => set_preview.set(Some(result)),
                    Err(err) => {
                        set_preview_error.set(Some(format!("Failed to run search preview: {err}")))
                    }
                }
                set_busy.set(false);
            }
        });
    });

    let queue_rebuild = move |_| {
        set_rebuild_busy.set(true);
        set_rebuild_feedback.set(None);
        spawn_local({
            let token_value = token.get_untracked();
            let tenant_value = tenant.get_untracked();
            let target_type = rebuild_target_type.get_untracked();
            let target_id = optional_text(rebuild_target_id.get_untracked());
            async move {
                match api::trigger_search_rebuild(
                    token_value,
                    tenant_value,
                    Some(target_type.clone()),
                    target_id,
                )
                .await
                {
                    Ok(result) => {
                        let suffix = result
                            .target_id
                            .as_ref()
                            .map(|id| format!(" for target {id}"))
                            .unwrap_or_default();
                        set_rebuild_feedback.set(Some(format!(
                            "Rebuild queued for {} scope{}.",
                            result.target_type, suffix
                        )));
                        set_refresh_nonce.update(|value| *value += 1);
                    }
                    Err(err) => set_rebuild_feedback
                        .set(Some(format!("Failed to queue search rebuild: {err}"))),
                }
                set_rebuild_busy.set(false);
            }
        });
    };

    view! {
        <div class="space-y-6">
            <header class="flex flex-col gap-4 rounded-2xl border border-border bg-card p-6 shadow-sm lg:flex-row lg:items-start lg:justify-between">
                <div class="space-y-2">
                    <span class="inline-flex items-center rounded-full border border-border px-3 py-1 text-xs font-medium text-muted-foreground">"search"</span>
                    <h1 class="text-2xl font-semibold text-card-foreground">"Search Control Plane"</h1>
                    <p class="max-w-3xl text-sm text-muted-foreground">
                        "Module-owned admin surface for search diagnostics, scoped rebuilds, and PostgreSQL FTS inspection."
                    </p>
                </div>
                <div class="flex flex-wrap gap-2">
                    <A href=format!("/modules/{route_segment}") attr:class=tab_class(!on_playground && !on_diagnostics && !on_dictionaries)>"Overview"</A>
                    <A href=format!("/modules/{route_segment}/playground") attr:class=tab_class(on_playground)>"Playground"</A>
                    <A href=format!("/modules/{route_segment}/analytics") attr:class=tab_class(on_diagnostics)>"Diagnostics"</A>
                    <A href=format!("/modules/{route_segment}/dictionaries") attr:class=tab_class(on_dictionaries)>"Dictionaries"</A>
                </div>
            </header>

            <Suspense fallback=move || view! { <div class="h-32 animate-pulse rounded-2xl bg-muted"></div> }>
                {move || {
                    bootstrap.get().map(|result| match result {
                        Ok(bootstrap) => {
                            if on_playground {
                                playground_view(
                                    query,
                                    set_query,
                                    entity_types,
                                    set_entity_types,
                                    source_modules,
                                    set_source_modules,
                                    statuses,
                                    set_statuses,
                                    preview,
                                    preview_error,
                                    busy,
                                    run_preview,
                                ).into_any()
                            } else if on_diagnostics {
                                diagnostics_view(bootstrap.search_diagnostics, lagging_documents).into_any()
                            } else if on_dictionaries {
                                placeholder_view(
                                    "Search Dictionaries",
                                    "Dictionary editors are still a later phase. Diagnostics and scoped rebuilds are already live.",
                                ).into_any()
                            } else {
                                overview_view(
                                    bootstrap,
                                    rebuild_target_type,
                                    set_rebuild_target_type,
                                    rebuild_target_id,
                                    set_rebuild_target_id,
                                    rebuild_busy,
                                    rebuild_feedback,
                                    queue_rebuild,
                                ).into_any()
                            }
                        }
                        Err(err) => view! {
                            <div class="rounded-xl border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
                                {format!("Failed to load search bootstrap data: {err}")}
                            </div>
                        }.into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}

fn overview_view(
    bootstrap: SearchAdminBootstrap,
    rebuild_target_type: ReadSignal<String>,
    set_rebuild_target_type: WriteSignal<String>,
    rebuild_target_id: ReadSignal<String>,
    set_rebuild_target_id: WriteSignal<String>,
    rebuild_busy: ReadSignal<bool>,
    rebuild_feedback: ReadSignal<Option<String>>,
    queue_rebuild: impl Fn(MouseEvent) + 'static + Copy,
) -> impl IntoView {
    view! {
        <section class="space-y-6">
            <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
                <InfoCard title="Active engine" value=bootstrap.search_settings_preview.active_engine detail="Effective search settings for the current tenant." />
                <InfoCard title="Fallback engine" value=bootstrap.search_settings_preview.fallback_engine detail="Used when an external engine is configured but unavailable." />
                <InfoCard title="Available engines" value=bootstrap.available_search_engines.len().to_string() detail="Only connectors installed in the runtime appear here." />
                <InfoCard title="Updated at" value=bootstrap.search_settings_preview.updated_at detail="Timestamp of the effective settings record." />
            </div>
            <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-5">
                <DiagnosticsCard diagnostics=bootstrap.search_diagnostics.clone() />
                <InfoCard title="Documents" value=bootstrap.search_diagnostics.total_documents.to_string() detail="Total search documents in rustok-search storage." />
                <InfoCard title="Public docs" value=bootstrap.search_diagnostics.public_documents.to_string() detail="Published documents visible to storefront search." />
                <InfoCard title="Stale docs" value=bootstrap.search_diagnostics.stale_documents.to_string() detail="Documents where indexed_at lags behind source updated_at." />
                <InfoCard title="Max lag" value=format!("{}s", bootstrap.search_diagnostics.max_lag_seconds) detail="Worst-case lag between source update and search projection." />
            </div>
            <section class="rounded-2xl border border-border bg-card p-6 shadow-sm">
                <div class="space-y-1">
                    <h2 class="text-lg font-semibold text-card-foreground">"Scoped Rebuild"</h2>
                    <p class="text-sm text-muted-foreground">
                        "Queue tenant-wide or scoped rebuilds. `content` and `product` rebuild the whole slice when target ID is empty, or a single entity when target ID is provided."
                    </p>
                </div>
                <div class="mt-5 grid gap-4 md:grid-cols-[14rem_minmax(0,1fr)_auto]">
                    <label class="block space-y-2">
                        <span class="text-sm font-medium text-card-foreground">"Scope"</span>
                        <select class="w-full rounded-lg border border-input bg-background px-3 py-2 text-sm" prop:value=rebuild_target_type on:change=move |ev| set_rebuild_target_type.set(event_target_value(&ev))>
                            <option value="search">"search"</option>
                            <option value="content">"content"</option>
                            <option value="product">"product"</option>
                        </select>
                    </label>
                    <label class="block space-y-2">
                        <span class="text-sm font-medium text-card-foreground">"Target ID (optional)"</span>
                        <input type="text" class="w-full rounded-lg border border-input bg-background px-3 py-2 text-sm" placeholder="UUID for single node/product rebuild" prop:value=rebuild_target_id on:input=move |ev| set_rebuild_target_id.set(event_target_value(&ev)) />
                    </label>
                    <div class="flex items-end">
                        <button type="button" class="inline-flex items-center justify-center rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition hover:bg-primary/90 disabled:opacity-50" disabled=move || rebuild_busy.get() on:click=queue_rebuild>
                            {move || if rebuild_busy.get() { "Queueing..." } else { "Queue Rebuild" }}
                        </button>
                    </div>
                </div>
                <Show when=move || rebuild_feedback.get().is_some()>
                    <div class="mt-4 rounded-xl border border-border bg-muted/20 px-4 py-3 text-sm text-muted-foreground">
                        {move || rebuild_feedback.get().unwrap_or_default()}
                    </div>
                </Show>
            </section>
        </section>
    }
}

fn playground_view(
    query: ReadSignal<String>,
    set_query: WriteSignal<String>,
    entity_types: ReadSignal<String>,
    set_entity_types: WriteSignal<String>,
    source_modules: ReadSignal<String>,
    set_source_modules: WriteSignal<String>,
    statuses: ReadSignal<String>,
    set_statuses: WriteSignal<String>,
    preview: ReadSignal<Option<SearchPreviewPayload>>,
    preview_error: ReadSignal<Option<String>>,
    busy: ReadSignal<bool>,
    run_preview: Callback<SubmitEvent>,
) -> impl IntoView {
    view! { <section class="grid gap-6 xl:grid-cols-[minmax(0,22rem)_minmax(0,1fr)]">
        <form class="space-y-4 rounded-2xl border border-border bg-card p-6 shadow-sm" on:submit=move |ev| run_preview.run(ev)>
            <div class="space-y-1"><h2 class="text-lg font-semibold text-card-foreground">"Search Preview"</h2><p class="text-sm text-muted-foreground">"Runs the current PostgreSQL FTS preview path over rustok-search documents."</p></div>
            <label class="block space-y-2"><span class="text-sm font-medium text-card-foreground">"Query"</span><input type="text" class="w-full rounded-lg border border-input bg-background px-3 py-2 text-sm" prop:value=query on:input=move |ev| set_query.set(event_target_value(&ev)) /></label>
            <label class="block space-y-2"><span class="text-sm font-medium text-card-foreground">"Entity types (CSV)"</span><input type="text" class="w-full rounded-lg border border-input bg-background px-3 py-2 text-sm" prop:value=entity_types on:input=move |ev| set_entity_types.set(event_target_value(&ev)) /></label>
            <label class="block space-y-2"><span class="text-sm font-medium text-card-foreground">"Source modules (CSV)"</span><input type="text" class="w-full rounded-lg border border-input bg-background px-3 py-2 text-sm" prop:value=source_modules on:input=move |ev| set_source_modules.set(event_target_value(&ev)) /></label>
            <label class="block space-y-2"><span class="text-sm font-medium text-card-foreground">"Statuses (CSV)"</span><input type="text" class="w-full rounded-lg border border-input bg-background px-3 py-2 text-sm" prop:value=statuses on:input=move |ev| set_statuses.set(event_target_value(&ev)) /></label>
            <Show when=move || preview_error.get().is_some()><div class="rounded-xl border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">{move || preview_error.get().unwrap_or_default()}</div></Show>
            <button type="submit" class="inline-flex w-full items-center justify-center rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition hover:bg-primary/90 disabled:opacity-50" disabled=move || busy.get()>{move || if busy.get() { "Running..." } else { "Run FTS Preview" }}</button>
        </form>
        <div class="space-y-6">
            <Show when=move || preview.get().is_some() fallback=move || view! { <div class="rounded-2xl border border-dashed border-border bg-card p-10 text-center text-sm text-muted-foreground shadow-sm">"Run a preview query to inspect FTS results, facets, and effective engine output."</div> }>
                {move || preview.get().map(preview_panel)}
            </Show>
        </div>
    </section> }
}

fn diagnostics_view(
    diagnostics: SearchDiagnosticsPayload,
    lagging_documents: Resource<Result<Vec<LaggingSearchDocumentPayload>, api::ApiError>>,
) -> impl IntoView {
    view! {
        <section class="space-y-6">
            <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-5">
                <DiagnosticsCard diagnostics=diagnostics.clone() />
                <InfoCard title="Lagging docs" value=diagnostics.stale_documents.to_string() detail="Documents where projection timestamps are behind source updates." />
                <InfoCard title="Max lag" value=format!("{}s", diagnostics.max_lag_seconds) detail="Largest observed lag in seconds." />
                <InfoCard title="Newest indexed" value=diagnostics.newest_indexed_at.unwrap_or_else(|| "not indexed yet".to_string()) detail="Most recent index write in rustok-search storage." />
                <InfoCard title="Oldest indexed" value=diagnostics.oldest_indexed_at.unwrap_or_else(|| "not indexed yet".to_string()) detail="Oldest surviving indexed document timestamp." />
            </div>
            <section class="rounded-2xl border border-border bg-card p-6 shadow-sm">
                <div class="space-y-1"><h2 class="text-lg font-semibold text-card-foreground">"Lagging Documents"</h2><p class="text-sm text-muted-foreground">"Raw diagnostics for the most stale documents in search storage."</p></div>
                <div class="mt-5">
                    <Suspense fallback=move || view! { <div class="h-24 animate-pulse rounded-xl bg-muted"></div> }>
                        {move || lagging_documents.get().map(|result| match result {
                            Ok(rows) => lagging_table(rows).into_any(),
                            Err(err) => view! { <div class="rounded-xl border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">{format!("Failed to load lagging search diagnostics: {err}")}</div> }.into_any(),
                        })}
                    </Suspense>
                </div>
            </section>
        </section>
    }
}

fn preview_panel(payload: SearchPreviewPayload) -> impl IntoView {
    view! { <section class="rounded-2xl border border-border bg-card p-6 shadow-sm">
        <div><h2 class="text-lg font-semibold text-card-foreground">"Preview Results"</h2><p class="text-sm text-muted-foreground">{format!("{} results in {} ms via {}", payload.total, payload.took_ms, payload.engine)}</p></div>
        <div class="mt-5 grid gap-4 lg:grid-cols-3">{payload.facets.iter().map(|facet| view! { <FacetCard facet=facet.clone() /> }).collect_view()}</div>
        <div class="mt-6 space-y-3">{payload.items.into_iter().map(|item| view! {
            <article class="rounded-xl border border-border bg-background p-4">
                <div class="flex flex-wrap items-center gap-2 text-xs font-medium uppercase tracking-[0.16em] text-muted-foreground"><span>{item.entity_type.clone()}</span><span>"|"</span><span>{item.source_module.clone()}</span><span>"|"</span><span>{format!("score {:.3}", item.score)}</span></div>
                <h3 class="mt-2 text-base font-semibold text-card-foreground">{item.title}</h3>
                <p class="mt-2 text-sm text-muted-foreground">{item.snippet.unwrap_or_else(|| "No snippet returned.".to_string())}</p>
            </article>
        }).collect_view()}</div>
    </section> }
}

fn lagging_table(rows: Vec<LaggingSearchDocumentPayload>) -> impl IntoView {
    if rows.is_empty() {
        return view! { <div class="rounded-xl border border-dashed border-border p-12 text-center"><p class="text-sm text-muted-foreground">"No lagging documents detected. Search projection is currently caught up."</p></div> }.into_any();
    }
    view! { <div class="overflow-hidden rounded-xl border border-border"><table class="w-full text-sm">
        <thead class="border-b border-border bg-muted/50"><tr>
            <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-muted-foreground">"Title"</th>
            <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-muted-foreground">"Type"</th>
            <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-muted-foreground">"Locale"</th>
            <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-muted-foreground">"Lag"</th>
            <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-muted-foreground">"Indexed"</th>
            <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-muted-foreground">"Updated"</th>
        </tr></thead>
        <tbody class="divide-y divide-border">{rows.into_iter().map(|row| view! {
            <tr class="transition-colors hover:bg-muted/30">
                <td class="px-4 py-3 align-top"><div class="font-medium text-card-foreground">{row.title}</div><div class="mt-1 text-xs text-muted-foreground">{row.document_key}</div></td>
                <td class="px-4 py-3 align-top text-xs text-muted-foreground">{format!("{}/{} ({})", row.source_module, row.entity_type, row.status)}</td>
                <td class="px-4 py-3 align-top text-xs text-muted-foreground">{row.locale}</td>
                <td class="px-4 py-3 align-top"><span class="inline-flex rounded-full border border-amber-200 bg-amber-50 px-2.5 py-0.5 text-xs font-semibold text-amber-700">{format!("{}s", row.lag_seconds)}</span></td>
                <td class="px-4 py-3 align-top text-xs text-muted-foreground">{row.indexed_at}</td>
                <td class="px-4 py-3 align-top text-xs text-muted-foreground">{row.updated_at}</td>
            </tr>
        }).collect_view()}</tbody>
    </table></div> }.into_any()
}

fn placeholder_view(title: &'static str, body: &'static str) -> impl IntoView {
    view! { <section class="rounded-2xl border border-dashed border-border bg-card p-10 text-center shadow-sm"><h2 class="text-xl font-semibold text-card-foreground">{title}</h2><p class="mt-3 text-sm text-muted-foreground">{body}</p></section> }
}

#[component]
fn DiagnosticsCard(diagnostics: SearchDiagnosticsPayload) -> impl IntoView {
    let badge_class = match diagnostics.state.as_str() {
        "healthy" => "border-emerald-200 bg-emerald-50 text-emerald-700",
        "lagging" => "border-amber-200 bg-amber-50 text-amber-700",
        _ => "border-slate-200 bg-slate-50 text-slate-700",
    };
    view! { <article class="rounded-2xl border border-border bg-card p-5 shadow-sm">
        <div class="text-xs font-medium uppercase tracking-[0.2em] text-muted-foreground">"Index state"</div>
        <div class="mt-3"><span class=format!("inline-flex rounded-full border px-3 py-1 text-xs font-semibold {badge_class}")>{diagnostics.state}</span></div>
        <p class="mt-3 text-sm text-muted-foreground">{format!("Newest indexed: {}", diagnostics.newest_indexed_at.unwrap_or_else(|| "not indexed yet".to_string()))}</p>
    </article> }
}

#[component]
fn InfoCard<T, U>(title: T, value: U, detail: &'static str) -> impl IntoView
where
    T: IntoView + 'static,
    U: IntoView + 'static,
{
    view! { <article class="rounded-2xl border border-border bg-card p-5 shadow-sm"><div class="text-xs font-medium uppercase tracking-[0.2em] text-muted-foreground">{title}</div><div class="mt-2 text-lg font-semibold text-card-foreground">{value}</div><p class="mt-2 text-sm text-muted-foreground">{detail}</p></article> }
}

#[component]
fn FacetCard(facet: SearchFacetGroup) -> impl IntoView {
    view! { <article class="rounded-xl border border-border bg-background p-4"><div class="text-sm font-semibold capitalize text-card-foreground">{facet.name.replace('_', " ")}</div><div class="mt-3 flex flex-wrap gap-2">{facet.buckets.into_iter().map(|bucket| view! { <span class="inline-flex items-center rounded-full border border-border px-3 py-1 text-xs font-medium text-muted-foreground">{format!("{} ({})", bucket.value, bucket.count)}</span> }).collect_view()}</div></article> }
}

fn parse_csv(value: String) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn optional_text(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn tab_class(active: bool) -> &'static str {
    if active {
        "inline-flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition hover:bg-primary/90"
    } else {
        "inline-flex items-center gap-2 rounded-lg border border-border px-4 py-2 text-sm font-medium text-foreground transition hover:bg-accent hover:text-accent-foreground"
    }
}
