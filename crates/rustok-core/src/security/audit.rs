//! Security Audit Logging - OWASP Logging Failures Prevention
//!
//! Comprehensive security event logging for:
//! - Authentication events (login, logout, failed attempts)
//! - Authorization events (access denied, privilege escalation)
//! - Data access events (sensitive data access)
//! - System events (configuration changes, errors)
//! - Security violations (rate limiting, validation failures)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn};
use uuid::Uuid;

use super::{SecurityCategory, SecurityFinding, Severity};
use crate::security::SecurityConfig;

/// Security audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID
    pub id: Uuid,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Event type
    pub event_type: AuditEventType,
    /// Severity level
    pub severity: Severity,
    /// User ID (if authenticated)
    pub user_id: Option<Uuid>,
    /// Tenant ID
    pub tenant_id: Option<Uuid>,
    /// IP address
    pub ip_address: Option<IpAddr>,
    /// Request ID
    pub request_id: Option<String>,
    /// Event description
    pub description: String,
    /// Additional context
    pub context: HashMap<String, String>,
    /// Resource being accessed
    pub resource: Option<String>,
    /// Action performed
    pub action: Option<String>,
    /// Success or failure
    pub success: bool,
}

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    Authentication,
    Authorization,
    DataAccess,
    DataModification,
    ConfigurationChange,
    SecurityViolation,
    SystemError,
    RateLimitExceeded,
    ValidationFailure,
    SuspiciousActivity,
}

