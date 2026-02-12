//! Security Audit Module
//!
//! This module provides comprehensive security auditing capabilities for the RusToK platform.
//! It covers authentication, authorization, input validation, data protection, and infrastructure security.
//!
//! ## Features
//!
//! - **Security Auditing**: Multi-layer security checks across all system components
//! - **Vulnerability Scanning**: Automated detection of common security issues
//! - **Compliance Reporting**: Generate security reports for compliance requirements
//! - **Real-time Monitoring**: Continuous security monitoring with configurable alerts
//!
//! ## Usage
//!
//! ```rust
//! use rustok_core::security::{SecurityAuditor, SecurityCheck, SecurityReport};
//!
//! // Run a full security audit
//! let auditor = SecurityAuditor::new();
//! let report = auditor.run_full_audit().await?;
//!
//! // Check for specific vulnerabilities
//! let check = SecurityCheck::authentication_audit();
//! let result = auditor.run_check(check).await?;
//! ```

pub mod audit;
pub mod checks;
pub mod models;
pub mod report;

pub use audit::SecurityAuditor;
pub use checks::SecurityCheck;
pub use models::{SecurityFinding, SecurityLevel, SecurityReport, SecurityStatus};
pub use report::SecurityReportGenerator;

use crate::permissions::{Action, Resource};
use crate::rbac::SecurityContext;
use crate::types::UserRole;
use std::collections::HashMap;

/// Security configuration for the audit system
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Minimum security level to report
    pub min_level: SecurityLevel,
    /// Whether to include informational findings
    pub include_info: bool,
    /// Custom security policies
    pub policies: SecurityPolicies,
    /// Enabled check categories
    pub enabled_categories: Vec<SecurityCategory>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            min_level: SecurityLevel::Low,
            include_info: true,
            policies: SecurityPolicies::default(),
            enabled_categories: SecurityCategory::all(),
        }
    }
}

/// Security policy configuration
#[derive(Debug, Clone)]
pub struct SecurityPolicies {
    /// Password policy
    pub password_policy: PasswordPolicy,
    /// Session policy
    pub session_policy: SessionPolicy,
    /// Rate limiting policy
    pub rate_limit_policy: RateLimitPolicy,
    /// Data retention policy
    pub data_retention_policy: DataRetentionPolicy,
}

impl Default for SecurityPolicies {
    fn default() -> Self {
        Self {
            password_policy: PasswordPolicy::default(),
            session_policy: SessionPolicy::default(),
            rate_limit_policy: RateLimitPolicy::default(),
            data_retention_policy: DataRetentionPolicy::default(),
        }
    }
}

/// Password policy configuration
#[derive(Debug, Clone)]
pub struct PasswordPolicy {
    /// Minimum password length
    pub min_length: usize,
    /// Require uppercase letters
    pub require_uppercase: bool,
    /// Require lowercase letters
    pub require_lowercase: bool,
    /// Require numbers
    pub require_numbers: bool,
    /// Require special characters
    pub require_special: bool,
    /// Maximum age in days (0 = no expiration)
    pub max_age_days: u32,
    /// Prevent reuse of last N passwords
    pub prevent_reuse_count: usize,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 12,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special: true,
            max_age_days: 90,
            prevent_reuse_count: 5,
        }
    }
}

/// Session policy configuration
#[derive(Debug, Clone)]
pub struct SessionPolicy {
    /// Session timeout in minutes
    pub timeout_minutes: u32,
    /// Absolute session timeout in minutes
    pub absolute_timeout_minutes: u32,
    /// Require re-authentication for sensitive operations
    pub require_reauth_for_sensitive: bool,
    /// Invalidate sessions on password change
    pub invalidate_on_password_change: bool,
    /// Maximum concurrent sessions per user
    pub max_concurrent_sessions: usize,
}

impl Default for SessionPolicy {
    fn default() -> Self {
        Self {
            timeout_minutes: 30,
            absolute_timeout_minutes: 480,
            require_reauth_for_sensitive: true,
            invalidate_on_password_change: true,
            max_concurrent_sessions: 5,
        }
    }
}

/// Rate limiting policy configuration
#[derive(Debug, Clone)]
pub struct RateLimitPolicy {
    /// Login attempts per minute
    pub login_attempts_per_minute: u32,
    /// API requests per minute for authenticated users
    pub api_requests_per_minute: u32,
    /// API requests per minute for anonymous users
    pub anonymous_requests_per_minute: u32,
    /// Burst allowance
    pub burst_allowance: u32,
}

impl Default for RateLimitPolicy {
    fn default() -> Self {
        Self {
            login_attempts_per_minute: 5,
            api_requests_per_minute: 100,
            anonymous_requests_per_minute: 30,
            burst_allowance: 10,
        }
    }
}

/// Data retention policy configuration
#[derive(Debug, Clone)]
pub struct DataRetentionPolicy {
    /// Audit log retention in days
    pub audit_log_retention_days: u32,
    /// User activity log retention in days
    pub activity_log_retention_days: u32,
    /// Failed login retention in days
    pub failed_login_retention_days: u32,
    /// Soft delete grace period in days
    pub soft_delete_grace_days: u32,
}

