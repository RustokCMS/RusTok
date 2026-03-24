mod api;
mod model;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_auth::hooks::{use_tenant, use_token};
use leptos_router::components::A;
use rustok_api::UiRouteContext;

use crate::model::{WorkflowStatus, WorkflowSummary, WorkflowTemplateDto};

#[component]
pub fn WorkflowAdmin() -> impl IntoView {
    let token = use_token();
    let tenant = use_tenant();
    let (refresh_nonce, set_refresh_nonce) = signal(0_u64);
    let route_context = use_context::<UiRouteContext>().unwrap_or_default();
    let route_segment = route_context
        .route_segment
        .clone()
        .unwrap_or_else(|| "workflow".to_string());
    let showing_templates = route_context.subpath_matches("templates");

    let workflows_resource = Resource::new(
        move || (token.get(), tenant.get(), refresh_nonce.get()),
        move |(token_value, tenant_value, _)| async move {
            api::fetch_workflows(token_value, tenant_value).await
        },
    );

    view! {
        <div class="space-y-6">
            <header class="flex flex-col gap-4 rounded-2xl border border-border bg-card p-6 shadow-sm lg:flex-row lg:items-start lg:justify-between">
                <div class="space-y-2">
                    <span class="inline-flex items-center rounded-full border border-border px-3 py-1 text-xs font-medium text-muted-foreground">
                        "workflow"
                    </span>
                    <h1 class="text-2xl font-semibold text-card-foreground">"Workflow Automation"</h1>
                    <p class="max-w-2xl text-sm text-muted-foreground">
                        "Module-owned admin surface for workflow templates and automation overview."
                    </p>
                </div>
                <div class="flex flex-wrap gap-2">
                    <A
                        href=if showing_templates {
                            format!("/modules/{route_segment}")
                        } else {
                            format!("/modules/{route_segment}/templates")
                        }
                        attr:class="inline-flex items-center gap-2 rounded-lg border border-border px-4 py-2 text-sm font-medium text-foreground transition hover:bg-accent hover:text-accent-foreground"
                    >
                        {if showing_templates { "Open overview" } else { "Open templates" }}
                    </A>
                    <A
                        href="/workflows"
                        attr:class="inline-flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition hover:bg-primary/90"
                    >
                        "Open legacy detail flow"
                    </A>
                </div>
            </header>

            <Show when=move || showing_templates>
                <div class="rounded-2xl border border-border bg-card p-6 shadow-sm">
                    <TemplateGallery
                        token=token.get()
                        tenant_slug=tenant.get()
                        on_created=Callback::new(move |_| {
                            set_refresh_nonce.update(|value| *value += 1);
                        })
                    />
                </div>
            </Show>

            <section class="rounded-2xl border border-border bg-card p-6 shadow-sm">
                <div class="mb-4 flex items-center justify-between gap-3">
                    <div>
                        <h2 class="text-lg font-semibold text-card-foreground">"Workflows"</h2>
                        <p class="text-sm text-muted-foreground">
                            "This root page is now published from the module crate instead of being wired manually in apps/admin."
                        </p>
                    </div>
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
                        workflows_resource.get().map(|result| {
                            match result {
                                Ok(workflows) => view! {
                                    <WorkflowList workflows />
                                }.into_any(),
                                Err(err) => view! {
                                    <div class="rounded-xl border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
                                        {format!("Failed to load workflows: {err}")}
                                    </div>
                                }.into_any(),
                            }
                        })
                    }}
                </Suspense>
            </section>
        </div>
    }
}

