//! Security Auditor
//!
//! The main security auditing implementation that orchestrates security checks
//! and generates comprehensive security reports.

use super::models::{SecurityReport, SecurityStatus};
use super::{AuditContext, SecurityCheck, SecurityConfig};
use crate::error::{ErrorContext, Result};

/// Main security auditor that runs security checks and generates reports
#[derive(Debug, Clone)]
pub struct SecurityAuditor {
    config: SecurityConfig,
    context: Option<AuditContext>,
}

impl SecurityAuditor {
    /// Creates a new security auditor with default configuration
    pub fn new() -> Self {
        Self {
            config: SecurityConfig::default(),
            context: None,
        }
    }

    /// Creates a new security auditor with custom configuration
    pub fn with_config(config: SecurityConfig) -> Self {
        Self {
            config,
            context: None,
        }
    }

    /// Sets the audit context
    pub fn with_context(mut self, context: AuditContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Returns the current configuration
    pub fn config(&self) -> &SecurityConfig {
        &self.config
    }

    /// Updates the configuration
    pub fn set_config(&mut self, config: SecurityConfig) {
        self.config = config;
    }

    /// Runs a full security audit
    pub async fn run_full_audit(&self) -> Result<SecurityReport> {
        let mut report = SecurityReport::new();

        // Check if user has permission to run audit
        if let Some(ctx) = &self.context {
            if !ctx.can_run_audit() {
                return Err(crate::Error::Forbidden(
                    "Insufficient permissions to run security audit".to_string(),
                ));
            }
        }

        // Run all enabled checks
        let checks = self.get_enabled_checks();

        for check in checks {
            let findings = check.execute(&self.config);
            for finding in findings {
                if finding.level.meets(self.config.min_level) {
                    report.add_finding(finding);
                }
            }
        }

        // Generate recommendations based on findings
        self.generate_recommendations(&mut report);

        // Mark as complete
        report.complete();

        Ok(report)
    }

    /// Runs a specific security check
    pub async fn run_check(&self, check: SecurityCheck) -> Result<SecurityReport> {
        let mut report = SecurityReport::new();

        // Check permissions
        if let Some(ctx) = &self.context {
            if !ctx.can_run_audit() {
                return Err(crate::Error::Forbidden(
                    "Insufficient permissions to run security audit".to_string(),
                ));
            }
        }

        // Run the specific check
        let findings = check.execute(&self.config);
        for finding in findings {
            if self.config.include_info || finding.level > super::SecurityLevel::Info {
                report.add_finding(finding);
            }
        }

        report.complete();
        Ok(report)
    }

    /// Runs checks for a specific category
    pub async fn run_category_audit(
        &self,
        category: super::SecurityCategory,
    ) -> Result<SecurityReport> {
        let mut report = SecurityReport::new();

        let check = match category {
            super::SecurityCategory::Authentication => SecurityCheck::authentication_audit(),
            super::SecurityCategory::Authorization => SecurityCheck::authorization_audit(),
            super::SecurityCategory::InputValidation => SecurityCheck::input_validation_audit(),
            super::SecurityCategory::DataProtection => SecurityCheck::data_protection_audit(),
            super::SecurityCategory::EventSystem => SecurityCheck::event_system_audit(),
            super::SecurityCategory::Infrastructure => SecurityCheck::infrastructure_audit(),
            super::SecurityCategory::TenantSecurity => SecurityCheck::tenant_security_audit(),
        };

        let findings = check.execute(&self.config);
        for finding in findings {
            report.add_finding(finding);
        }

        report.complete();
        Ok(report)
    }

    /// Returns the list of enabled security checks based on configuration
    fn get_enabled_checks(&self) -> Vec<SecurityCheck> {
        let mut checks = Vec::new();

        if self.config.enabled_categories.contains(&super::SecurityCategory::Authentication) {
            checks.push(SecurityCheck::authentication_audit());
        }

        if self.config.enabled_categories.contains(&super::SecurityCategory::Authorization) {
            checks.push(SecurityCheck::authorization_audit());
        }

        if self.config.enabled_categories.contains(&super::SecurityCategory::InputValidation) {
            checks.push(SecurityCheck::input_validation_audit());
        }

        if self.config.enabled_categories.contains(&super::SecurityCategory::DataProtection) {
            checks.push(SecurityCheck::data_protection_audit());
        }

        if self.config.enabled_categories.contains(&super::SecurityCategory::EventSystem) {
            checks.push(SecurityCheck::event_system_audit());
        }

        if self.config.enabled_categories.contains(&super::SecurityCategory::Infrastructure) {
            checks.push(SecurityCheck::infrastructure_audit());
        }

        if self.config.enabled_categories.contains(&super::SecurityCategory::TenantSecurity) {
            checks.push(SecurityCheck::tenant_security_audit());
        }

        checks
    }

    /// Generates recommendations based on report findings
    fn generate_recommendations(&self, report: &mut SecurityReport) {
        let critical_findings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.level == super::SecurityLevel::Critical)
            .collect();

        let high_findings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.level == super::SecurityLevel::High)
            .collect();

