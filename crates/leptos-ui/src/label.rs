use leptos::*;

#[component]
pub fn Label(
    #[prop(default = false)] required: bool,
    #[prop(optional)] r#for: Option<&'static str>,
    #[prop(optional)] class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let base_classes = "block text-sm font-medium text-gray-700";
    let additional_classes = class.unwrap_or("");
    let full_class = format!("{} {}", base_classes, additional_classes);

    view! {
        <label
            for=r#for.unwrap_or("")
            class=full_class
        >
            {children()}
            {move || if required {
                view! { <span class="text-red-500 ml-1">"*"</span> }.into_view()
            } else {
                ().into_view()
            }}
        </label>
    }
}
