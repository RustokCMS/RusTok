use leptos::*;

use crate::components::ui::Button;
use crate::providers::auth::use_auth;

#[component]
pub fn Dashboard() -> impl IntoView {
    let auth = use_auth();

    let stats = [
        ("Активные тенанты", "28", "+3 за неделю"),
        ("Модули в работе", "12", "Commerce, Blog, Tickets"),
        ("Время отклика API", "128ms", "−14% за 7 дней"),
        ("Задач в очереди", "7", "2 критичных"),
    ];

    let activity = [
        ("Новый тенант", "Nordic Supply", "2 минуты назад"),
        ("Модуль", "Commerce обновлён до v1.0.3", "20 минут назад"),
        ("Безопасность", "Обновлены роли редакторов", "1 час назад"),
        ("Контент", "Запущена публикация промо-страницы", "Сегодня"),
    ];

    let logout = move |_| {
        auth.set_token.set(None);
        auth.set_user.set(None);
    };

    view! {
        <section class="dashboard">
            <header class="dashboard-header">
                <div>
                    <span class="badge">"Dashboard"</span>
                    <h1>
                        {move || {
                            auth.user
                                .get()
                                .and_then(|user| user.name)
                                .unwrap_or_else(|| "Добро пожаловать, Админ".to_string())
                        }}
                    </h1>
                    <p style="margin:8px 0 0; color:#64748b;">
                        "Сводка системы RusToK: ключевые метрики и быстрый доступ к модулям."
                    </p>
                </div>
                <div class="dashboard-actions">
                    <Button on_click=logout class="ghost-button">
                        "Выйти"
                    </Button>
                    <Button on_click=move |_| {}>
                        "Создать тенант"
                    </Button>
                </div>
            </header>

            <div class="stats-grid">
                {stats
                    .iter()
                    .map(|(title, value, hint)| {
                        view! {
                            <div class="stat-card">
                                <h3>{*title}</h3>
                                <strong>{*value}</strong>
                                <p style="margin:8px 0 0; color:#94a3b8;">{*hint}</p>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>

            <div class="dashboard-panels">
                <div class="panel">
                    <h4>"Последняя активность"</h4>
                    {activity
                        .iter()
                        .map(|(title, detail, time)| {
                            view! {
                                <div class="activity-item">
                                    <div>
                                        <strong>{*title}</strong>
                                        <p style="margin:4px 0 0; color:#64748b;">{*detail}</p>
                                    </div>
                                    <span class="badge">{*time}</span>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
                <div class="panel">
                    <h4>"Быстрые действия"</h4>
                    <div class="quick-actions">
                        <button type="button">"Запустить аудит безопасности"</button>
                        <a href="/users">"Управление пользователями"</a>
                        <button type="button">"Проверить метрики API"</button>
                        <button type="button">"Сформировать отчёт по ролям"</button>
                    </div>
                </div>
            </div>
        </section>
    }
}
