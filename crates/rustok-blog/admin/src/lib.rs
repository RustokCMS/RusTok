mod api;
mod model;

use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_auth::hooks::{use_tenant, use_token};

use crate::model::{BlogPostDetail, BlogPostDraft, BlogPostListItem};

#[component]
pub fn BlogAdmin() -> impl IntoView {
    let token = use_token();
    let tenant = use_tenant();

    let (refresh_nonce, set_refresh_nonce) = signal(0_u64);
    let (editing_post_id, set_editing_post_id) = signal(Option::<String>::None);
    let (title, set_title) = signal(String::new());
    let (slug, set_slug) = signal(String::new());
    let (excerpt, set_excerpt) = signal(String::new());
    let (body, set_body) = signal(String::new());
    let (locale, set_locale) = signal("en".to_string());
    let (body_format, set_body_format) = signal("markdown".to_string());
    let (tags_input, set_tags_input) = signal(String::new());
    let (publish_now, set_publish_now) = signal(false);
    let (busy_key, set_busy_key) = signal(Option::<String>::None);
    let (submit_error, set_submit_error) = signal(Option::<String>::None);

    let posts_resource = Resource::new(
        move || (token.get(), tenant.get(), refresh_nonce.get(), locale.get()),
        move |(token_value, tenant_value, _, locale_value)| async move {
            api::fetch_posts(token_value, tenant_value, Some(locale_value)).await
        },
    );

    let edit_post = Callback::new(move |(post_id, requested_locale): (String, String)| {
        let token_value = token.get_untracked();
        let tenant_value = tenant.get_untracked();
        set_submit_error.set(None);
        set_busy_key.set(Some(format!("edit:{post_id}")));

        spawn_local(async move {
            match api::fetch_post(
                token_value,
                tenant_value,
                post_id.clone(),
                Some(requested_locale),
            )
            .await
            {
                Ok(Some(post)) => {
                    apply_post_to_form(
                        set_editing_post_id,
                        set_title,
                        set_slug,
                        set_excerpt,
                        set_body,
                        set_locale,
                        set_body_format,
                        set_tags_input,
                        set_publish_now,
                        &post,
                    );
                }
                Ok(None) => {
                    set_submit_error.set(Some("Post not found for editing.".to_string()));
                }
                Err(err) => {
                    set_submit_error.set(Some(format!("Failed to load post: {err}")));
                }
            }

            set_busy_key.set(None);
        });
    });

    let submit_post = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_submit_error.set(None);

        let draft = BlogPostDraft {
            locale: locale.get_untracked(),
            title: title.get_untracked().trim().to_string(),
            slug: slug.get_untracked().trim().to_string(),
            excerpt: excerpt.get_untracked().trim().to_string(),
            body: body.get_untracked().trim().to_string(),
            body_format: body_format.get_untracked(),
            publish: publish_now.get_untracked(),
            tags: parse_tags(tags_input.get_untracked().as_str()),
        };

        if draft.title.is_empty() || draft.body.is_empty() {
            set_submit_error.set(Some(
                "Title and body are required to save a blog post.".to_string(),
            ));
            return;
        }

        let token_value = token.get_untracked();
        let tenant_value = tenant.get_untracked();
        let editing_post = editing_post_id.get_untracked();
        set_busy_key.set(Some(if let Some(post_id) = editing_post.as_ref() {
            format!("save:{post_id}")
        } else {
            "create".to_string()
        }));

        spawn_local(async move {
            let result = match editing_post {
                Some(post_id) => api::update_post(token_value, tenant_value, post_id, draft).await,
                None => api::create_post(token_value, tenant_value, draft).await,
            };

            match result {
                Ok(post) => {
                    apply_post_to_form(
                        set_editing_post_id,
                        set_title,
                        set_slug,
                        set_excerpt,
                        set_body,
                        set_locale,
                        set_body_format,
                        set_tags_input,
                        set_publish_now,
                        &post,
                    );
                    set_refresh_nonce.update(|value| *value += 1);
                }
                Err(err) => {
                    set_submit_error.set(Some(format!("Failed to save post: {err}")));
                }
            }

            set_busy_key.set(None);
        });
    };

    let toggle_publish = Callback::new(
        move |(post_id, publish, post_locale): (String, bool, String)| {
            let token_value = token.get_untracked();
            let tenant_value = tenant.get_untracked();
            set_submit_error.set(None);
            set_busy_key.set(Some(format!("publish:{post_id}")));

            spawn_local(async move {
                let result = if publish {
                    api::publish_post(
                        token_value,
                        tenant_value,
                        post_id.clone(),
                        Some(post_locale.clone()),
                    )
                    .await
                } else {
                    api::unpublish_post(
                        token_value,
                        tenant_value,
                        post_id.clone(),
                        Some(post_locale),
                    )
                    .await
                };

                match result {
                    Ok(post) => {
                        if editing_post_id.get_untracked().as_deref() == Some(post.id.as_str()) {
                            apply_post_to_form(
                                set_editing_post_id,
                                set_title,
                                set_slug,
                                set_excerpt,
                                set_body,
                                set_locale,
                                set_body_format,
                                set_tags_input,
                                set_publish_now,
                                &post,
                            );
                        }
                        set_refresh_nonce.update(|value| *value += 1);
                    }
                    Err(err) => {
                        set_submit_error.set(Some(format!("Failed to update post status: {err}")));
                    }
                }

                set_busy_key.set(None);
            });
        },
    );

    let archive_post = Callback::new(move |(post_id, post_locale): (String, String)| {
        let token_value = token.get_untracked();
        let tenant_value = tenant.get_untracked();
        set_submit_error.set(None);
        set_busy_key.set(Some(format!("archive:{post_id}")));

        spawn_local(async move {
            match api::archive_post(
                token_value,
                tenant_value,
                post_id.clone(),
                Some(post_locale),
            )
            .await
            {
                Ok(post) => {
                    if editing_post_id.get_untracked().as_deref() == Some(post.id.as_str()) {
                        apply_post_to_form(
                            set_editing_post_id,
                            set_title,
                            set_slug,
                            set_excerpt,
                            set_body,
                            set_locale,
                            set_body_format,
                            set_tags_input,
                            set_publish_now,
                            &post,
                        );
                    }
                    set_refresh_nonce.update(|value| *value += 1);
                }
                Err(err) => {
                    set_submit_error.set(Some(format!("Failed to archive post: {err}")));
                }
            }

            set_busy_key.set(None);
        });
    });

    let delete_post = Callback::new(move |post_id: String| {
        let token_value = token.get_untracked();
        let tenant_value = tenant.get_untracked();
        set_submit_error.set(None);
        set_busy_key.set(Some(format!("delete:{post_id}")));

        spawn_local(async move {
            match api::delete_post(token_value, tenant_value, post_id.clone()).await {
                Ok(true) => {
                    if editing_post_id.get_untracked().as_deref() == Some(post_id.as_str()) {
                        reset_form(
                            set_editing_post_id,
                            set_title,
                            set_slug,
                            set_excerpt,
                            set_body,
                            set_locale,
                            set_body_format,
                            set_tags_input,
                            set_publish_now,
                        );
                    }
                    set_refresh_nonce.update(|value| *value += 1);
                }
                Ok(false) => {
                    set_submit_error.set(Some(
                        "Delete post returned false. Unpublish or archive it first.".to_string(),
                    ));
                }
                Err(err) => {
                    set_submit_error.set(Some(format!("Failed to delete post: {err}")));
                }
            }

            set_busy_key.set(None);
        });
    });

    view! {
        <div class="space-y-6">
            <header class="flex flex-col gap-4 rounded-2xl border border-border bg-card p-6 shadow-sm lg:flex-row lg:items-start lg:justify-between">
                <div class="space-y-2">
                    <span class="inline-flex items-center rounded-full border border-border px-3 py-1 text-xs font-medium text-muted-foreground">
                        "blog"
                    </span>
                    <h1 class="text-2xl font-semibold text-card-foreground">"Blog Publishing"</h1>
                    <p class="max-w-2xl text-sm text-muted-foreground">
                        "Canonical module-owned CRUD flow for blog posts through the blog GraphQL contract."
                    </p>
                </div>
            </header>

            <section class="grid gap-6 xl:grid-cols-[minmax(0,1fr)_28rem]">
                <div class="rounded-2xl border border-border bg-card p-6 shadow-sm">
                    <div class="mb-4 flex items-end justify-between gap-4">
                        <div>
                            <h2 class="text-lg font-semibold text-card-foreground">"Posts"</h2>
                            <p class="text-sm text-muted-foreground">
                                "Loaded from rustok-blog-admin via GraphQL, not wired manually in apps/admin."
                            </p>
                        </div>
                        <label class="block space-y-2">
                            <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                                "Locale"
                            </span>
                            <input
                                type="text"
                                class="rounded-lg border border-input bg-background px-3 py-2 text-sm"
                                prop:value=locale
                                on:input=move |ev| set_locale.set(event_target_value(&ev))
                            />
                        </label>
                    </div>

                    <Suspense
                        fallback=move || view! {
                            <div class="space-y-2">
                                {(0..4).map(|_| view! {
                                    <div class="h-14 animate-pulse rounded-xl bg-muted"></div>
                                }).collect_view()}
                            </div>
                        }
                    >
                        {move || {
                            posts_resource.get().map(|result| {
                                match result {
                                    Ok(post_list) => view! {
                                        <BlogPostsTable
                                            items=post_list.items
                                            total=post_list.total
                                            editing_post_id=editing_post_id.get()
                                            busy_key=busy_key.get()
                                            on_edit=edit_post
                                            on_toggle_publish=toggle_publish
                                            on_archive=archive_post
                                            on_delete=delete_post
                                        />
                                    }.into_any(),
                                    Err(err) => view! {
                                        <div class="rounded-xl border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
                                            {format!("Failed to load posts: {err}")}
                                        </div>
                                    }.into_any(),
                                }
                            })
                        }}
                    </Suspense>
                </div>

                <section class="rounded-2xl border border-border bg-card p-6 shadow-sm">
                    <div class="space-y-1">
                        <h2 class="text-lg font-semibold text-card-foreground">
                            {move || {
                                if editing_post_id.get().is_some() {
                                    "Edit post"
                                } else {
                                    "Create post"
                                }
                            }}
                        </h2>
                        <p class="text-sm text-muted-foreground">
                            "The package owns both the list and the form. apps/admin only hosts the module route."
                        </p>
                    </div>

                    <Show when=move || editing_post_id.get().is_some()>
                        <div class="mt-4 flex items-center justify-between gap-3 rounded-xl border border-border bg-muted/30 px-4 py-3">
                            <div class="text-sm text-muted-foreground">
                                {move || {
                                    editing_post_id
                                        .get()
                                        .map(|post_id| format!("Editing post {post_id}"))
                                        .unwrap_or_default()
                                }}
                            </div>
                            <button
                                type="button"
                                class="text-xs font-medium text-primary hover:underline"
                                on:click=move |_| {
                                    reset_form(
                                        set_editing_post_id,
                                        set_title,
                                        set_slug,
                                        set_excerpt,
                                        set_body,
                                        set_locale,
                                        set_body_format,
                                        set_tags_input,
                                        set_publish_now,
                                    )
                                }
                            >
                                "Create new instead"
                            </button>
                        </div>
                    </Show>

                    <form class="mt-5 space-y-4" on:submit=submit_post>
                        <label class="block space-y-2">
                            <span class="text-sm font-medium text-card-foreground">"Title"</span>
                            <input
                                type="text"
                                class="w-full rounded-lg border border-input bg-background px-3 py-2 text-sm"
                                prop:value=title
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);
                                    if slug.get_untracked().trim().is_empty() {
                                        set_slug.set(slugify(value.as_str()));
                                    }
                                    set_title.set(value);
                                }
                            />
                        </label>

                        <label class="block space-y-2">
                            <span class="text-sm font-medium text-card-foreground">"Slug"</span>
                            <input
                                type="text"
                                class="w-full rounded-lg border border-input bg-background px-3 py-2 text-sm"
                                prop:value=slug
                                on:input=move |ev| set_slug.set(event_target_value(&ev))
                            />
                        </label>

                        <div class="grid gap-4 md:grid-cols-2">
                            <label class="block space-y-2">
                                <span class="text-sm font-medium text-card-foreground">"Locale"</span>
                                <input
                                    type="text"
                                    class="w-full rounded-lg border border-input bg-background px-3 py-2 text-sm"
                                    prop:value=locale
                                    on:input=move |ev| set_locale.set(event_target_value(&ev))
                                />
                            </label>

                            <label class="block space-y-2">
                                <span class="text-sm font-medium text-card-foreground">"Body format"</span>
                                <select
                                    class="w-full rounded-lg border border-input bg-background px-3 py-2 text-sm"
                                    prop:value=body_format
                                    on:change=move |ev| set_body_format.set(event_target_value(&ev))
                                >
                                    <option value="markdown">"markdown"</option>
                                    <option value="rt_json_v1">"rt_json_v1"</option>
                                </select>
                            </label>
                        </div>

                        <label class="block space-y-2">
                            <span class="text-sm font-medium text-card-foreground">"Excerpt"</span>
                            <textarea
                                class="min-h-24 w-full rounded-lg border border-input bg-background px-3 py-2 text-sm"
                                prop:value=excerpt
                                on:input=move |ev| set_excerpt.set(event_target_value(&ev))
                            />
                        </label>

                        <label class="block space-y-2">
                            <span class="text-sm font-medium text-card-foreground">"Body"</span>
                            <textarea
                                class="min-h-48 w-full rounded-lg border border-input bg-background px-3 py-2 text-sm"
                                prop:value=body
                                on:input=move |ev| set_body.set(event_target_value(&ev))
                            />
                        </label>

                        <Show when=move || body_format.get() != "markdown">
                            <div class="rounded-xl border border-amber-300/60 bg-amber-50 px-4 py-3 text-sm text-amber-900">
                                "This exemplar edits non-markdown content as raw serialized payload through the same GraphQL contract."
                            </div>
                        </Show>

                        <label class="block space-y-2">
                            <span class="text-sm font-medium text-card-foreground">"Tags"</span>
                            <input
                                type="text"
                                class="w-full rounded-lg border border-input bg-background px-3 py-2 text-sm"
                                placeholder="news, launch, release"
                                prop:value=tags_input
                                on:input=move |ev| set_tags_input.set(event_target_value(&ev))
                            />
                        </label>

                        <label class="flex items-center gap-2 text-sm text-card-foreground">
                            <input
                                type="checkbox"
                                prop:checked=publish_now
                                on:change=move |ev| set_publish_now.set(event_target_checked(&ev))
                            />
                            "Publish immediately"
                        </label>

                        <Show when=move || submit_error.get().is_some()>
                            <div class="rounded-xl border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
                                {move || submit_error.get().unwrap_or_default()}
                            </div>
                        </Show>

                        <button
                            type="submit"
                            class="inline-flex w-full items-center justify-center rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition hover:bg-primary/90 disabled:opacity-50"
                            disabled=move || {
                                busy_key.get().as_deref() == Some("create")
                                    || busy_key
                                        .get()
                                        .as_deref()
                                        .map(|key| key.starts_with("save:"))
                                        .unwrap_or(false)
                            }
                        >
                            {move || {
                                if busy_key.get().as_deref() == Some("create")
                                    || busy_key
                                        .get()
                                        .as_deref()
                                        .map(|key| key.starts_with("save:"))
                                        .unwrap_or(false)
                                {
                                    "Saving..."
                                } else if editing_post_id.get().is_some() {
                                    "Update post"
                                } else {
                                    "Create post"
                                }
                            }}
                        </button>
                    </form>
                </section>
            </section>
        </div>
    }
}