impl Default for DataRetentionPolicy {
    fn default() -> Self {
        Self {
            audit_log_retention_days: 365,
            activity_log_retention_days: 90,
            failed_login_retention_days: 30,
            soft_delete_grace_days: 30,
        }
    }
}

/// Security audit categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecurityCategory {
    Authentication,
    Authorization,
    InputValidation,
    DataProtection,
    EventSystem,
    Infrastructure,
    TenantSecurity,
}

impl SecurityCategory {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Authentication,
            Self::Authorization,
            Self::InputValidation,
            Self::DataProtection,
            Self::EventSystem,
            Self::Infrastructure,
            Self::TenantSecurity,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Authentication => "authentication",
            Self::Authorization => "authorization",
            Self::InputValidation => "input_validation",
            Self::DataProtection => "data_protection",
            Self::EventSystem => "event_system",
            Self::Infrastructure => "infrastructure",
            Self::TenantSecurity => "tenant_security",
        }
    }
}

/// Security context for audit operations
#[derive(Debug, Clone)]
pub struct AuditContext {
    /// Current security context
    pub security_ctx: SecurityContext,
    /// Tenant ID being audited
    pub tenant_id: Option<uuid::Uuid>,
    /// Audit scope
    pub scope: AuditScope,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl AuditContext {
    pub fn new(security_ctx: SecurityContext) -> Self {
        Self {
            security_ctx,
            tenant_id: None,
            scope: AuditScope::Global,
            metadata: HashMap::new(),
        }
    }

    pub fn for_tenant(mut self, tenant_id: uuid::Uuid) -> Self {
        self.tenant_id = Some(tenant_id);
        self.scope = AuditScope::Tenant;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if the current user has permission to run security audits
    pub fn can_run_audit(&self) -> bool {
        matches!(
            self.security_ctx.role,
            UserRole::SuperAdmin | UserRole::Admin
        )
    }

    /// Check if the current user can audit a specific tenant
    pub fn can_audit_tenant(&self, tenant_id: uuid::Uuid) -> bool {
        if !self.can_run_audit() {
            return false;
        }

        match self.security_ctx.role {
            UserRole::SuperAdmin => true,
            UserRole::Admin => self.tenant_id.map(|id| id == tenant_id).unwrap_or(true),
            _ => false,
        }
    }
}

/// Audit scope
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditScope {
    Global,
    Tenant,
    Module,
    Resource,
}

/// Security remediation recommendation
#[derive(Debug, Clone)]
pub struct Remediation {
    /// Unique identifier for the remediation
    pub id: String,
    /// Title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Steps to implement
    pub steps: Vec<String>,
    /// Priority level
    pub priority: SecurityLevel,
    /// Estimated effort (hours)
    pub estimated_effort_hours: u32,
    /// Resources for further reading
    pub resources: Vec<String>,
}

impl Remediation {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: String::new(),
            steps: Vec::new(),
            priority: SecurityLevel::Medium,
            estimated_effort_hours: 0,
            resources: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_step(mut self, step: impl Into<String>) -> Self {
        self.steps.push(step.into());
        self
    }

    pub fn with_priority(mut self, priority: SecurityLevel) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_effort(mut self, hours: u32) -> Self {
        self.estimated_effort_hours = hours;
        self
    }

    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resources.push(resource.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_config_default() {
        let config = SecurityConfig::default();
        assert_eq!(config.min_level, SecurityLevel::Low);
        assert!(config.include_info);
        assert_eq!(config.enabled_categories.len(), 7);
    }

    #[test]
    fn test_password_policy_default() {
        let policy = PasswordPolicy::default();
        assert_eq!(policy.min_length, 12);
        assert!(policy.require_uppercase);
        assert!(policy.require_lowercase);
        assert!(policy.require_numbers);
        assert!(policy.require_special);
        assert_eq!(policy.max_age_days, 90);
        assert_eq!(policy.prevent_reuse_count, 5);
    }

    #[test]
    fn test_audit_context_permissions() {
        let admin_ctx =
            AuditContext::new(SecurityContext::new(UserRole::Admin, Some(uuid::Uuid::new_v4())));
        assert!(admin_ctx.can_run_audit());

        let customer_ctx =
            AuditContext::new(SecurityContext::new(UserRole::Customer, Some(uuid::Uuid::new_v4())));
        assert!(!customer_ctx.can_run_audit());
    }

    #[test]
    fn test_remediation_builder() {
        let remediation = Remediation::new("AUTH-001", "Enable MFA")
            .with_description("Enable multi-factor authentication for all admin users")
            .with_step("Configure MFA provider")
            .with_step("Enforce MFA for admin role")
            .with_priority(SecurityLevel::High)
            .with_effort(4)
            .with_resource("https://docs.example.com/mfa");

        assert_eq!(remediation.id, "AUTH-001");
        assert_eq!(remediation.title, "Enable MFA");
        assert_eq!(remediation.steps.len(), 2);
        assert_eq!(remediation.estimated_effort_hours, 4);
    }
}
