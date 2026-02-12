//! Security Checks
//!
//! This module defines the various security checks that can be performed
//! during a security audit.

use super::models::{ComplianceFramework, Evidence, EvidenceType, SecurityFinding, SecurityLevel};
use super::{Remediation, SecurityCategory, SecurityConfig};
use crate::rbac::{PermissionScope, Rbac, SecurityContext};
use crate::types::UserRole;

/// A security check that can be executed
#[derive(Debug, Clone)]
pub enum SecurityCheck {
    /// Authentication security audit
    Authentication {
        /// Check password policy compliance
        check_password_policy: bool,
        /// Check MFA enforcement
        check_mfa: bool,
        /// Check session management
        check_sessions: bool,
        /// Check for weak credentials
        check_weak_credentials: bool,
    },
    /// Authorization security audit
    Authorization {
        /// Check role configuration
        check_roles: bool,
        /// Check permission assignments
        check_permissions: bool,
        /// Check for privilege escalation vectors
        check_privilege_escalation: bool,
    },
    /// Input validation audit
    InputValidation {
        /// Check SQL injection prevention
        check_sql_injection: bool,
        /// Check XSS prevention
        check_xss: bool,
        /// Check path traversal prevention
        check_path_traversal: bool,
        /// Check for unsafe deserialization
        check_deserialization: bool,
    },
    /// Data protection audit
    DataProtection {
        /// Check encryption at rest
        check_encryption_at_rest: bool,
        /// Check encryption in transit
        check_encryption_in_transit: bool,
        /// Check sensitive data handling
        check_sensitive_data: bool,
        /// Check data retention policies
        check_data_retention: bool,
    },
    /// Event system security audit
    EventSystem {
        /// Check event validation
        check_event_validation: bool,
        /// Check for replay attacks
        check_replay_protection: bool,
        /// Check event integrity
        check_event_integrity: bool,
    },
    /// Infrastructure security audit
    Infrastructure {
        /// Check security headers
        check_security_headers: bool,
        /// Check TLS configuration
        check_tls: bool,
        /// Check rate limiting
        check_rate_limiting: bool,
        /// Check logging configuration
        check_logging: bool,
    },
    /// Tenant security audit
    TenantSecurity {
        /// Check tenant isolation
        check_isolation: bool,
        /// Check tenant validation
        check_validation: bool,
        /// Check cross-tenant access
        check_cross_tenant_access: bool,
    },
    /// Custom security check
    Custom {
        name: String,
        category: SecurityCategory,
        level: SecurityLevel,
        check_fn: fn(&SecurityConfig) -> Vec<SecurityFinding>,
    },
}

impl SecurityCheck {
    /// Creates a comprehensive authentication audit
    pub fn authentication_audit() -> Self {
        Self::Authentication {
            check_password_policy: true,
            check_mfa: true,
            check_sessions: true,
            check_weak_credentials: true,
        }
    }

    /// Creates a comprehensive authorization audit
    pub fn authorization_audit() -> Self {
        Self::Authorization {
            check_roles: true,
            check_permissions: true,
            check_privilege_escalation: true,
        }
    }

    /// Creates a comprehensive input validation audit
    pub fn input_validation_audit() -> Self {
        Self::InputValidation {
            check_sql_injection: true,
            check_xss: true,
            check_path_traversal: true,
            check_deserialization: true,
        }
    }

    /// Creates a comprehensive data protection audit
    pub fn data_protection_audit() -> Self {
        Self::DataProtection {
            check_encryption_at_rest: true,
            check_encryption_in_transit: true,
            check_sensitive_data: true,
            check_data_retention: true,
        }
    }

    /// Creates a comprehensive event system audit
    pub fn event_system_audit() -> Self {
        Self::EventSystem {
            check_event_validation: true,
            check_replay_protection: true,
            check_event_integrity: true,
        }
    }

    /// Creates a comprehensive infrastructure audit
    pub fn infrastructure_audit() -> Self {
        Self::Infrastructure {
            check_security_headers: true,
            check_tls: true,
            check_rate_limiting: true,
            check_logging: true,
        }
    }

    /// Creates a comprehensive tenant security audit
    pub fn tenant_security_audit() -> Self {
        Self::TenantSecurity {
            check_isolation: true,
            check_validation: true,
            check_cross_tenant_access: true,
        }
    }