        // Add general recommendations based on findings
        if !critical_findings.is_empty() {
            report.add_recommendation(
                super::Remediation::new("GEN-001", "Address Critical Findings Immediately")
                    .with_description(format!(
                        "{} critical security findings require immediate attention",
                        critical_findings.len()
                    ))
                    .with_step("Review all critical findings in the report")
                    .with_step("Prioritize based on business impact")
                    .with_step("Assign owners for each finding")
                    .with_step("Set remediation deadlines")
                    .with_step("Implement emergency response procedures if needed")
                    .with_priority(super::SecurityLevel::Critical)
                    .with_effort(0),
            );
        }

        if !high_findings.is_empty() {
            report.add_recommendation(
                super::Remediation::new("GEN-002", "Address High Severity Findings")
                    .with_description(format!(
                        "{} high severity findings should be addressed in the next sprint",
                        high_findings.len()
                    ))
                    .with_step("Review all high severity findings")
                    .with_step("Estimate effort for each remediation")
                    .with_step("Schedule remediation work")
                    .with_step("Add security testing to CI/CD")
                    .with_priority(super::SecurityLevel::High)
                    .with_effort(0),
            );
        }

        // Add general security hygiene recommendations
        report.add_recommendation(
            super::Remediation::new("GEN-003", "Implement Security Monitoring")
                .with_description("Set up continuous security monitoring and alerting")
                .with_step("Configure security event logging")
                .with_step("Set up alerts for suspicious activity")
                .with_step("Implement SIEM integration if available")
                .with_step("Create security incident response procedures")
                .with_step("Schedule regular security reviews")
                .with_priority(super::SecurityLevel::Medium)
                .with_effort(16),
        );

        report.add_recommendation(
            super::Remediation::new("GEN-004", "Establish Security Training Program")
                .with_description("Ensure all developers and administrators receive security training")
                .with_step("Create secure coding guidelines")
                .with_step("Conduct OWASP Top 10 training")
                .with_step("Establish secure code review process")
                .with_step("Run periodic security exercises")
                .with_priority(super::SecurityLevel::Medium)
                .with_effort(24),
        );
    }

    /// Validates the current configuration
    pub fn validate_config(&self) -> Vec<String> {
        let mut issues = Vec::new();

        // Validate password policy
        if self.config.policies.password_policy.min_length < 8 {
            issues.push("Password minimum length should be at least 8 characters".to_string());
        }

        // Validate session policy
        if self.config.policies.session_policy.timeout_minutes == 0 {
            issues.push("Session timeout should not be 0 (no timeout)".to_string());
        }

        // Validate rate limiting
        if self.config.policies.rate_limit_policy.login_attempts_per_minute > 10 {
            issues.push("Login rate limit seems high (> 10 attempts per minute)".to_string());
        }

        issues
    }
}

impl Default for SecurityAuditor {
    fn default() -> Self {
        Self::new()
    }
}

/// Security audit result for a specific component
#[derive(Debug, Clone)]
pub struct ComponentAuditResult {
    /// Component name
    pub component: String,
    /// Audit status
    pub status: SecurityStatus,
    /// Number of findings
    pub finding_count: usize,
    /// Highest severity level found
    pub highest_severity: Option<super::SecurityLevel>,
    /// Audit duration in milliseconds
    pub duration_ms: u64,
}

impl ComponentAuditResult {
    pub fn new(component: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            status: SecurityStatus::Pending,
            finding_count: 0,
            highest_severity: None,
            duration_ms: 0,
        }
    }

    pub fn passed(&self) -> bool {
        self.status == SecurityStatus::Completed && self.finding_count == 0
    }

    pub fn has_critical(&self) -> bool {
        matches!(self.highest_severity, Some(super::SecurityLevel::Critical))
    }
}

/// Continuous security monitor
#[derive(Debug)]
pub struct SecurityMonitor {
    auditor: SecurityAuditor,
    check_interval: std::time::Duration,
    alert_threshold: super::SecurityLevel,
}

