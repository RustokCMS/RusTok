pub fn translate_en(key: &str) -> Option<&'static str> {
    match key {
        "app.dashboard.subtitle" => {
            Some("RusToK system summary: key metrics and quick module access.")
        }
        "app.dashboard.logout" => Some("Log out"),
        "app.dashboard.createTenant" => Some("Create tenant"),
        "app.dashboard.stats.tenants" => Some("Active tenants"),
        "app.dashboard.stats.tenantsHint" => Some("+3 in a week"),
        "app.dashboard.stats.modules" => Some("Modules in work"),
        "app.dashboard.stats.modulesHint" => Some("Commerce, Blog, Tickets"),
        "app.dashboard.stats.latency" => Some("API response time"),
        "app.dashboard.stats.latencyHint" => Some("−14% in 7 days"),
        "app.dashboard.stats.queue" => Some("Queue tasks"),
        "app.dashboard.stats.queueHint" => Some("2 critical"),
        "app.dashboard.activity.title" => Some("Latest activity"),
        "app.dashboard.activity.tenant" => Some("New tenant"),
        "app.dashboard.activity.tenantDetail" => Some("Nordic Supply"),
        "app.dashboard.activity.tenantTime" => Some("2 minutes ago"),
        "app.dashboard.activity.module" => Some("Module"),
        "app.dashboard.activity.moduleDetail" => Some("Commerce updated to v1.0.3"),
        "app.dashboard.activity.moduleTime" => Some("20 minutes ago"),
        "app.dashboard.activity.security" => Some("Security"),
        "app.dashboard.activity.securityDetail" => Some("Editor roles updated"),
        "app.dashboard.activity.securityTime" => Some("1 hour ago"),
        "app.dashboard.activity.content" => Some("Content"),
        "app.dashboard.activity.contentDetail" => Some("Promo page published"),
        "app.dashboard.activity.contentTime" => Some("Today"),
        "app.dashboard.quick.title" => Some("Quick actions"),
        "app.dashboard.quick.security" => Some("Run security audit"),
        "app.dashboard.quick.users" => Some("User management"),
        "app.dashboard.quick.profile" => Some("Profile & access"),
        "app.dashboard.quick.metrics" => Some("Check API metrics"),
        "app.dashboard.quick.roles" => Some("Generate role report"),
        _ => None,
    }
}

pub fn translate_ru(key: &str) -> Option<&'static str> {
    match key {
        "app.dashboard.subtitle" => {
            Some("Сводка системы RusToK: ключевые метрики и быстрый доступ к модулям.")
        }
        "app.dashboard.logout" => Some("Выйти"),
        "app.dashboard.createTenant" => Some("Создать тенант"),
        "app.dashboard.stats.tenants" => Some("Активные тенанты"),
        "app.dashboard.stats.tenantsHint" => Some("+3 за неделю"),
        "app.dashboard.stats.modules" => Some("Модули в работе"),
        "app.dashboard.stats.modulesHint" => Some("Commerce, Blog, Tickets"),
        "app.dashboard.stats.latency" => Some("Время отклика API"),
        "app.dashboard.stats.latencyHint" => Some("−14% за 7 дней"),
        "app.dashboard.stats.queue" => Some("Задач в очереди"),
        "app.dashboard.stats.queueHint" => Some("2 критичных"),
        "app.dashboard.activity.title" => Some("Последняя активность"),
        "app.dashboard.activity.tenant" => Some("Новый тенант"),
        "app.dashboard.activity.tenantDetail" => Some("Nordic Supply"),
        "app.dashboard.activity.tenantTime" => Some("2 минуты назад"),
        "app.dashboard.activity.module" => Some("Модуль"),
        "app.dashboard.activity.moduleDetail" => Some("Commerce обновлён до v1.0.3"),
        "app.dashboard.activity.moduleTime" => Some("20 минут назад"),
        "app.dashboard.activity.security" => Some("Безопасность"),
        "app.dashboard.activity.securityDetail" => Some("Обновлены роли редакторов"),
        "app.dashboard.activity.securityTime" => Some("1 час назад"),
        "app.dashboard.activity.content" => Some("Контент"),
        "app.dashboard.activity.contentDetail" => Some("Запущена публикация промо-страницы"),
        "app.dashboard.activity.contentTime" => Some("Сегодня"),
        "app.dashboard.quick.title" => Some("Быстрые действия"),
        "app.dashboard.quick.security" => Some("Запустить аудит безопасности"),
        "app.dashboard.quick.users" => Some("Управление пользователями"),
        "app.dashboard.quick.profile" => Some("Профиль и доступ"),
        "app.dashboard.quick.metrics" => Some("Проверить метрики API"),
        "app.dashboard.quick.roles" => Some("Сформировать отчёт по ролям"),
        _ => None,
    }
}
