mod api;
mod model;

use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_auth::hooks::{use_tenant, use_token};

use crate::model::{NodeDetail, NodeDraft, NodeListItem};

#[component]
pub fn ContentAdmin() -> impl IntoView {
    let token = use_token();
    let tenant = use_tenant();
    let (refresh_nonce, set_refresh_nonce) = signal(0_u64);
    let (editing_node_id, set_editing_node_id) = signal(Option::<String>::None);
    let (kind, set_kind) = signal("article".to_string());
    let (title, set_title) = signal(String::new());
    let (slug, set_slug) = signal(String::new());
    let (excerpt, set_excerpt) = signal(String::new());
    let (body, set_body) = signal(String::new());
    let (locale, set_locale) = signal("en".to_string());
    let (body_format, set_body_format) = signal("markdown".to_string());
    let (publish_now, set_publish_now) = signal(false);
    let (kind_filter, set_kind_filter) = signal(String::new());
    let (busy_key, set_busy_key) = signal(Option::<String>::None);
    let (error, set_error) = signal(Option::<String>::None);

    let nodes = Resource::new(
        move || {
            (
                token.get(),
                tenant.get(),
                refresh_nonce.get(),
                locale.get(),
                kind_filter.get(),
            )
        },
        move |(token_value, tenant_value, _, locale_value, kind_filter_value)| async move {
            api::fetch_nodes(
                token_value,
                tenant_value,
                Some(locale_value),
                (!kind_filter_value.trim().is_empty()).then_some(kind_filter_value),
            )
            .await
        },
    );

    let reset_form = move || {
        set_editing_node_id.set(None);
        set_kind.set("article".to_string());
        set_title.set(String::new());
        set_slug.set(String::new());
        set_excerpt.set(String::new());
        set_body.set(String::new());
        set_locale.set("en".to_string());
        set_body_format.set("markdown".to_string());
        set_publish_now.set(false);
    };

    let edit_node = Callback::new(move |(node_id, requested_locale): (String, String)| {
        let token_value = token.get_untracked();
        let tenant_value = tenant.get_untracked();
        set_error.set(None);
        set_busy_key.set(Some(format!("edit:{node_id}")));
        spawn_local(async move {
            match api::fetch_node(
                token_value,
                tenant_value,
                node_id.clone(),
                Some(requested_locale),
            )
            .await
            {
                Ok(Some(node)) => apply_node_to_form(
                    set_editing_node_id,
                    set_kind,
                    set_title,
                    set_slug,
                    set_excerpt,
                    set_body,
                    set_locale,
                    set_body_format,
                    set_publish_now,
                    &node,
                ),
                Ok(None) => set_error.set(Some("Node not found for editing.".to_string())),
                Err(err) => set_error.set(Some(format!("Failed to load node: {err}"))),
            }
            set_busy_key.set(None);
        });
    });

    let submit_node = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        let draft = NodeDraft {
            locale: locale.get_untracked(),
            kind: kind.get_untracked().trim().to_string(),
            title: title.get_untracked().trim().to_string(),
            slug: slug.get_untracked().trim().to_string(),
            excerpt: excerpt.get_untracked().trim().to_string(),
            body: body.get_untracked().trim().to_string(),
            body_format: body_format.get_untracked().trim().to_string(),
        };
        let should_publish = publish_now.get_untracked();
        if draft.kind.is_empty() || draft.title.is_empty() || draft.body.is_empty() {
            set_error.set(Some("Kind, title and body are required.".to_string()));
            return;
        }

        let token_value = token.get_untracked();
        let tenant_value = tenant.get_untracked();
        let editing_node = editing_node_id.get_untracked();
        set_busy_key.set(Some(
            editing_node.clone().unwrap_or_else(|| "create".to_string()),
        ));
        spawn_local(async move {
            let save_result = match editing_node {
                Some(node_id) => {
                    api::update_node(token_value.clone(), tenant_value.clone(), node_id, draft)
                        .await
                }
                None => api::create_node(token_value.clone(), tenant_value.clone(), draft).await,
            };

            match save_result {
                Ok(mut node) => {
                    let status = node.status.to_ascii_uppercase();
                    if should_publish && status != "PUBLISHED" {
                        if let Ok(published) = api::publish_node(
                            token_value.clone(),
                            tenant_value.clone(),
                            node.id.clone(),
                        )
                        .await
                        {
                            node = published;
                        }
                    }
                    if !should_publish && status == "PUBLISHED" {
                        if let Ok(unpublished) = api::unpublish_node(
                            token_value.clone(),
                            tenant_value.clone(),
                            node.id.clone(),
                        )
                        .await
                        {
                            node = unpublished;
                        }
                    }
                    apply_node_to_form(
                        set_editing_node_id,
                        set_kind,
                        set_title,
                        set_slug,
                        set_excerpt,
                        set_body,
                        set_locale,
                        set_body_format,
                        set_publish_now,
                        &node,
                    );
                    set_refresh_nonce.update(|value| *value += 1);
                }
                Err(err) => set_error.set(Some(format!("Failed to save node: {err}"))),
            }
            set_busy_key.set(None);
        });
    };

    let archive_node = Callback::new(move |node_id: String| {
        let token_value = token.get_untracked();
        let tenant_value = tenant.get_untracked();
        set_busy_key.set(Some(format!("archive:{node_id}")));
        set_error.set(None);
        spawn_local(async move {
            match api::archive_node(token_value, tenant_value, node_id).await {
                Ok(_) => set_refresh_nonce.update(|value| *value += 1),
                Err(err) => set_error.set(Some(format!("Failed to archive node: {err}"))),
            }
            set_busy_key.set(None);
        });
    });

    let restore_node = Callback::new(move |node_id: String| {
        let token_value = token.get_untracked();
        let tenant_value = tenant.get_untracked();
        set_busy_key.set(Some(format!("restore:{node_id}")));
        set_error.set(None);
        spawn_local(async move {
            match api::restore_node(token_value, tenant_value, node_id).await {
                Ok(_) => set_refresh_nonce.update(|value| *value += 1),
                Err(err) => set_error.set(Some(format!("Failed to restore node: {err}"))),
            }
            set_busy_key.set(None);
        });
    });
    let delete_node = Callback::new(move |node_id: String| {
        let token_value = token.get_untracked();
        let tenant_value = tenant.get_untracked();
        set_busy_key.set(Some(format!("delete:{node_id}")));
        set_error.set(None);
        spawn_local(async move {
            match api::delete_node(token_value, tenant_value, node_id.clone()).await {
                Ok(true) => {
                    if editing_node_id.get_untracked().as_deref() == Some(node_id.as_str()) {
                        reset_form();
                    }
                    set_refresh_nonce.update(|value| *value += 1);
                }
                Ok(false) => set_error.set(Some("Delete node returned false.".to_string())),
                Err(err) => set_error.set(Some(format!("Failed to delete node: {err}"))),
            }
            set_busy_key.set(None);
        });
    });

    let toggle_publish = Callback::new(
        move |(node_id, publish, requested_locale): (String, bool, String)| {
            let token_value = token.get_untracked();
            let tenant_value = tenant.get_untracked();
            set_busy_key.set(Some(format!("publish:{node_id}")));
            set_error.set(None);
            spawn_local(async move {
                let result = if publish {
                    api::publish_node(token_value.clone(), tenant_value.clone(), node_id.clone())
                        .await
                } else {
                    api::unpublish_node(token_value.clone(), tenant_value.clone(), node_id.clone())
                        .await
                };
                match result {
                    Ok(node) => {
                        if editing_node_id.get_untracked().as_deref() == Some(node.id.as_str()) {
                            if let Ok(Some(full_node)) = api::fetch_node(
                                token.get_untracked(),
                                tenant.get_untracked(),
                                node.id.clone(),
                                Some(requested_locale),
                            )
                            .await
                            {
                                apply_node_to_form(
                                    set_editing_node_id,
                                    set_kind,
                                    set_title,
                                    set_slug,
                                    set_excerpt,
                                    set_body,
                                    set_locale,
                                    set_body_format,
                                    set_publish_now,
                                    &full_node,
                                );
                            }
                        }
                        set_refresh_nonce.update(|value| *value += 1);
                    }
                    Err(err) => {
                        set_error.set(Some(format!("Failed to change publish status: {err}")))
                    }
                }
                set_busy_key.set(None);
            });
        },
    );

    view! {
        <div class="space-y-6">
            <header class="rounded-2xl border border-border bg-card p-6 shadow-sm">
                <span class="inline-flex items-center rounded-full border border-border px-3 py-1 text-xs font-medium text-muted-foreground">"content"</span>
                <h1 class="mt-3 text-2xl font-semibold text-card-foreground">"Content Studio"</h1>
                <p class="mt-2 text-sm text-muted-foreground">"Publishable module-owned admin UI for core content nodes."</p>
            </header>
            <section class="grid gap-6 xl:grid-cols-[minmax(0,1fr)_28rem]">
                <div class="rounded-2xl border border-border bg-card p-6 shadow-sm">
                    <div class="mb-4 flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
                        <div>
                            <h2 class="text-lg font-semibold text-card-foreground">"Nodes"</h2>
                            <p class="text-sm text-muted-foreground">"Loaded through the content module package itself."</p>
                        </div>
                        <input
                            class="rounded-xl border border-border bg-background px-3 py-2 text-sm text-foreground outline-none transition focus:border-primary"
                            prop:value=move || kind_filter.get()
                            on:input=move |ev| set_kind_filter.set(event_target_value(&ev))
                            placeholder="Filter kind"
                        />
                    </div>
                    <Suspense fallback=move || view! { <div class="h-32 animate-pulse rounded-2xl bg-muted"></div> }>
                        {move || nodes.get().map(|result| match result {
                            Ok(list) => view! {
                                <ContentNodesList
                                    items=list.items
                                    total=list.total
                                    editing_node_id=editing_node_id.get()
                                    busy_key=busy_key.get()
                                    on_edit=edit_node
                                    on_toggle_publish=toggle_publish
                                    on_archive=archive_node
                                    on_restore=restore_node
                                    on_delete=delete_node
                                />
                            }.into_any(),
                            Err(err) => view! {
                                <div class="rounded-xl border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
                                    {format!("Failed to load nodes: {err}")}
                                </div>
                            }.into_any(),
                        })}
                    </Suspense>
                </div>
                <section class="rounded-2xl border border-border bg-card p-6 shadow-sm">
                    <h2 class="text-lg font-semibold text-card-foreground">{move || if editing_node_id.get().is_some() { "Edit Node" } else { "Create Node" }}</h2>
                    <form class="mt-6 space-y-4" on:submit=submit_node>
                        <input class="w-full rounded-xl border border-border bg-background px-3 py-2 text-sm" prop:value=move || kind.get() on:input=move |ev| set_kind.set(event_target_value(&ev)) placeholder="Kind" />
                        <input class="w-full rounded-xl border border-border bg-background px-3 py-2 text-sm" prop:value=move || locale.get() on:input=move |ev| set_locale.set(event_target_value(&ev)) placeholder="Locale" />
                        <input class="w-full rounded-xl border border-border bg-background px-3 py-2 text-sm" prop:value=move || title.get() on:input=move |ev| set_title.set(event_target_value(&ev)) placeholder="Title" />
                        <input class="w-full rounded-xl border border-border bg-background px-3 py-2 text-sm" prop:value=move || slug.get() on:input=move |ev| set_slug.set(event_target_value(&ev)) placeholder="Slug" />
                        <textarea class="min-h-20 w-full rounded-xl border border-border bg-background px-3 py-2 text-sm" prop:value=move || excerpt.get() on:input=move |ev| set_excerpt.set(event_target_value(&ev)) placeholder="Excerpt"></textarea>
                        <input class="w-full rounded-xl border border-border bg-background px-3 py-2 text-sm" prop:value=move || body_format.get() on:input=move |ev| set_body_format.set(event_target_value(&ev)) placeholder="Body format" />
                        <label class="flex items-center gap-3 rounded-xl border border-border bg-background px-4 py-3 text-sm">
                            <input type="checkbox" prop:checked=move || publish_now.get() on:change=move |ev| set_publish_now.set(event_target_checked(&ev)) />
                            <span>"Publish after save"</span>
                        </label>
                        <textarea class="min-h-48 w-full rounded-xl border border-border bg-background px-3 py-2 font-mono text-sm" prop:value=move || body.get() on:input=move |ev| set_body.set(event_target_value(&ev)) placeholder="Body"></textarea>
                        {move || error.get().map(|value| view! {
                            <div class="rounded-xl border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">{value}</div>
                        })}
                        <div class="flex flex-wrap gap-3">
                            <button type="submit" class="rounded-xl bg-primary px-4 py-2 text-sm font-medium text-primary-foreground" disabled=move || busy_key.get().is_some()>{move || if editing_node_id.get().is_some() { "Save Node" } else { "Create Node" }}</button>
                            <button type="button" class="rounded-xl border border-border px-4 py-2 text-sm font-medium" on:click=move |_| reset_form()>"Reset"</button>
                        </div>
                    </form>
                </section>
            </section>
        </div>
    }
}

