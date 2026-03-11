use crate::entities::oauth_app::model::OAuthApp;
use crate::shared::ui::Button;
use leptos::prelude::*;

#[component]
pub fn RevokeAppDialog(
    app: OAuthApp,
    on_success: impl Fn() + 'static + Clone,
    on_cancel: impl Fn() + 'static + Clone,
) -> impl IntoView {
    let name = app.name.clone();
    let revoke_action = Action::new(move |_: &()| {
        let on_success = on_success.clone();
        async move { on_success() }
    });

    view! {
        <div class="space-y-4">
            <h3 class="text-lg font-medium text-red-600">"Revoke OAuth Application"</h3>
            <p class="text-sm text-slate-500">"Revoke access for "<span class="font-semibold">{name}</span>"?"</p>
            <div class="flex items-center gap-2 pt-4">
                <Button on_click=move |_| { revoke_action.dispatch(()); }>
                    "Revoke Application"
                </Button>
                <Button on_click=move |_| on_cancel()>
                    "Cancel"
                </Button>
            </div>
        </div>
    }
}
