use crate::entities::oauth_app::model::OAuthApp;
use crate::shared::ui::Button;
use leptos::prelude::*;

#[component]
pub fn RevokeAppDialog(app: OAuthApp, on_success: Callback<()>, on_cancel: Callback<()>) -> impl IntoView {
    let name = app.name.clone();
    view! {
        <div class="space-y-4">
            <h3 class="text-lg font-medium text-red-600">"Revoke OAuth Application"</h3>
            <p class="text-sm text-slate-500">"Revoke access for "<span class="font-semibold">{name}</span>"?"</p>
            <div class="flex items-center gap-2 pt-4">
                <Button on_click=move |_| on_success.run(())>"Revoke Application"</Button>
                <Button on_click=move |_| on_cancel.run(())>"Cancel"</Button>
            </div>
        </div>
    }
}
