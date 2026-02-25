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
        <div class=format!("rounded-2xl bg-white p-6 shadow-[0_18px_36px_rgba(15,23,42,0.08)] transition-all hover:-translate-y-1 hover:shadow-[0_22px_42px_rgba(15,23,42,0.12)] {}", class)>
            <div class="flex items-start justify-between">
                <div>
                    <p class="text-sm font-medium text-slate-500">{title}</p>
                    <h3 class="mt-2 text-3xl font-bold text-slate-900">{value}</h3>
                </div>
                <div class="rounded-xl bg-blue-50 p-3 text-blue-600">
                    {icon}
                </div>
            </div>
            {
                let is_up = trend_up.unwrap_or(true);
                let color_class = if is_up { "text-emerald-600" } else { "text-rose-600" };
                let _bg_class = if is_up { "bg-emerald-50" } else { "bg-rose-50" };
                view! {
                    <div class="mt-4 flex items-center gap-2">
                        <span class=format!("flex items-center text-sm font-medium {}", color_class)>
                            {if is_up { "+" } else { "" }} {trend}
                        </span>
                        <span class="text-sm text-slate-400">"vs last month"</span>
                    </div>
                }
            }
        </div>
    }
}
