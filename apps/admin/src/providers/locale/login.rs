pub fn translate_en(key: &str) -> Option<&'static str> {
    match key {
        "login.badge" => Some("Admin Foundation"),
        "login.heroTitle" => Some("RusToK Control Center"),
        "login.heroSubtitle" => Some(
            "Manage tenants, modules, and content in one place. Configurable access, fast actions, and transparent analytics.",
        ),
        "login.heroListTitle" => Some("Included in v1.0"),
        "login.heroListSubtitle" => Some("Login, roles, activity charts, and module control."),
        "login.title" => Some("Sign in to admin"),
        "login.subtitle" => Some("Enter your credentials to access the control panel."),
        "login.tenantLabel" => Some("Tenant Slug"),
        "login.emailLabel" => Some("Email"),
        "login.passwordLabel" => Some("Password"),
        "login.submit" => Some("Continue"),
        "login.demoLink" => Some("Open demo dashboard →"),
        "login.footer" => Some("Need access? Contact a security administrator to activate."),
        "login.errorRequired" => Some("Please fill in all fields"),
        "login.errorDemoDisabled" => Some(
            "Demo login is disabled. Use server auth or enable RUSTOK_DEMO_MODE.",
        ),
        _ => None,
    }
}

pub fn translate_ru(key: &str) -> Option<&'static str> {
    match key {
        "login.badge" => Some("Admin Foundation"),
        "login.heroTitle" => Some("RusToK Control Center"),
        "login.heroSubtitle" => Some(
            "Управляйте тенантами, модулями и контентом в одном месте. Настраиваемый доступ, быстрые действия и прозрачная аналитика.",
        ),
        "login.heroListTitle" => Some("Входит в v1.0"),
        "login.heroListSubtitle" => Some("Логин, роли, графики активности и контроль модулей."),
        "login.title" => Some("Вход в админ-панель"),
        "login.subtitle" => Some("Введите рабочие данные для доступа к панели управления."),
        "login.tenantLabel" => Some("Tenant Slug"),
        "login.emailLabel" => Some("Email"),
        "login.passwordLabel" => Some("Пароль"),
        "login.submit" => Some("Продолжить"),
        "login.demoLink" => Some("Перейти в демонстрационный дашборд →"),
        "login.footer" => Some(
            "Нужен доступ? Напишите администратору безопасности для активации.",
        ),
        "login.errorRequired" => Some("Заполните все поля"),
        "login.errorDemoDisabled" => Some(
            "Демо-вход отключен. Используйте серверную аутентификацию или включите RUSTOK_DEMO_MODE.",
        ),
        _ => None,
    }
}
