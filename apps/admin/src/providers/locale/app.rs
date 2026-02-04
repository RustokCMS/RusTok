pub fn translate_en(key: &str) -> Option<&'static str> {
    match key {
        "app.dashboard" => Some("Dashboard"),
        "app.users" => Some("Users"),
        _ => None,
    }
}

pub fn translate_ru(key: &str) -> Option<&'static str> {
    match key {
        "app.dashboard" => Some("Дашборд"),
        "app.users" => Some("Пользователи"),
        _ => None,
    }
}
