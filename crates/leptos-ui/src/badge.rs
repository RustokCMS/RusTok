use leptos::*;
use crate::types::BadgeVariant;

#[component]
pub fn Badge(
    #[prop(default = BadgeVariant::Default)] variant: BadgeVariant,
    #[prop(optional)] class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let base_classes = "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium";
    let variant_classes = variant.classes();
    let additional_classes = class.unwrap_or("");
    let full_class = format!("{} {} {}", base_classes, variant_classes, additional_classes);

    view! {
        <span class=full_class>
            {children()}
        </span>
    }
}