impl SecurityMonitor {
    /// Creates a new security monitor
    pub fn new(auditor: SecurityAuditor) -> Self {
        Self {
            auditor,
            check_interval: std::time::Duration::from_secs(3600), // 1 hour default
            alert_threshold: super::SecurityLevel::High,
        }
    }

    /// Sets the check interval
    pub fn with_interval(mut self, interval: std::time::Duration) -> Self {
        self.check_interval = interval;
        self
    }

    /// Sets the alert threshold
    pub fn with_threshold(mut self, threshold: super::SecurityLevel) -> Self {
        self.alert_threshold = threshold;
        self
    }

    /// Runs a continuous monitoring loop (should be spawned as a task)
    pub async fn run(&self) {
        let mut interval = tokio::time::interval(self.check_interval);

        loop {
            interval.tick().await;

            match self.auditor.run_full_audit().await {
                Ok(report) => {
                    if report.has_high_or_critical() {
                        self.send_alert(&report).await;
                    }
                }
                Err(e) => {
                    tracing::error!("Security audit failed: {}", e);
                }
            }
        }
    }

    /// Sends an alert about security findings
    async fn send_alert(&self, report: &SecurityReport) {
        tracing::warn!(
            "Security Alert: {} critical, {} high severity findings detected",
            report.summary.critical_count,
            report.summary.high_count
        );

        // In a real implementation, this would send alerts via:
        // - Email notifications
        // - Slack/Teams webhooks
        // - PagerDuty integration
        // - Custom webhook endpoints
    }
}

/// Batch security auditor for multiple systems
#[derive(Debug)]
pub struct BatchSecurityAuditor {
    auditors: Vec<(String, SecurityAuditor)>,
}

impl BatchSecurityAuditor {
    /// Creates a new batch auditor
    pub fn new() -> Self {
        Self {
            auditors: Vec::new(),
        }
    }

    /// Adds an auditor for a specific system
    pub fn add_auditor(&mut self, name: impl Into<String>, auditor: SecurityAuditor) {
        self.auditors.push((name.into(), auditor));
    }

    /// Runs audits for all systems
    pub async fn run_all(&self) -> HashMap<String, SecurityReport> {
        let mut results = HashMap::new();

        for (name, auditor) in &self.auditors {
            match auditor.run_full_audit().await {
                Ok(report) => {
                    results.insert(name.clone(), report);
                }
                Err(e) => {
                    tracing::error!("Audit failed for {}: {}", name, e);
                }
            }
        }

        results
    }

    /// Returns a consolidated report from all systems
    pub fn consolidate_reports(&self, reports: &HashMap<String, SecurityReport>) -> SecurityReport {
        let mut consolidated = SecurityReport::new();

        for (system, report) in reports {
            for finding in &report.findings {
                let mut adapted_finding = finding.clone();
                adapted_finding
                    .affected_resources
                    .insert(0, format!("system:{}", system));
                consolidated.add_finding(adapted_finding);
            }

            for rec in &report.recommendations {
                consolidated.add_recommendation(rec.clone());
            }
        }

        consolidated.complete();
        consolidated
    }
}

impl Default for BatchSecurityAuditor {
    fn default() -> Self {
        Self::new()
    }
}

use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rbac::SecurityContext;
    use crate::types::UserRole;

    #[test]
    fn test_security_auditor_creation() {
        let auditor = SecurityAuditor::new();
        assert_eq!(auditor.config().min_level, super::super::SecurityLevel::Low);
    }

    #[test]
    fn test_security_auditor_with_context() {
        let ctx = AuditContext::new(SecurityContext::new(UserRole::SuperAdmin, Some(uuid::Uuid::new_v4())));
        let auditor = SecurityAuditor::new().with_context(ctx);
        assert!(auditor.context.is_some());
    }

    #[test]
    fn test_component_audit_result() {
        let mut result = ComponentAuditResult::new("test-component");
        result.status = SecurityStatus::Completed;
        result.finding_count = 0;

        assert!(result.passed());
        assert!(!result.has_critical());
    }

    #[test]
    fn test_validate_config() {
        let mut config = SecurityConfig::default();
        config.policies.password_policy.min_length = 6;
        config.policies.session_policy.timeout_minutes = 0;

        let auditor = SecurityAuditor::with_config(config);
        let issues = auditor.validate_config();

        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.contains("Password")));
        assert!(issues.iter().any(|i| i.contains("Session")));
    }

    #[test]
    fn test_get_enabled_checks() {
        let config = SecurityConfig::default();
        let auditor = SecurityAuditor::with_config(config);

        let checks = auditor.get_enabled_checks();
        assert_eq!(checks.len(), 7); // All categories enabled by default
    }
}
