use leptos::prelude::*;

#[component]
pub fn StatsCard(
    #[prop(into)] title: String,
    #[prop(into)] value: String,
    #[prop(into)] icon: AnyView,
    #[prop(into)] trend: String,
    #[prop(optional, into)] trend_up: Option<bool>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    view! {
        <div class=format!(
            "rounded-xl border bg-card text-card-foreground shadow transition-all hover:-translate-y-1 hover:shadow-md p-6 {}",
            class
        )>
            <div class="flex items-start justify-between">
                <div>
                    <p class="text-sm font-medium text-muted-foreground">{title}</p>
                    <h3 class="mt-2 text-3xl font-bold">{value}</h3>
                </div>
                <div class="rounded-lg bg-primary/10 p-3 text-primary">
                    {icon}
                </div>
            </div>
            {
                let is_up = trend_up.unwrap_or(true);
                let color_class = if is_up { "text-emerald-600" } else { "text-destructive" };
                view! {
                    <div class="mt-4 flex items-center gap-2">
                        <span class=format!("flex items-center text-sm font-medium {}", color_class)>
                            {if is_up { "+" } else { "" }} {trend}
                        </span>
                        <span class="text-sm text-muted-foreground">"vs last month"</span>
                    </div>
                }
            }
        </div>
    }
}