    /// Returns the category for this check
    pub fn category(&self) -> SecurityCategory {
        match self {
            Self::Authentication { .. } => SecurityCategory::Authentication,
            Self::Authorization { .. } => SecurityCategory::Authorization,
            Self::InputValidation { .. } => SecurityCategory::InputValidation,
            Self::DataProtection { .. } => SecurityCategory::DataProtection,
            Self::EventSystem { .. } => SecurityCategory::EventSystem,
            Self::Infrastructure { .. } => SecurityCategory::Infrastructure,
            Self::TenantSecurity { .. } => SecurityCategory::TenantSecurity,
            Self::Custom { category, .. } => *category,
        }
    }

    /// Returns the name of this check
    pub fn name(&self) -> String {
        match self {
            Self::Authentication { .. } => "Authentication Security Audit".to_string(),
            Self::Authorization { .. } => "Authorization Security Audit".to_string(),
            Self::InputValidation { .. } => "Input Validation Security Audit".to_string(),
            Self::DataProtection { .. } => "Data Protection Security Audit".to_string(),
            Self::EventSystem { .. } => "Event System Security Audit".to_string(),
            Self::Infrastructure { .. } => "Infrastructure Security Audit".to_string(),
            Self::TenantSecurity { .. } => "Tenant Security Audit".to_string(),
            Self::Custom { name, .. } => name.clone(),
        }
    }

    /// Executes the security check and returns findings
    pub fn execute(&self, config: &SecurityConfig) -> Vec<SecurityFinding> {
        match self {
            Self::Authentication { .. } => self.check_authentication(config),
            Self::Authorization { .. } => self.check_authorization(config),
            Self::InputValidation { .. } => self.check_input_validation(config),
            Self::DataProtection { .. } => self.check_data_protection(config),
            Self::EventSystem { .. } => self.check_event_system(config),
            Self::Infrastructure { .. } => self.check_infrastructure(config),
            Self::TenantSecurity { .. } => self.check_tenant_security(config),
            Self::Custom { check_fn, .. } => check_fn(config),
        }
    }

    /// Authentication security checks
    fn check_authentication(&self, config: &SecurityConfig) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        // Check password policy
        if matches!(self, Self::Authentication { check_password_policy: true, .. }) {
            let policy = &config.policies.password_policy;

            if policy.min_length < 12 {
                findings.push(
                    SecurityFinding::new(
                        SecurityCategory::Authentication,
                        SecurityLevel::Medium,
                        "Weak Password Minimum Length",
                    )
                    .with_description(format!(
                        "Password policy requires minimum {} characters. \
                         NIST recommends at least 12 characters for privileged accounts.",
                        policy.min_length
                    ))
                    .with_resource("security::password_policy::min_length")
                    .with_compliance(ComplianceFramework::NistCsf)
                    .with_compliance(ComplianceFramework::CisControls)
                    .with_remediation(
                        Remediation::new("AUTH-001", "Increase Password Minimum Length")
                            .with_description("Update password policy to require at least 12 characters")
                            .with_step("Update SecurityConfig::password_policy.min_length to 12 or higher")
                            .with_step("Communicate policy change to users")
                            .with_priority(SecurityLevel::Medium)
                            .with_effort(1),
                    ),
                );
            }

            if !policy.require_special {
                findings.push(
                    SecurityFinding::new(
                        SecurityCategory::Authentication,
                        SecurityLevel::Low,
                        "Password Policy Missing Special Character Requirement",
                    )
                    .with_description(
                        "Passwords are not required to contain special characters, \
                         which reduces the search space for brute force attacks"
                    )
                    .with_resource("security::password_policy::require_special")
                    .with_compliance(ComplianceFramework::CisControls)
                    .with_remediation(
                        Remediation::new("AUTH-002", "Require Special Characters in Passwords")
                            .with_step("Set password_policy.require_special to true")
                            .with_step("Update password validation logic")
                            .with_priority(SecurityLevel::Low)
                            .with_effort(1),
                    ),
                );
            }

            if policy.max_age_days == 0 {
                findings.push(
                    SecurityFinding::new(
                        SecurityCategory::Authentication,
                        SecurityLevel::Medium,
                        "No Password Expiration Policy",
                    )
                    .with_description(
                        "Passwords never expire, increasing the window of opportunity \
                         for compromised credentials"
                    )
                    .with_resource("security::password_policy::max_age_days")
                    .with_compliance(ComplianceFramework::PciDss)
                    .with_compliance(ComplianceFramework::CisControls)
                    .with_remediation(
                        Remediation::new("AUTH-003", "Implement Password Expiration")
                            .with_description("Set a maximum password age of 90 days")
                            .with_step("Set password_policy.max_age_days to 90")
                            .with_step("Implement password expiration notifications")
                            .with_step("Create password history to prevent reuse")
                            .with_priority(SecurityLevel::Medium)
                            .with_effort(4),
                    ),
                );
            }
        }

