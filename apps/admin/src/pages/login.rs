use leptos::*;
use leptos_router::use_navigate;

use crate::components::ui::{Button, Input};
use crate::providers::auth::{use_auth, User};

#[component]
pub fn Login() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    let demo_mode = option_env!("RUSTOK_DEMO_MODE").is_some();
    let (tenant, set_tenant) = create_signal(String::new());
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (error, set_error) = create_signal(Option::<String>::None);

    let navigate_effect = navigate.clone();
    create_effect(move |_| {
        if auth.token.get().is_some() {
            navigate_effect("/dashboard", Default::default());
        }
    });

    let on_submit = move |_| {
        if tenant.get().is_empty() || email.get().is_empty() || password.get().is_empty() {
            set_error.set(Some("Заполните все поля".to_string()));
            return;
        }

        if !demo_mode {
            set_error.set(Some(
                "Демо-вход отключен. Используйте серверную аутентификацию или включите RUSTOK_DEMO_MODE."
                    .to_string(),
            ));
            return;
        }

        auth.set_token
            .set(Some(format!("demo-token:{}", tenant.get())));
        auth.set_user.set(Some(User {
            id: "demo".to_string(),
            email: email.get(),
            name: Some("Администратор".to_string()),
            role: "admin".to_string(),
        }));
        navigate("/dashboard", Default::default());
    };

    view! {
        <section class="auth-grid">
            <aside class="auth-visual">
                <span class="badge">"Admin Foundation"</span>
                <h1>"RusToK Control Center"</h1>
                <p>
                    "Управляйте тенантами, модулями и контентом в одном месте. "
                    "Настраиваемый доступ, быстрые действия и прозрачная аналитика."
                </p>
                <div>
                    <p><strong>"Входит в v1.0"</strong></p>
                    <p>"Логин, роли, графики активности и контроль модулей."</p>
                </div>
            </aside>
            <div class="auth-form">
                <div class="auth-card">
                    <div>
                        <h2>"Вход в админ-панель"</h2>
                        <p>"Введите рабочие данные для доступа к панели управления."</p>
                    </div>
                    <Show when=move || error.get().is_some()>
                        <div class="alert">{move || error.get().unwrap_or_default()}</div>
                    </Show>
                    <Input value=tenant set_value=set_tenant placeholder="demo" label="Tenant Slug" />
                    <Input value=email set_value=set_email placeholder="admin@rustok.io" label="Email" />
                    <Input value=password set_value=set_password placeholder="••••••••" type_="password" label="Пароль" />
                    <Button on_click=on_submit class="w-full">
                        "Продолжить"
                    </Button>
                    <Show when=move || demo_mode>
                        <a class="secondary-link" href="/dashboard">
                            "Перейти в демонстрационный дашборд →"
                        </a>
                    </Show>
                </div>
                <p style="margin:0; color:#64748b;">
                    "Нужен доступ? Напишите администратору безопасности для активации."
                </p>
            </div>
        </section>
    }
}