#[component]
fn BlogPostsTable(
    items: Vec<BlogPostListItem>,
    total: u64,
    editing_post_id: Option<String>,
    busy_key: Option<String>,
    on_edit: Callback<(String, String)>,
    on_toggle_publish: Callback<(String, bool, String)>,
    on_archive: Callback<(String, String)>,
    on_delete: Callback<String>,
) -> impl IntoView {
    if items.is_empty() {
        return view! {
            <div class="rounded-xl border border-dashed border-border p-12 text-center">
                <p class="text-sm text-muted-foreground">
                    "No posts yet. Create the first one from the module package form."
                </p>
            </div>
        }
        .into_any();
    }

    view! {
        <div class="space-y-4">
            <div class="text-sm text-muted-foreground">{format!("{total} post(s)")}</div>
            <div class="overflow-hidden rounded-xl border border-border">
                <table class="w-full text-sm">
                    <thead class="border-b border-border bg-muted/50">
                        <tr>
                            <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-muted-foreground">"Title"</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-muted-foreground">"Slug"</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-muted-foreground">"Status"</th>
                            <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-muted-foreground">"Locale"</th>
                            <th class="px-4 py-3"></th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-border">
                        {items
                            .into_iter()
                            .map(|post| {
                                let post_id = post.id.clone();
                                let post_id_edit = post_id.clone();
                                let post_id_publish = post_id.clone();
                                let post_id_archive = post_id.clone();
                                let post_id_delete = post_id.clone();
                                let post_slug = post.slug.clone().unwrap_or_else(|| "draft".to_string());
                                let post_locale = post.effective_locale.clone();
                                let post_locale_edit = post_locale.clone();
                                let post_locale_publish = post_locale.clone();
                                let post_locale_archive = post_locale.clone();
                                let is_editing = editing_post_id.as_deref() == Some(post_id.as_str());
                                let row_busy = busy_key
                                    .as_deref()
                                    .map(|key| key.contains(post_id.as_str()))
                                    .unwrap_or(false);
                                let is_published = post.status.eq_ignore_ascii_case("published");
                                let is_archived = post.status.eq_ignore_ascii_case("archived");

                                view! {
                                    <tr class="transition-colors hover:bg-muted/30">
                                        <td class="px-4 py-3 align-top">
                                            <div class="font-medium text-foreground">{post.title}</div>
                                            <div class="mt-1 text-xs text-muted-foreground">
                                                {post.excerpt.unwrap_or_else(|| "No excerpt".to_string())}
                                            </div>
                                        </td>
                                        <td class="px-4 py-3 align-top text-xs text-muted-foreground">{post_slug}</td>
                                        <td class="px-4 py-3 align-top">
                                            <StatusBadge status=post.status.clone() />
                                        </td>
                                        <td class="px-4 py-3 align-top text-xs text-muted-foreground">{post_locale.clone()}</td>
                                        <td class="px-4 py-3 align-top text-right">
                                            <div class="flex flex-wrap justify-end gap-2">
                                                <button
                                                    type="button"
                                                    class="text-xs font-medium text-primary hover:underline"
                                                    disabled=row_busy
                                                    on:click={
                                                        move |_| on_edit.run((post_id_edit.clone(), post_locale_edit.clone()))
                                                    }
                                                >
                                                    {if is_editing { "Editing" } else { "Edit" }}
                                                </button>
                                                <button
                                                    type="button"
                                                    class="text-xs font-medium text-primary hover:underline"
                                                    disabled=row_busy
                                                    on:click={
                                                        move |_| on_toggle_publish.run((post_id_publish.clone(), !is_published, post_locale_publish.clone()))
                                                    }
                                                >
                                                    {if is_published { "Unpublish" } else { "Publish" }}
                                                </button>
                                                {if is_archived {
                                                    view! { <></> }.into_any()
                                                } else {
                                                    view! {
                                                        <button
                                                            type="button"
                                                            class="text-xs font-medium text-primary hover:underline"
                                                            disabled=row_busy
                                                            on:click={
                                                                move |_| on_archive.run((post_id_archive.clone(), post_locale_archive.clone()))
                                                            }
                                                        >
                                                            "Archive"
                                                        </button>
                                                    }
                                                    .into_any()
                                                }}
                                                <button
                                                    type="button"
                                                    class="text-xs font-medium text-destructive hover:underline"
                                                    disabled=row_busy
                                                    on:click={
                                                        move |_| on_delete.run(post_id_delete.clone())
                                                    }
                                                >
                                                    "Delete"
                                                </button>
                                            </div>
                                        </td>
                                    </tr>
                                }
                            })
                            .collect_view()}
                    </tbody>
                </table>
            </div>
        </div>
    }
    .into_any()
}

