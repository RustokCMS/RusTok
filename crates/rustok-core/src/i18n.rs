/// Internationalization support for RusToK
///
/// Provides localized error messages and validation feedback

use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Supported locales
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Locale {
    En,
    Ru,
    Es,
    De,
    Fr,
    Zh,
}

impl Locale {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().split('-').next()? {
            "en" => Some(Locale::En),
            "ru" => Some(Locale::Ru),
            "es" => Some(Locale::Es),
            "de" => Some(Locale::De),
            "fr" => Some(Locale::Fr),
            "zh" => Some(Locale::Zh),
            _ => None,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Ru => "ru",
            Locale::Es => "es",
            Locale::De => "de",
            Locale::Fr => "fr",
            Locale::Zh => "zh",
        }
    }
}

impl Default for Locale {
    fn default() -> Self {
        Locale::En
    }
}

/// Static translations map
static TRANSLATIONS: Lazy<HashMap<(Locale, &'static str), &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    
    // Validation errors - English
    map.insert((Locale::En, "invalid_kind"), "Invalid content type");
    map.insert((Locale::En, "invalid_format"), "Invalid body format");
    map.insert((Locale::En, "invalid_locale_length"), "Locale must be 2-10 characters");
    map.insert((Locale::En, "invalid_locale_format"), "Invalid locale format");
    map.insert((Locale::En, "position_must_be_non_negative"), "Position must be non-negative");
    map.insert((Locale::En, "position_too_large"), "Position is too large");
    map.insert((Locale::En, "depth_must_be_non_negative"), "Depth must be non-negative");
    map.insert((Locale::En, "depth_too_large"), "Depth is too large (max 100)");
    map.insert((Locale::En, "reply_count_must_be_non_negative"), "Reply count must be non-negative");
    map.insert((Locale::En, "slug_empty"), "Slug cannot be empty");
    map.insert((Locale::En, "slug_too_long"), "Slug is too long (max 255 characters)");
    map.insert((Locale::En, "slug_invalid_characters"), "Slug can only contain lowercase letters, numbers, and hyphens");
    map.insert((Locale::En, "slug_hyphen_boundary"), "Slug cannot start or end with a hyphen");
    
    // Validation errors - Russian
    map.insert((Locale::Ru, "invalid_kind"), "Неверный тип контента");
    map.insert((Locale::Ru, "invalid_format"), "Неверный формат");
    map.insert((Locale::Ru, "invalid_locale_length"), "Локаль должна быть 2-10 символов");
    map.insert((Locale::Ru, "invalid_locale_format"), "Неверный формат локали");
    map.insert((Locale::Ru, "position_must_be_non_negative"), "Позиция должна быть неотрицательной");
    map.insert((Locale::Ru, "position_too_large"), "Позиция слишком большая");
    map.insert((Locale::Ru, "depth_must_be_non_negative"), "Глубина должна быть неотрицательной");
    map.insert((Locale::Ru, "depth_too_large"), "Глубина слишком большая (макс. 100)");
    map.insert((Locale::Ru, "reply_count_must_be_non_negative"), "Количество ответов должно быть неотрицательным");
    map.insert((Locale::Ru, "slug_empty"), "Slug не может быть пустым");
    map.insert((Locale::Ru, "slug_too_long"), "Slug слишком длинный (макс. 255 символов)");
    map.insert((Locale::Ru, "slug_invalid_characters"), "Slug может содержать только строчные буквы, цифры и дефисы");
    map.insert((Locale::Ru, "slug_hyphen_boundary"), "Slug не может начинаться или заканчиваться дефисом");
    
    // Validation errors - Spanish
    map.insert((Locale::Es, "invalid_kind"), "Tipo de contenido inválido");
    map.insert((Locale::Es, "invalid_format"), "Formato de cuerpo inválido");
    map.insert((Locale::Es, "invalid_locale_length"), "La configuración regional debe tener 2-10 caracteres");
    map.insert((Locale::Es, "invalid_locale_format"), "Formato de configuración regional inválido");
    map.insert((Locale::Es, "position_must_be_non_negative"), "La posición debe ser no negativa");
    map.insert((Locale::Es, "position_too_large"), "La posición es demasiado grande");
    map.insert((Locale::Es, "depth_must_be_non_negative"), "La profundidad debe ser no negativa");
    map.insert((Locale::Es, "depth_too_large"), "La profundidad es demasiado grande (máx. 100)");
    map.insert((Locale::Es, "reply_count_must_be_non_negative"), "El recuento de respuestas debe ser no negativo");
    map.insert((Locale::Es, "slug_empty"), "El slug no puede estar vacío");
    map.insert((Locale::Es, "slug_too_long"), "El slug es demasiado largo (máx. 255 caracteres)");
    map.insert((Locale::Es, "slug_invalid_characters"), "El slug solo puede contener letras minúsculas, números y guiones");
    map.insert((Locale::Es, "slug_hyphen_boundary"), "El slug no puede comenzar o terminar con un guion");
    
    // Validation errors - German
    map.insert((Locale::De, "invalid_kind"), "Ungültiger Inhaltstyp");
    map.insert((Locale::De, "invalid_format"), "Ungültiges Body-Format");
    map.insert((Locale::De, "invalid_locale_length"), "Locale muss 2-10 Zeichen lang sein");
    map.insert((Locale::De, "invalid_locale_format"), "Ungültiges Locale-Format");
    map.insert((Locale::De, "position_must_be_non_negative"), "Position muss nicht-negativ sein");
    map.insert((Locale::De, "position_too_large"), "Position ist zu groß");
    map.insert((Locale::De, "depth_must_be_non_negative"), "Tiefe muss nicht-negativ sein");
    map.insert((Locale::De, "depth_too_large"), "Tiefe ist zu groß (max. 100)");
    map.insert((Locale::De, "reply_count_must_be_non_negative"), "Antwortanzahl muss nicht-negativ sein");
    map.insert((Locale::De, "slug_empty"), "Slug darf nicht leer sein");
    map.insert((Locale::De, "slug_too_long"), "Slug ist zu lang (max. 255 Zeichen)");
    map.insert((Locale::De, "slug_invalid_characters"), "Slug darf nur Kleinbuchstaben, Zahlen und Bindestriche enthalten");
    map.insert((Locale::De, "slug_hyphen_boundary"), "Slug darf nicht mit einem Bindestrich beginnen oder enden");
    
    // Validation errors - French
    map.insert((Locale::Fr, "invalid_kind"), "Type de contenu invalide");
    map.insert((Locale::Fr, "invalid_format"), "Format de corps invalide");
    map.insert((Locale::Fr, "invalid_locale_length"), "La locale doit avoir 2 à 10 caractères");
    map.insert((Locale::Fr, "invalid_locale_format"), "Format de locale invalide");
    map.insert((Locale::Fr, "position_must_be_non_negative"), "La position doit être non négative");
    map.insert((Locale::Fr, "position_too_large"), "La position est trop grande");
    map.insert((Locale::Fr, "depth_must_be_non_negative"), "La profondeur doit être non négative");
    map.insert((Locale::Fr, "depth_too_large"), "La profondeur est trop grande (max. 100)");
    map.insert((Locale::Fr, "reply_count_must_be_non_negative"), "Le nombre de réponses doit être non négatif");
    map.insert((Locale::Fr, "slug_empty"), "Le slug ne peut pas être vide");
    map.insert((Locale::Fr, "slug_too_long"), "Le slug est trop long (max. 255 caractères)");
    map.insert((Locale::Fr, "slug_invalid_characters"), "Le slug ne peut contenir que des lettres minuscules, des chiffres et des traits d'union");
    map.insert((Locale::Fr, "slug_hyphen_boundary"), "Le slug ne peut pas commencer ou se terminer par un trait d'union");
    
    // Validation errors - Chinese
    map.insert((Locale::Zh, "invalid_kind"), "无效的内容类型");
    map.insert((Locale::Zh, "invalid_format"), "无效的正文格式");
    map.insert((Locale::Zh, "invalid_locale_length"), "语言环境必须为2-10个字符");
    map.insert((Locale::Zh, "invalid_locale_format"), "无效的语言环境格式");
    map.insert((Locale::Zh, "position_must_be_non_negative"), "位置必须为非负数");
    map.insert((Locale::Zh, "position_too_large"), "位置太大");
    map.insert((Locale::Zh, "depth_must_be_non_negative"), "深度必须为非负数");
    map.insert((Locale::Zh, "depth_too_large"), "深度太大（最多100）");
    map.insert((Locale::Zh, "reply_count_must_be_non_negative"), "回复数必须为非负数");
    map.insert((Locale::Zh, "slug_empty"), "Slug不能为空");
    map.insert((Locale::Zh, "slug_too_long"), "Slug太长（最多255个字符）");
    map.insert((Locale::Zh, "slug_invalid_characters"), "Slug只能包含小写字母、数字和连字符");
    map.insert((Locale::Zh, "slug_hyphen_boundary"), "Slug不能以连字符开头或结尾");
    
    map
});