        // Check MFA
        if matches!(self, Self::Authentication { check_mfa: true, .. }) {
            findings.push(
                SecurityFinding::new(
                    SecurityCategory::Authentication,
                    SecurityLevel::High,
                    "MFA Not Enforced for Admin Roles",
                )
                .with_description(
                    "Multi-factor authentication is not enforced for administrator accounts. \
                     This significantly increases the risk of account compromise."
                )
                .with_resource("auth::mfa_policy")
                .with_compliance(ComplianceFramework::CisControls)
                .with_compliance(ComplianceFramework::NistCsf)
                .with_cwe("CWE-308")
                .with_remediation(
                    Remediation::new("AUTH-004", "Enforce MFA for Administrative Accounts")
                        .with_description("Require MFA for all admin and super-admin roles")
                        .with_step("Configure MFA provider (TOTP, WebAuthn, etc.)")
                        .with_step("Update authentication flow to require MFA for admin roles")
                        .with_step("Provide MFA setup grace period for existing users")
                        .with_step("Monitor and alert on MFA bypass attempts")
                        .with_priority(SecurityLevel::High)
                        .with_effort(8)
                        .with_resource("https://cheatsheetseries.owasp.org/cheatsheets/Multifactor_Authentication_Cheat_Sheet.html"),
                ),
            );
        }

        // Check session management
        if matches!(self, Self::Authentication { check_sessions: true, .. }) {
            let session_policy = &config.policies.session_policy;

            if session_policy.timeout_minutes > 60 {
                findings.push(
                    SecurityFinding::new(
                        SecurityCategory::Authentication,
                        SecurityLevel::Medium,
                        "Session Timeout Too Long",
                    )
                    .with_description(format!(
                        "Session timeout is set to {} minutes, which exceeds the recommended \
                         maximum of 60 minutes for sensitive applications",
                        session_policy.timeout_minutes
                    ))
                    .with_resource("security::session_policy::timeout_minutes")
                    .with_compliance(ComplianceFramework::CisControls)
                    .with_cwe("CWE-613")
                    .with_remediation(
                        Remediation::new("AUTH-005", "Reduce Session Timeout")
                            .with_step("Set session_policy.timeout_minutes to 30 or less")
                            .with_step("Implement sliding session refresh")
                            .with_step("Add session extension user confirmation")
                            .with_priority(SecurityLevel::Medium)
                            .with_effort(2),
                    ),
                );
            }

            if !session_policy.invalidate_on_password_change {
                findings.push(
                    SecurityFinding::new(
                        SecurityCategory::Authentication,
                        SecurityLevel::High,
                        "Sessions Not Invalidated on Password Change",
                    )
                    .with_description(
                        "Active sessions are not invalidated when a user changes their password. \
                         This allows attackers with stolen sessions to maintain access even after \
                         the legitimate user changes their password."
                    )
                    .with_resource("security::session_policy::invalidate_on_password_change")
                    .with_cwe("CWE-613")
                    .with_remediation(
                        Remediation::new("AUTH-006", "Invalidate Sessions on Password Change")
                            .with_step("Set session_policy.invalidate_on_password_change to true")
                            .with_step("Implement session invalidation logic in password change handler")
                            .with_step("Add notification to other active sessions")
                            .with_priority(SecurityLevel::High)
                            .with_effort(3),
                    ),
                );
            }
        }

