use leptos::*;

mod app;
mod dashboard;
mod errors;
mod login;
mod users;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Locale {
    En,
    Ru,
}

impl Locale {
    pub fn from_code(code: &str) -> Self {
        match code.to_lowercase().as_str() {
            "en" => Locale::En,
            _ => Locale::Ru,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Ru => "ru",
        }
    }
}

#[derive(Clone)]
pub struct LocaleContext {
    pub locale: ReadSignal<Locale>,
    pub set_locale: WriteSignal<Locale>,
}

pub fn provide_locale_context() {
    let initial_locale = load_locale_from_storage().unwrap_or(Locale::Ru);
    let (locale, set_locale) = create_signal(initial_locale);

    create_effect(move |_| {
        if let Some(storage) = local_storage() {
            let _ = storage.set_item("rustok-admin-locale", locale.get().code());
        }
    });

    provide_context(LocaleContext { locale, set_locale });
}

pub fn use_locale() -> LocaleContext {
    use_context::<LocaleContext>().expect("LocaleContext not found")
}

pub fn translate(locale: Locale, key: &str) -> &'static str {
    match locale {
        Locale::En => translate_en(key),
        Locale::Ru => translate_ru(key),
    }
}

fn translate_en(key: &str) -> &'static str {
    app::translate_en(key)
        .or_else(|| login::translate_en(key))
        .or_else(|| dashboard::translate_en(key))
        .or_else(|| users::translate_en(key))
        .or_else(|| errors::translate_en(key))
        .unwrap_or(key)
}

fn translate_ru(key: &str) -> &'static str {
    app::translate_ru(key)
        .or_else(|| login::translate_ru(key))
        .or_else(|| dashboard::translate_ru(key))
        .or_else(|| users::translate_ru(key))
        .or_else(|| errors::translate_ru(key))
        .unwrap_or(key)
}

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
}

fn load_locale_from_storage() -> Option<Locale> {
    let storage = local_storage()?;
    let value = storage.get_item("rustok-admin-locale").ok().flatten()?;
    Some(Locale::from_code(&value))
}