#[component]
fn StatusBadge(status: String) -> impl IntoView {
    let class_name = if status.eq_ignore_ascii_case("published") {
        "bg-emerald-50 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-400"
    } else if status.eq_ignore_ascii_case("archived") {
        "bg-muted text-muted-foreground"
    } else {
        "bg-primary/10 text-primary"
    };

    view! {
        <span class=format!("inline-flex rounded-full px-2.5 py-0.5 text-xs font-semibold {class_name}")>
            {status}
        </span>
    }
}

fn parse_tags(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(ToString::to_string)
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn apply_post_to_form(
    set_editing_post_id: WriteSignal<Option<String>>,
    set_title: WriteSignal<String>,
    set_slug: WriteSignal<String>,
    set_excerpt: WriteSignal<String>,
    set_body: WriteSignal<String>,
    set_locale: WriteSignal<String>,
    set_body_format: WriteSignal<String>,
    set_tags_input: WriteSignal<String>,
    set_publish_now: WriteSignal<bool>,
    post: &BlogPostDetail,
) {
    set_editing_post_id.set(Some(post.id.clone()));
    set_locale.set(post.requested_locale.clone());
    set_title.set(post.title.clone());
    set_slug.set(post.slug.clone().unwrap_or_default());
    set_excerpt.set(post.excerpt.clone().unwrap_or_default());
    set_body.set(post.body.clone().unwrap_or_default());
    set_body_format.set(post.body_format.clone());
    set_tags_input.set(post.tags.join(", "));
    set_publish_now.set(post.status.eq_ignore_ascii_case("published"));
}

#[allow(clippy::too_many_arguments)]
fn reset_form(
    set_editing_post_id: WriteSignal<Option<String>>,
    set_title: WriteSignal<String>,
    set_slug: WriteSignal<String>,
    set_excerpt: WriteSignal<String>,
    set_body: WriteSignal<String>,
    set_locale: WriteSignal<String>,
    set_body_format: WriteSignal<String>,
    set_tags_input: WriteSignal<String>,
    set_publish_now: WriteSignal<bool>,
) {
    set_editing_post_id.set(None);
    set_title.set(String::new());
    set_slug.set(String::new());
    set_excerpt.set(String::new());
    set_body.set(String::new());
    set_locale.set("en".to_string());
    set_body_format.set("markdown".to_string());
    set_tags_input.set(String::new());
    set_publish_now.set(false);
}

fn slugify(input: &str) -> String {
    input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
