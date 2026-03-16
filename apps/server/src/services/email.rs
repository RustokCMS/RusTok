// Re-export from rustok-email for backward compatibility.
pub use rustok_email::{EmailService, PasswordResetEmail, PasswordResetEmailSender};

use crate::common::settings::RustokSettings;
use loco_rs::app::AppContext;

use crate::error::{Error, Result};

/// Loco bridge: convert `EmailError` → `loco_rs::Error`.
pub fn email_err(err: rustok_email::EmailError) -> Error {
    Error::Message(err.to_string())
}

/// Build `EmailService` from Loco's `AppContext`.
pub fn email_service_from_ctx(ctx: &AppContext) -> Result<EmailService> {
    let settings = RustokSettings::from_settings(&ctx.config.settings)
        .map_err(|e| Error::Message(e.to_string()))?;

    let config = rustok_email::EmailConfig {
        enabled: settings.email.enabled,
        smtp: rustok_email::SmtpConfig {
            host: settings.email.smtp.host.clone(),
            port: settings.email.smtp.port,
            username: settings.email.smtp.username.clone(),
            password: settings.email.smtp.password.clone(),
        },
        from: settings.email.from.clone(),
        reset_base_url: settings.email.reset_base_url.clone(),
    };

    EmailService::from_config(&config).map_err(email_err)
}

/// Build password reset URL from settings + token.
pub fn password_reset_url(ctx: &AppContext, token: &str) -> Result<String> {
    let settings = RustokSettings::from_settings(&ctx.config.settings)
        .map_err(|e| Error::Message(e.to_string()))?;

    let config = rustok_email::EmailConfig {
        reset_base_url: settings.email.reset_base_url.clone(),
        ..Default::default()
    };

    Ok(rustok_email::EmailService::password_reset_url(&config, token))
}