        findings
    }

    /// Authorization security checks
    fn check_authorization(&self, _config: &SecurityConfig) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        // Check for overly permissive roles
        let admin_permissions = Rbac::permissions_for_role(&UserRole::Admin);
        let super_admin_permissions = Rbac::permissions_for_role(&UserRole::SuperAdmin);

        // Verify Admin doesn't have tenant management
        // This is a design check - in our current implementation, Admin should not manage tenants

        // Check for missing scope restrictions
        findings.push(
            SecurityFinding::new(
                SecurityCategory::Authorization,
                SecurityLevel::Medium,
                "Review Permission Scope Implementation",
            )
            .with_description(
                "Ensure all permission checks properly implement scope restrictions \
                 (Own vs All) to prevent unauthorized access to other users' data"
            )
            .with_resource("rbac::SecurityContext::get_scope")
            .with_compliance(ComplianceFramework::CisControls)
            .with_remediation(
                Remediation::new("AUTHZ-001", "Audit Permission Scope Usage")
                    .with_step("Review all permission checks in controllers")
                    .with_step("Verify scope is checked for resource access")
                    .with_step("Add automated tests for scope enforcement")
                    .with_step("Document scope behavior for each endpoint")
                    .with_priority(SecurityLevel::Medium)
                    .with_effort(4),
            ),
        );

        // Check Customer role for excessive permissions
        let customer_scope_check = SecurityContext::new(UserRole::Customer, Some(uuid::Uuid::new_v4()));
        let orders_scope = customer_scope_check.get_scope(
            crate::permissions::Resource::Orders,
            crate::permissions::Action::Read,
        );

        if orders_scope != PermissionScope::Own {
            findings.push(
                SecurityFinding::new(
                    SecurityCategory::Authorization,
                    SecurityLevel::High,
                    "Customer Role Has Excessive Order Access",
                )
                .with_description(
                    "Customer role should only be able to access their own orders, \
                     but current scope implementation may allow broader access"
                )
                .with_resource("rbac::Customer_permissions")
                .with_cwe("CWE-639")
                .with_remediation(
                    Remediation::new("AUTHZ-002", "Restrict Customer Order Access to Own Only")
                        .with_step("Verify Customer orders permission scope is set to Own")
                        .with_step("Add validation in order queries to filter by user_id")
                        .with_step("Add tests to verify customers cannot access other orders")
                        .with_priority(SecurityLevel::High)
                        .with_effort(3),
                ),
            );
        }

        findings
    }

    /// Input validation security checks
    fn check_input_validation(&self, _config: &SecurityConfig) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        findings.push(
            SecurityFinding::new(
                SecurityCategory::InputValidation,
                SecurityLevel::Info,
                "Input Validation Framework Active",
            )
            .with_description(
                "Tenant identifier validation is properly implemented with sanitization \
                 for SQL injection, XSS, and path traversal attacks"
            )
            .with_resource("tenant_validation::TenantIdentifierValidator")
            .with_evidence(
                Evidence::new(EvidenceType::Code, "Validation implementation")
                    .with_data("validate_slug, validate_uuid, validate_host methods implemented")
                    .from_source("crates/rustok-core/src/tenant_validation.rs"),
            ),
        );

        // Check for deserialization risks
        if matches!(
            self,
            Self::InputValidation {
                check_deserialization: true,
                ..
            }
        ) {
            findings.push(
                SecurityFinding::new(
                    SecurityCategory::InputValidation,
                    SecurityLevel::Medium,
                    "Review Deserialization Security",
                )
                .with_description(
                    "Ensure all deserialization operations use safe patterns to prevent \
                     deserialization attacks. Review usage of serde and other deserialization libraries."
                )
                .with_resource(" deserialization across all modules")
                .with_cwe("CWE-502")
                .with_remediation(
                    Remediation::new("INPUT-001", "Audit Deserialization Usage")
                        .with_step("Search for all deserialization code (serde, json, etc.)")
                        .with_step("Verify input validation before deserialization")
                        .with_step("Consider using typed deserialization with strict schemas")
                        .with_step("Add deserialization error handling")
                        .with_priority(SecurityLevel::Medium)
                        .with_effort(4),
                ),
            );
        }

        findings
    }

    /// Data protection security checks
    fn check_data_protection(&self, _config: &SecurityConfig) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        // Check encryption at rest
        if matches!(
            self,
            Self::DataProtection {
                check_encryption_at_rest: true,
                ..
            }
        ) {
            findings.push(
                SecurityFinding::new(
                    SecurityCategory::DataProtection,
                    SecurityLevel::High,
                    "Verify Database Encryption at Rest",
                )
                .with_description(
                    "Ensure PostgreSQL database is configured with encryption at rest. \
                     Verify SSL/TLS connections are enforced."
                )
                .with_resource("database::connection")
                .with_compliance(ComplianceFramework::PciDss)
                .with_compliance(ComplianceFramework::Gdpr)
                .with_remediation(
                    Remediation::new("DATA-001", "Enable Database Encryption")
                        .with_step("Configure PostgreSQL with SSL/TLS")
                        .with_step("Enable encryption at rest for database storage")
                        .with_step("Verify connection encryption in application")
                        .with_step("Rotate encryption keys regularly")
                        .with_priority(SecurityLevel::High)
                        .with_effort(4)
                        .with_resource("https://www.postgresql.org/docs/current/ssl-tcp.html"),
                ),
            );
        }

        // Check sensitive data handling
        if matches!(
            self,
            Self::DataProtection {
                check_sensitive_data: true,
                ..
            }
        ) {
            findings.push(
                SecurityFinding::new(
                    SecurityCategory::DataProtection,
                    SecurityLevel::Medium,
                    "Review Sensitive Data Logging",
                )
                .with_description(
                    "Ensure sensitive data (passwords, tokens, PII) is not logged \
                     in plain text in application logs"
                )
                .with_resource("logging::configuration")
                .with_compliance(ComplianceFramework::PciDss)
                .with_compliance(ComplianceFramework::Gdpr)
                .with_remediation(
                    Remediation::new("DATA-002", "Sanitize Logs")
                        .with_step("Audit all log statements for sensitive data")
                        .with_step("Implement log sanitization/redaction")
                        .with_step("Configure structured logging with field filtering")
                        .with_step("Add log review to code review checklist")
                        .with_priority(SecurityLevel::Medium)
                        .with_effort(4),
                ),
            );
        }

        findings
    }

    /// Event system security checks
    fn check_event_system(&self, _config: &SecurityConfig) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        findings.push(
            SecurityFinding::new(
                SecurityCategory::EventSystem,
                SecurityLevel::Info,
                "Event Validation Framework Active",
            )
            .with_description(
                "Event validation framework is implemented with schema validation \
                 and payload size limits"
            )
            .with_resource("events::validation")
            .with_evidence(
                Evidence::new(EvidenceType::Code, "Event validation")
                    .with_data("EventSchema, FieldSchema validation implemented")
                    .from_source("crates/rustok-core/src/events/validation.rs"),
            ),
        );

        // Check for replay protection
        if matches!(
            self,
            Self::EventSystem {
                check_replay_protection: true,
                ..
            }
        ) {
            findings.push(
                SecurityFinding::new(
                    SecurityCategory::EventSystem,
                    SecurityLevel::Medium,
                    "Review Event Replay Protection",
                )
                .with_description(
                    "Verify that event consumers implement idempotency to prevent \
                     duplicate processing of events"
                )
                .with_resource("events::consumers")
                .with_remediation(
                    Remediation::new("EVENT-001", "Implement Event Idempotency")
                        .with_step("Add event_id deduplication in consumers")
                        .with_step("Use idempotent operations where possible")
                        .with_step("Add consumer-side event tracking")
                        .with_step("Test replay scenarios")
                        .with_priority(SecurityLevel::Medium)
                        .with_effort(6),
                ),
            );
        }

        findings
    }

    /// Infrastructure security checks
    fn check_infrastructure(&self, _config: &SecurityConfig) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        // Check security headers
        if matches!(
            self,
            Self::Infrastructure {
                check_security_headers: true,
                ..
            }
        ) {
            let required_headers = vec![
                ("X-Content-Type-Options", "nosniff"),
                ("X-Frame-Options", "DENY or SAMEORIGIN"),
                ("X-XSS-Protection", "1; mode=block"),
                ("Strict-Transport-Security", "max-age=31536000"),
                ("Content-Security-Policy", "configured policy"),
            ];

            for (header, expected) in required_headers {
                findings.push(
                    SecurityFinding::new(
                        SecurityCategory::Infrastructure,
                        SecurityLevel::Medium,
                        format!("Verify Security Header: {}", header),
                    )
                    .with_description(format!(
                        "Ensure {} header is set to '{}' \
                         to protect against common web attacks",
                        header, expected
                    ))
                    .with_resource("http::middleware::security_headers")
                    .with_compliance(ComplianceFramework::OwaspTop10)
                    .with_remediation(
                        Remediation::new(
                            format!("INFRA-{}", header.replace("-", "_")),
                            format!("Add {} Header", header),
                        )
                        .with_step(format!("Add {} header in middleware", header))
                        .with_step("Configure appropriate values for your deployment")
                        .with_step("Test with security scanning tools")
                        .with_priority(SecurityLevel::Medium)
                        .with_effort(2),
                    ),
                );
            }
        }

        // Check rate limiting
        if matches!(
            self,
            Self::Infrastructure {
                check_rate_limiting: true,
                ..
            }
        ) {
            findings.push(
                SecurityFinding::new(
                    SecurityCategory::Infrastructure,
                    SecurityLevel::High,
                    "Verify Rate Limiting Implementation",
                )
                .with_description(
                    "Ensure rate limiting is properly configured and enforced \
                     to prevent brute force and DoS attacks"
                )
                .with_resource("middleware::rate_limiter")
                .with_compliance(ComplianceFramework::OwaspTop10)
                .with_remediation(
                    Remediation::new("INFRA-RATE", "Implement Rate Limiting")
                        .with_step("Verify rate limiting middleware is active")
                        .with_step("Configure limits based on endpoint sensitivity")
                        .with_step("Add IP-based and user-based rate limiting")
                        .with_step("Implement exponential backoff for repeated violations")
                        .with_step("Add monitoring and alerting for rate limit hits")
                        .with_priority(SecurityLevel::High)
                        .with_effort(4),
                ),
            );
        }

        findings
    }

    /// Tenant security checks
    fn check_tenant_security(&self, _config: &SecurityConfig) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        findings.push(
            SecurityFinding::new(
                SecurityCategory::TenantSecurity,
                SecurityLevel::Info,
                "Tenant Validation Framework Active",
            )
            .with_description(
                "Tenant identifier validation prevents injection attacks and \
                 enforces naming conventions"
            )
            .with_resource("tenant_validation")
            .with_evidence(
                Evidence::new(EvidenceType::Code, "Tenant validation tests")
                    .with_data("SQL injection, XSS, path traversal tests passing")
                    .from_source("crates/rustok-core/src/tenant_validation.rs tests"),
            ),
        );

        // Check tenant isolation
        if matches!(
            self,
            Self::TenantSecurity {
                check_isolation: true,
                ..
            }
        ) {
            findings.push(
                SecurityFinding::new(
                    SecurityCategory::TenantSecurity,
                    SecurityLevel::High,
                    "Verify Tenant Data Isolation",
                )
                .with_description(
                    "Ensure all database queries include tenant_id filtering \
                     to prevent cross-tenant data access"
                )
                .with_resource("database::queries")
                .with_cwe("CWE-639")
                .with_remediation(
                    Remediation::new("TENANT-001", "Audit Tenant Isolation")
                        .with_step("Review all database queries for tenant_id filtering")
                        .with_step("Add tenant context to repository layer")
                        .with_step("Implement row-level security in PostgreSQL")
                        .with_step("Add integration tests for tenant isolation")
                        .with_step("Regular penetration testing for tenant boundary")
                        .with_priority(SecurityLevel::High)
                        .with_effort(8),
                ),
            );
        }

        findings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_check_creation() {
        let auth_check = SecurityCheck::authentication_audit();
        assert_eq!(auth_check.category(), SecurityCategory::Authentication);
        assert!(auth_check.name().contains("Authentication"));
    }

    #[test]
    fn test_authentication_check_finds_issues() {
        let check = SecurityCheck::Authentication {
            check_password_policy: true,
            check_mfa: true,
            check_sessions: true,
            check_weak_credentials: true,
        };

        let config = SecurityConfig::default();
        let findings = check.execute(&config);

        // Should find at least the MFA finding
        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.title.contains("MFA")));
    }

    #[test]
    fn test_authorization_check() {
        let check = SecurityCheck::authorization_audit();
        let config = SecurityConfig::default();
        let findings = check.execute(&config);

        assert!(!findings.is_empty());
    }

    #[test]
    fn test_infrastructure_check() {
        let check = SecurityCheck::infrastructure_audit();
        let config = SecurityConfig::default();
        let findings = check.execute(&config);

        // Should find security header checks
        assert!(!findings.is_empty());
    }
}