#[component]
fn ContentNodesList(
    items: Vec<NodeListItem>,
    total: u64,
    editing_node_id: Option<String>,
    busy_key: Option<String>,
    on_edit: Callback<(String, String)>,
    on_toggle_publish: Callback<(String, bool, String)>,
    on_archive: Callback<String>,
    on_restore: Callback<String>,
    on_delete: Callback<String>,
) -> impl IntoView {
    if items.is_empty() {
        return view! { <div class="rounded-2xl border border-dashed border-border p-6 text-sm text-muted-foreground">"No nodes found."</div> }.into_any();
    }
    view! {
        <div class="space-y-3">
            <div class="flex items-center justify-between gap-3">
                <h3 class="text-lg font-semibold text-card-foreground">"Node List"</h3>
                <span class="text-sm text-muted-foreground">{format!("{total} total")}</span>
            </div>
            {items.into_iter().map(|node| {
                let node_id = node.id.clone();
                let edit_locale = node.effective_locale.clone();
                let is_editing = editing_node_id.as_deref() == Some(node_id.as_str());
                let is_busy = busy_key.as_ref().map(|value| value.contains(node_id.as_str())).unwrap_or(false);
                let is_published = node.status.eq_ignore_ascii_case("published");
                let is_archived = node.status.eq_ignore_ascii_case("archived");
                view! {
                    <article class="rounded-2xl border border-border bg-background p-5">
                        <div class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
                            <div class="space-y-1">
                                <div class="text-xs uppercase tracking-[0.2em] text-muted-foreground">{format!("{} · {} · {}", node.kind, node.status, node.effective_locale)}</div>
                                <h4 class="text-base font-semibold text-foreground">{node.title.unwrap_or_else(|| "Untitled node".to_string())}</h4>
                                <p class="text-sm text-muted-foreground">{node.excerpt.unwrap_or_else(|| "No excerpt yet.".to_string())}</p>
                            </div>
                            <div class="flex flex-wrap gap-2">
                                <button type="button" class="rounded-xl border border-border px-3 py-2 text-sm" on:click={ let node_id = node_id.clone(); let edit_locale = edit_locale.clone(); move |_| on_edit.run((node_id.clone(), edit_locale.clone())) } disabled=is_busy>{if is_editing { "Editing" } else { "Edit" }}</button>
                                <button type="button" class="rounded-xl border border-border px-3 py-2 text-sm" on:click={ let node_id = node_id.clone(); let edit_locale = edit_locale.clone(); move |_| on_toggle_publish.run((node_id.clone(), !is_published, edit_locale.clone())) } disabled=is_busy || is_archived>{if is_published { "Unpublish" } else { "Publish" }}</button>
                                <button type="button" class="rounded-xl border border-border px-3 py-2 text-sm" on:click={ let node_id = node_id.clone(); move |_| if is_archived { on_restore.run(node_id.clone()) } else { on_archive.run(node_id.clone()) } } disabled=is_busy>{if is_archived { "Restore" } else { "Archive" }}</button>
                                <button type="button" class="rounded-xl border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive" on:click={ let node_id = node_id.clone(); move |_| on_delete.run(node_id.clone()) } disabled=is_busy>"Delete"</button>
                            </div>
                        </div>
                    </article>
                }
            }).collect_view()}
        </div>
    }.into_any()
}