#[component]
fn WorkflowList(workflows: Vec<WorkflowSummary>) -> impl IntoView {
    if workflows.is_empty() {
        return view! {
            <div class="rounded-xl border border-dashed border-border p-12 text-center">
                <p class="text-sm text-muted-foreground">
                    "No workflows yet. Start with a template or open the legacy workflow screens."
                </p>
                <A
                    href="/workflows"
                    attr:class="mt-4 inline-flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground"
                >
                    "Open workflows"
                </A>
            </div>
        }
        .into_any();
    }

    view! {
        <div class="overflow-hidden rounded-xl border border-border">
            <table class="w-full text-sm">
                <thead class="border-b border-border bg-muted/50">
                    <tr>
                        <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-muted-foreground">"Name"</th>
                        <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-muted-foreground">"Status"</th>
                        <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-muted-foreground">"Failures"</th>
                        <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider text-muted-foreground">"Updated"</th>
                        <th class="px-4 py-3"></th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-border">
                    {workflows.into_iter().map(|workflow| {
                        let detail_href = format!("/workflows/{}", workflow.id);
                        view! {
                            <tr class="transition-colors hover:bg-muted/30">
                                <td class="px-4 py-3 font-medium text-foreground">{workflow.name}</td>
                                <td class="px-4 py-3">
                                    <StatusBadge status=workflow.status />
                                </td>
                                <td class="px-4 py-3 text-muted-foreground">{workflow.failure_count}</td>
                                <td class="px-4 py-3 text-xs text-muted-foreground">{workflow.updated_at}</td>
                                <td class="px-4 py-3 text-right">
                                    <A
                                        href=detail_href
                                        attr:class="text-xs font-medium text-primary hover:underline"
                                    >
                                        "Legacy details ->"
                                    </A>
                                </td>
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>
        </div>
    }
    .into_any()
}

#[component]
fn StatusBadge(status: WorkflowStatus) -> impl IntoView {
    let (label, class_name) = match status {
        WorkflowStatus::Active => (
            "Active",
            "bg-emerald-50 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-400",
        ),
        WorkflowStatus::Paused => (
            "Paused",
            "bg-yellow-50 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400",
        ),
        WorkflowStatus::Archived => ("Archived", "bg-muted text-muted-foreground"),
        WorkflowStatus::Draft | WorkflowStatus::Unknown => ("Draft", "bg-primary/10 text-primary"),
    };

    view! {
        <span class=format!("inline-flex rounded-full px-2.5 py-0.5 text-xs font-semibold {class_name}")>
            {label}
        </span>
    }
}

#[component]
fn TemplateGallery(
    token: Option<String>,
    tenant_slug: Option<String>,
    on_created: Callback<String>,
) -> impl IntoView {
    let (pending_id, set_pending_id) = signal(Option::<String>::None);
    let (name_input, set_name_input) = signal(String::new());

    let token_for_templates = token.clone();
    let tenant_for_templates = tenant_slug.clone();
    let templates_resource = Resource::new_blocking(
        move || (token_for_templates.clone(), tenant_for_templates.clone()),
        move |(token_value, tenant_value)| async move {
            api::fetch_templates(token_value, tenant_value).await
        },
    );

    view! {
        <div class="space-y-4">
            <div class="space-y-1">
                <h2 class="text-lg font-semibold text-card-foreground">"Workflow Templates"</h2>
                <p class="text-sm text-muted-foreground">
                    "Create a workflow from a starter template directly from the module package."
                </p>
            </div>

            <Suspense fallback=move || view! {
                <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
                    {(0..4).map(|_| view! {
                        <div class="h-28 animate-pulse rounded-xl bg-muted"></div>
                    }).collect_view()}
                </div>
            }>
                {move || {
                    let pending = pending_id.get();
                    let current_name = name_input.get();

                    templates_resource.get().map(|result| {
                        match result {
                            Ok(templates) => view! {
                                <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
                                    {templates.into_iter().map(|template| {
                                        let template_id = template.id.clone();
                                        let token_for_request = token.clone();
                                        let tenant_for_request = tenant_slug.clone();
                                        let is_pending = pending.as_deref() == Some(template_id.as_str());

                                        view! {
                                            <TemplateCard
                                                template=template
                                                is_pending=is_pending
                                                name=current_name.clone()
                                                on_name_change=Callback::new(move |value| set_name_input.set(value))
                                                on_use=Callback::new(move |_| {
                                                    let token_value = token_for_request.clone();
                                                    let tenant_value = tenant_for_request.clone();
                                                    let template_id = template_id.clone();
                                                    let workflow_name = {
                                                        let entered = name_input.get_untracked();
                                                        if entered.trim().is_empty() {
                                                            format!("Workflow from {}", template_id)
                                                        } else {
                                                            entered
                                                        }
                                                    };
                                                    set_pending_id.set(Some(template_id.clone()));

                                                    spawn_local(async move {
                                                        match api::create_from_template(
                                                            token_value,
                                                            tenant_value,
                                                            template_id.clone(),
                                                            workflow_name,
                                                        ).await {
                                                            Ok(workflow_id) => {
                                                                set_pending_id.set(None);
                                                                set_name_input.set(String::new());
                                                                on_created.run(workflow_id);
                                                            }
                                                            Err(_) => {
                                                                set_pending_id.set(None);
                                                            }
                                                        }
                                                    });
                                                })
                                            />
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any(),
                            Err(err) => view! {
                                <div class="rounded-xl border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
                                    {format!("Failed to load templates: {err}")}
                                </div>
                            }.into_any(),
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn TemplateCard(
    template: WorkflowTemplateDto,
    is_pending: bool,
    name: String,
    on_name_change: Callback<String>,
    on_use: Callback<()>,
) -> impl IntoView {
    let category_color = match template.category.as_str() {
        "content" => "bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300",
        "commerce" => "bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-300",
        "auth" => "bg-purple-100 text-purple-700 dark:bg-purple-900/40 dark:text-purple-300",
        "reporting" => "bg-yellow-100 text-yellow-700 dark:bg-yellow-900/40 dark:text-yellow-300",
        "integrations" => {
            "bg-orange-100 text-orange-700 dark:bg-orange-900/40 dark:text-orange-300"
        }
        _ => "bg-muted text-muted-foreground",
    };

    view! {
        <div class="flex flex-col gap-2 rounded-xl border border-border bg-background p-4">
            <div class="flex items-start justify-between gap-2">
                <span class=format!("rounded-full px-2 py-0.5 text-xs font-medium {category_color}")>
                    {template.category.clone()}
                </span>
            </div>
            <h3 class="text-sm font-semibold text-foreground leading-tight">{template.name.clone()}</h3>
            <p class="text-xs text-muted-foreground line-clamp-2">{template.description.clone()}</p>
            <div class="mt-auto flex gap-2 pt-2">
                <input
                    type="text"
                    placeholder="Workflow name..."
                    class="flex-1 min-w-0 rounded-lg border border-input bg-background px-2 py-1 text-xs"
                    prop:value=name
                    on:input=move |ev| on_name_change.run(event_target_value(&ev))
                />
                <button
                    on:click=move |_| on_use.run(())
                    disabled=is_pending
                    class="rounded-lg bg-primary px-3 py-1 text-xs font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
                >
                    {if is_pending { "..." } else { "Use" }}
                </button>
            </div>
        </div>
    }
}
