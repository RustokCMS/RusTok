use leptos::*;

#[component]
pub fn Card(
    #[prop(optional)] class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let base_classes = "bg-white rounded-lg border border-gray-200 shadow-sm";
    let additional_classes = class.unwrap_or("");
    let full_class = format!("{} {}", base_classes, additional_classes);

    view! {
        <div class=full_class>
            {children()}
        </div>
    }
}

#[component]
pub fn CardHeader(
    #[prop(optional)] class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let base_classes = "px-6 py-4 border-b border-gray-200";
    let additional_classes = class.unwrap_or("");
    let full_class = format!("{} {}", base_classes, additional_classes);

    view! {
        <div class=full_class>
            {children()}
        </div>
    }
}

#[component]
pub fn CardContent(
    #[prop(optional)] class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let base_classes = "px-6 py-4";
    let additional_classes = class.unwrap_or("");
    let full_class = format!("{} {}", base_classes, additional_classes);

    view! {
        <div class=full_class>
            {children()}
        </div>
    }
}

#[component]
pub fn CardFooter(
    #[prop(optional)] class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let base_classes = "px-6 py-4 border-t border-gray-200";
    let additional_classes = class.unwrap_or("");
    let full_class = format!("{} {}", base_classes, additional_classes);

    view! {
        <div class=full_class>
            {children()}
        </div>
    }
}
