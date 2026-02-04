pub fn translate_en(key: &str) -> Option<&'static str> {
    match key {
        "errors.auth.invalid_credentials" => Some("Invalid email or password."),
        "errors.auth.unauthorized" => Some("You are not authorized to perform this action."),
        "errors.http" => Some("Server error. Please try again."),
        "errors.network" => Some("Network error. Check your connection."),
        "errors.unknown" => Some("Something went wrong. Please try again."),
        _ => None,
    }
}

pub fn translate_ru(key: &str) -> Option<&'static str> {
    match key {
        "errors.auth.invalid_credentials" => Some("Неверный email или пароль."),
        "errors.auth.unauthorized" => Some("Недостаточно прав для выполнения действия."),
        "errors.http" => Some("Ошибка сервера. Попробуйте снова."),
        "errors.network" => Some("Сетевая ошибка. Проверьте соединение."),
        "errors.unknown" => Some("Что-то пошло не так. Попробуйте снова."),
        _ => None,
    }
}