impl AuditEvent {
    /// Create new audit event
    pub fn new(event_type: AuditEventType) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            severity: Severity::Info,
            user_id: None,
            tenant_id: None,
            ip_address: None,
            request_id: None,
            description: String::new(),
            context: HashMap::new(),
            resource: None,
            action: None,
            success: true,
        }
    }

    /// Set severity
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Set user ID
    pub fn with_user(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set tenant ID
    pub fn with_tenant(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    /// Set IP address
    pub fn with_ip(mut self, ip: IpAddr) -> Self {
        self.ip_address = Some(ip);
        self
    }

    /// Set request ID
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set resource
    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    /// Set action
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Set success
    pub fn with_success(mut self, success: bool) -> Self {
        self.success = success;
        self
    }

    /// Add context
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Create authentication event
    pub fn authentication(user_id: Uuid, success: bool, ip: IpAddr) -> Self {
        let severity = if success { Severity::Info } else { Severity::Warning };
        Self::new(AuditEventType::Authentication)
            .with_user(user_id)
            .with_ip(ip)
            .with_success(success)
            .with_severity(severity)
            .with_description(if success {
                "User authenticated successfully"
            } else {
                "Authentication failed"
            })
    }

    /// Create authorization failure event
    pub fn authorization_denied(user_id: Uuid, resource: &str, action: &str, ip: IpAddr) -> Self {
        Self::new(AuditEventType::Authorization)
            .with_user(user_id)
            .with_resource(resource.to_string())
            .with_action(action.to_string())
            .with_ip(ip)
            .with_success(false)
            .with_severity(Severity::Warning)
            .with_description(format!("Access denied to {} for action {}", resource, action))
    }

    /// Create rate limit event
    pub fn rate_limit_exceeded(ip: IpAddr, identifier: &str) -> Self {
        Self::new(AuditEventType::RateLimitExceeded)
            .with_ip(ip)
            .with_severity(Severity::Warning)
            .with_description(format!("Rate limit exceeded for {}", identifier))
            .with_context("identifier", identifier)
    }

    /// Create validation failure event
    pub fn validation_failure(ip: IpAddr, reason: &str) -> Self {
        Self::new(AuditEventType::ValidationFailure)
            .with_ip(ip)
            .with_severity(Severity::Warning)
            .with_description(format!("Input validation failed: {}", reason))
    }

    /// Create suspicious activity event
    pub fn suspicious_activity(ip: IpAddr, description: &str) -> Self {
        Self::new(AuditEventType::SuspiciousActivity)
            .with_ip(ip)
            .with_severity(Severity::High)
            .with_description(description.to_string())
    }
}

/// Security audit logger
#[derive(Debug, Clone)]
pub struct AuditLogger {
    sender: mpsc::Sender<AuditEvent>,
    enabled: bool,
}

impl AuditLogger {
    /// Create new audit logger
    pub fn new(enabled: bool) -> (Self, mpsc::Receiver<AuditEvent>) {
        let (sender, receiver) = mpsc::channel(1000);
        (Self { sender, enabled }, receiver)
    }

    /// Log an audit event
    pub async fn log(&self, event: AuditEvent) {
        if !self.enabled {
            return;
        }

        // Send to channel (non-blocking)
        if let Err(_) = self.sender.try_send(event.clone()) {
            // Channel full, log directly
            self.write_to_log(&event);
        }
    }

    /// Log authentication event
    pub async fn log_auth(&self, user_id: Uuid, success: bool, ip: IpAddr) {
        self.log(AuditEvent::authentication(user_id, success, ip)).await;
    }

    /// Log authorization failure
    pub async fn log_authz_denied(&self, user_id: Uuid, resource: &str, action: &str, ip: IpAddr) {
        self.log(AuditEvent::authorization_denied(user_id, resource, action, ip))
            .await;
    }

    /// Log rate limit exceeded
    pub async fn log_rate_limit(&self, ip: IpAddr, identifier: &str) {
        self.log(AuditEvent::rate_limit_exceeded(ip, identifier)).await;
    }

    /// Log validation failure
    pub async fn log_validation_failure(&self, ip: IpAddr, reason: &str) {
        self.log(AuditEvent::validation_failure(ip, reason)).await;
    }

    /// Log suspicious activity
    pub async fn log_suspicious(&self, ip: IpAddr, description: &str) {
        self.log(AuditEvent::suspicious_activity(ip, description)).await;
    }

    fn write_to_log(&self, event: &AuditEvent) {
        let json = serde_json::to_string(event).unwrap_or_default();

        match event.severity {
            Severity::Critical | Severity::High => {
                warn!(target: "security_audit", "{}", json);
            }
            _ => {
                info!(target: "security_audit", "{}", json);
            }
        }
    }
}

/// Security audit processor
pub struct SecurityAudit {
    logger: AuditLogger,
    receiver: mpsc::Receiver<AuditEvent>,
}

impl SecurityAudit {
    /// Create new security audit
    pub fn new(enabled: bool) -> Self {
        let (logger, receiver) = AuditLogger::new(enabled);
        Self { logger, receiver }
    }

    /// Get logger instance
    pub fn logger(&self) -> &AuditLogger {
        &self.logger
    }

    /// Process audit events (run in background task)
    pub async fn run(mut self) {
        while let Some(event) = self.receiver.recv().await {
            // Write to persistent storage or SIEM
            let json = serde_json::to_string(&event).unwrap_or_default();

            match event.severity {
                Severity::Critical | Severity::High => {
                    warn!(target: "security_audit", "{}", json);
                }
                _ => {
                    info!(target: "security_audit", "{}", json);
                }
            }

            // TODO: Send to external SIEM if configured
        }
    }

    /// Run processor for a single event (for testing)
    pub async fn process_one(&mut self) -> Option<AuditEvent> {
        self.receiver.recv().await
    }
}

/// Audit logging configuration check
pub async fn audit_logging(config: &SecurityConfig) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();

    if !config.audit_logging {
        findings.push(SecurityFinding {
            category: SecurityCategory::LoggingFailures,
            severity: Severity::High,
            description: "Security audit logging is disabled".to_string(),
            remediation: "Enable audit_logging for security event tracking".to_string(),
        });
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_audit_event_builder() {
        let event = AuditEvent::new(AuditEventType::Authentication)
            .with_user(Uuid::new_v4())
            .with_description("Test event")
            .with_success(true)
            .with_severity(Severity::Info);

        assert!(event.success);
        assert_eq!(event.severity, Severity::Info);
    }

    #[test]
    fn test_authentication_event() {
        let user_id = Uuid::new_v4();
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        let event = AuditEvent::authentication(user_id, true, ip);

        assert_eq!(event.event_type, AuditEventType::Authentication);
        assert!(event.success);
        assert_eq!(event.user_id, Some(user_id));
    }

    #[test]
    fn test_authorization_denied_event() {
        let user_id = Uuid::new_v4();
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        let event = AuditEvent::authorization_denied(user_id, "/admin", "DELETE", ip);

        assert_eq!(event.event_type, AuditEventType::Authorization);
        assert!(!event.success);
        assert_eq!(event.severity, Severity::Warning);
    }

    #[tokio::test]
    async fn test_audit_logger() {
        let (logger, mut receiver) = AuditLogger::new(true);

        let event = AuditEvent::new(AuditEventType::SystemError)
            .with_description("Test error");

        logger.log(event.clone()).await;

        let received = receiver.recv().await;
        assert!(received.is_some());
        assert_eq!(received.unwrap().description, "Test error");
    }
}
