use async_trait::async_trait;
use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use crate::config::EmailConfig;
use crate::error::{EmailError, Result};

/// Email to send for password reset.
#[derive(Debug, Clone)]
pub struct PasswordResetEmail {
    pub to: String,
    pub reset_url: String,
}

/// Trait for sending password reset emails. Allows test doubles.
#[async_trait]
pub trait PasswordResetEmailSender: Send + Sync {
    async fn send_password_reset(&self, email: PasswordResetEmail) -> Result<()>;
}

/// Top-level email service — disabled or SMTP-backed.
#[derive(Clone)]
pub enum EmailService {
    Disabled,
    Smtp(Box<SmtpEmailSender>),
}

impl EmailService {
    /// Build from config. Returns `Disabled` if `enabled == false`.
    pub fn from_config(config: &EmailConfig) -> Result<Self> {
        if !config.enabled {
            return Ok(Self::Disabled);
        }
        Ok(Self::Smtp(Box::new(SmtpEmailSender::try_new(config)?)))
    }

    /// Build the password-reset URL from config + token.
    pub fn password_reset_url(config: &EmailConfig, token: &str) -> String {
        format!("{}?token={}", config.reset_base_url, token)
    }
}

#[async_trait]
impl PasswordResetEmailSender for EmailService {
    async fn send_password_reset(&self, email: PasswordResetEmail) -> Result<()> {
        match self {
            Self::Disabled => {
                tracing::info!(
                    recipient = %email.to,
                    "Password reset email provider disabled; skipping outbound send"
                );
                Ok(())
            }
            Self::Smtp(sender) => sender.send_password_reset(email).await,
        }
    }
}

/// SMTP-backed email sender.
#[derive(Clone)]
pub struct SmtpEmailSender {
    from: Mailbox,
    transport: AsyncSmtpTransport<Tokio1Executor>,
}

impl SmtpEmailSender {
    fn try_new(config: &EmailConfig) -> Result<Self> {
        let from = config
            .from
            .parse::<Mailbox>()
            .map_err(|e| EmailError::InvalidAddress(format!("Invalid from address: {e}")))?;

        let mut transport_builder = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp.host)
            .map_err(|e| EmailError::SmtpConfig(format!("Invalid SMTP relay: {e}")))?
            .port(config.smtp.port);

        if !config.smtp.username.trim().is_empty() {
            let creds =
                Credentials::new(config.smtp.username.clone(), config.smtp.password.clone());
            transport_builder = transport_builder.credentials(creds);
        }

        Ok(Self {
            from,
            transport: transport_builder.build(),
        })
    }
}

#[async_trait]
impl PasswordResetEmailSender for SmtpEmailSender {
    async fn send_password_reset(&self, email: PasswordResetEmail) -> Result<()> {
        let recipient = email
            .to
            .parse::<Mailbox>()
            .map_err(|e| EmailError::InvalidAddress(format!("Invalid recipient: {e}")))?;

        let message = Message::builder()
            .from(self.from.clone())
            .to(recipient)
            .subject("RusToK password reset")
            .header(ContentType::TEXT_HTML)
            .body(format!(
                "<p>You requested a password reset.</p><p><a href=\"{}\">Reset password</a></p>",
                email.reset_url
            ))
            .map_err(|e| EmailError::Build(e.to_string()))?;

        self.transport
            .send(message)
            .await
            .map_err(|e| EmailError::Send(e.to_string()))?;

        Ok(())
    }
}
