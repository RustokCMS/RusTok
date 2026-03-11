use crate::entities::oauth_app::model::{AppType, OAuthApp};
use crate::shared::ui::{Button, Input, Textarea};
use leptos::prelude::*;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateOAuthAppResult {
    pub app: OAuthApp,
    pub client_secret: String,
}

#[component]
pub fn CreateAppForm(
    on_success: impl Fn(CreateOAuthAppResult) + 'static + Clone,
    on_cancel: impl Fn() + 'static + Clone,
) -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let (slug, set_slug) = signal(String::new());
    let (description, set_description) = signal(String::new());

    let create_action = Action::new(move |_: &()| {
        let name_val = name.get();
        let slug_val = slug.get();
        let desc_val = description.get();
        let on_success = on_success.clone();

        async move {
            info!("MOCK: Creating app {} ({})", name_val, slug_val);
            let mock_app = OAuthApp {
                id: uuid::Uuid::new_v4(),
                name: name_val,
                slug: slug_val,
                description: Some(desc_val),
                app_type: AppType::ThirdParty,
                client_id: uuid::Uuid::new_v4(),
                redirect_uris: vec![],
                scopes: vec![],
                grant_types: vec!["authorization_code".into()],
                manifest_ref: None,
                auto_created: false,
                is_active: true,
                active_token_count: 0,
                last_used_at: None,
                created_at: chrono::Utc::now(),
            };

            on_success(CreateOAuthAppResult {
                app: mock_app,
                client_secret: "sk_live_mock_secret_12345".into(),
            });
        }
    });

    view! {
        <div class="space-y-4">
            <h3 class="text-lg font-medium">"Create New Connected App"</h3>
            <Input value=name set_value=set_name placeholder="App name" />
            <Input value=slug set_value=set_slug placeholder="Slug/Identifier" />
            <Textarea
                prop:value=description
                on:input=move |ev| set_description.set(event_target_value(&ev))
            />
            <div class="flex items-center gap-2 pt-4">
                <Button on_click=move |_| { create_action.dispatch(()); }>
                    "Create App"
                </Button>
                <Button on_click=move |_| on_cancel()>
                    "Cancel"
                </Button>
            </div>
        </div>
    }
}