fn apply_node_to_form(
    set_editing_node_id: WriteSignal<Option<String>>,
    set_kind: WriteSignal<String>,
    set_title: WriteSignal<String>,
    set_slug: WriteSignal<String>,
    set_excerpt: WriteSignal<String>,
    set_body: WriteSignal<String>,
    set_locale: WriteSignal<String>,
    set_body_format: WriteSignal<String>,
    set_publish_now: WriteSignal<bool>,
    node: &NodeDetail,
) {
    let translation = node.translation.clone();
    let body = node.body.clone();
    let locale = translation
        .as_ref()
        .map(|value| value.locale.clone())
        .or_else(|| body.as_ref().map(|value| value.locale.clone()))
        .unwrap_or_else(|| "en".to_string());
    set_editing_node_id.set(Some(node.id.clone()));
    set_kind.set(node.kind.clone());
    set_title.set(
        translation
            .as_ref()
            .and_then(|value| value.title.clone())
            .unwrap_or_default(),
    );
    set_slug.set(
        translation
            .as_ref()
            .and_then(|value| value.slug.clone())
            .unwrap_or_default(),
    );
    set_excerpt.set(
        translation
            .as_ref()
            .and_then(|value| value.excerpt.clone())
            .unwrap_or_default(),
    );
    set_body.set(
        body.as_ref()
            .and_then(|value| value.body.clone())
            .unwrap_or_default(),
    );
    set_locale.set(locale);
    set_body_format.set(
        body.map(|value| value.format)
            .unwrap_or_else(|| "markdown".to_string()),
    );
    set_publish_now.set(node.status.eq_ignore_ascii_case("published"));
}
