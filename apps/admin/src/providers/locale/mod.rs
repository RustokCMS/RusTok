use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;

use crate::i18n;
pub use crate::i18n::Locale;

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

pub fn provide_locale_context() -> LocaleContext {
    let initial_locale = load_locale_from_storage().unwrap_or(Locale::Ru);
    let (locale, set_locale) = signal(initial_locale);

    Effect::new(move |_| {
        let _ = LocalStorage::set("rustok-admin-locale", locale.get().code());
    });

    let context = LocaleContext { locale, set_locale };
    provide_context(context.clone());
    context
}

pub fn use_locale() -> LocaleContext {
    use_context::<LocaleContext>().expect("LocaleContext not found")
}

pub fn translate(key: &str) -> String {
    let locale = use_locale().locale.get_untracked();
    i18n::translate(locale, key)
}

fn load_locale_from_storage() -> Option<Locale> {
    let value: String = LocalStorage::get("rustok-admin-locale").ok()?;
    Some(Locale::from_code(&value))
}