/// Get translation for a key in the specified locale  
/// Returns the translation if found, otherwise returns the key itself
pub fn translate(locale: Locale, key: &str) -> String {
    // Try to get translation from map
    // We need to iterate and compare because key is not 'static
    for ((loc, trans_key), trans_value) in TRANSLATIONS.iter() {
        if *loc == locale && *trans_key == key {
            return trans_value.to_string();
        }
    }
    
    // Fallback to English
    for ((loc, trans_key), trans_value) in TRANSLATIONS.iter() {
        if *loc == Locale::En && *trans_key == key {
            return trans_value.to_string();
        }
    }
    
    // Return key if no translation found
    key.to_string()
}

/// Extract locale from Accept-Language header
pub fn extract_locale_from_header(accept_language: Option<&str>) -> Locale {
    accept_language
        .and_then(|header| {
            // Parse Accept-Language header (e.g., "en-US,en;q=0.9,ru;q=0.8")
            header
                .split(',')
                .next()
                .and_then(|lang| lang.split(';').next())
                .and_then(|lang| Locale::from_str(lang.trim()))
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_from_str() {
        assert_eq!(Locale::from_str("en"), Some(Locale::En));
        assert_eq!(Locale::from_str("en-US"), Some(Locale::En));
        assert_eq!(Locale::from_str("ru"), Some(Locale::Ru));
        assert_eq!(Locale::from_str("ru-RU"), Some(Locale::Ru));
        assert_eq!(Locale::from_str("invalid"), None);
    }

    #[test]
    fn test_translate_english() {
        assert_eq!(translate(Locale::En, "invalid_kind"), "Invalid content type");
        assert_eq!(translate(Locale::En, "slug_empty"), "Slug cannot be empty");
    }

    #[test]
    fn test_translate_russian() {
        assert_eq!(translate(Locale::Ru, "invalid_kind"), "Неверный тип контента");
        assert_eq!(translate(Locale::Ru, "slug_empty"), "Slug не может быть пустым");
    }

    #[test]
    fn test_translate_fallback() {
        // Unknown key should return the key itself
        assert_eq!(translate(Locale::En, "unknown_key"), "unknown_key");
    }

    #[test]
    fn test_extract_locale_from_header() {
        assert_eq!(extract_locale_from_header(Some("en-US,en;q=0.9")), Locale::En);
        assert_eq!(extract_locale_from_header(Some("ru-RU,ru;q=0.9")), Locale::Ru);
        assert_eq!(extract_locale_from_header(Some("es")), Locale::Es);
        assert_eq!(extract_locale_from_header(None), Locale::En);
    }
}
