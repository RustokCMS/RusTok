# Security Audit Guide

This guide covers the security audit capabilities of the RusToK platform.

## Overview

The Security Audit module provides comprehensive security auditing capabilities covering:

- **Authentication**: Password policies, MFA, session management
- **Authorization**: Role-based access control, permission scopes
- **Input Validation**: SQL injection, XSS, path traversal prevention
- **Data Protection**: Encryption, sensitive data handling
- **Event System**: Event validation, integrity, replay protection
- **Infrastructure**: Security headers, TLS, rate limiting
- **Tenant Security**: Multi-tenant isolation and validation

## Quick Start

### Running a Full Security Audit

```rust
use rustok_core::security::{SecurityAuditor, AuditContext, SecurityContext};
use rustok_core::types::UserRole;

// Create auditor with context
let ctx = AuditContext::new(
    SecurityContext::new(UserRole::SuperAdmin, None)
);

let auditor = SecurityAuditor::new()
    .with_context(ctx);

// Run full audit
let report = auditor.run_full_audit().await?;

// Check results
if report.has_critical_findings() {
    println!("Critical issues found!");
}

println!("Risk Score: {}/100", report.summary.risk_score);
```

### Running Specific Checks

```rust
use rustok_core::security::SecurityCheck;

// Authentication audit only
let auth_check = SecurityCheck::authentication_audit();
let auth_report = auditor.run_check(auth_check).await?;

// Tenant security audit
let tenant_check = SecurityCheck::tenant_security_audit();
let tenant_report = auditor.run_check(tenant_check).await?;
```

### Generating Reports

```rust
use rustok_core::security::SecurityReportGenerator;

let generator = SecurityReportGenerator::new()
    .with_evidence(true)
    .with_remediation(true);

// Generate JSON report
let json = generator.generate_json(&report)?;

// Generate Markdown report
let markdown = generator.generate_markdown(&report);

// Generate HTML report
let html = generator.generate_html(&report);
```

## Configuration

### Security Policies

```rust
use rustok_core::security::{
    SecurityConfig, SecurityPolicies, PasswordPolicy, SessionPolicy
};

let config = SecurityConfig {
    min_level: SecurityLevel::Low,
    include_info: true,
    policies: SecurityPolicies {
        password_policy: PasswordPolicy {
            min_length: 12,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special: true,
            max_age_days: 90,
            prevent_reuse_count: 5,
        },
        session_policy: SessionPolicy {
            timeout_minutes: 30,
            absolute_timeout_minutes: 480,
            require_reauth_for_sensitive: true,
            invalidate_on_password_change: true,
            max_concurrent_sessions: 5,
        },
        ..Default::default()
    },
    ..Default::default()
};

let auditor = SecurityAuditor::with_config(config);
```

### Category Selection

```rust
use rustok_core::security::SecurityCategory;

// Audit only specific categories
let config = SecurityConfig {
    enabled_categories: vec![
        SecurityCategory::Authentication,
        SecurityCategory::Authorization,
    ],
    ..Default::default()
};
```

## Security Findings

### Finding Structure

```rust
use rustok_core::security::{SecurityFinding, SecurityLevel, SecurityCategory};

let finding = SecurityFinding::new(
    SecurityCategory::Authentication,
    SecurityLevel::High,
    "Weak Password Policy"
)
.with_description("Password policy does not meet security standards")
.with_resource("auth::password_policy")
.with_cvss(7.5)
.with_cwe("CWE-521")
.with_remediation(
    Remediation::new("AUTH-001", "Strengthen Password Policy")
        .with_step("Increase minimum password length to 12")
        .with_step("Require special characters")
        .with_effort(2)
);
```

### Severity Levels

| Level | Description | Response Time |
|-------|-------------|---------------|
| Info | Informational only | None |
| Low | Minor issue | Next maintenance |
| Medium | Moderate concern | Next sprint |
| High | Significant risk | Within 1 week |
| Critical | Severe vulnerability | Immediate |

## Remediation

### Creating Remediations

```rust
use rustok_core::security::Remediation;

let remediation = Remediation::new("AUTH-001", "Enable MFA")
    .with_description("Require multi-factor authentication for admin accounts")
    .with_step("Configure MFA provider")
    .with_step("Update authentication flow")
    .with_step("Test MFA enrollment")
    .with_priority(SecurityLevel::High)
    .with_effort(8)
    .with_resource("https://docs.example.com/mfa");
```

## Continuous Monitoring

### Setting Up Security Monitoring

```rust
use rustok_core::security::SecurityMonitor;
use std::time::Duration;

let monitor = SecurityMonitor::new(auditor)
    .with_interval(Duration::from_secs(3600))  // Check every hour
    .with_threshold(SecurityLevel::High);       // Alert on high+ findings

// Run in background
tokio::spawn(async move {
    monitor.run().await;
});
```

### Batch Auditing Multiple Systems

```rust
use rustok_core::security::BatchSecurityAuditor;

let mut batch = BatchSecurityAuditor::new();
batch.add_auditor("production", prod_auditor);
batch.add_auditor("staging", staging_auditor);

// Run all audits
let results = batch.run_all().await;

// Get consolidated report
let consolidated = batch.consolidate_reports(&results);
```

## Compliance Mapping

### Supported Frameworks

- **SOC 2**: Service Organization Control 2
- **ISO 27001**: Information Security Management
- **GDPR**: Data Protection Regulation
- **HIPAA**: Healthcare data protection
- **PCI DSS**: Payment Card Industry standards
- **NIST CSF**: Cybersecurity Framework
- **CIS Controls**: Security controls
- **OWASP Top 10**: Web application security

### Adding Compliance to Findings

```rust
let finding = SecurityFinding::new(
    SecurityCategory::DataProtection,
    SecurityLevel::High,
    "Missing Encryption"
)
.with_compliance(ComplianceFramework::PciDss)
.with_compliance(ComplianceFramework::Gdpr);
```

## Best Practices

### 1. Regular Audits

Schedule security audits:
- **Daily**: Automated vulnerability scans
- **Weekly**: Configuration audits
- **Monthly**: Full security audits
- **Quarterly**: External penetration testing

### 2. Remediation Workflow

1. **Triage**: Assess severity and impact
2. **Prioritize**: Address critical findings first
3. **Assign**: Designate responsible teams
4. **Remediate**: Implement fixes
5. **Verify**: Confirm remediation
6. **Document**: Record lessons learned

### 3. Integration with CI/CD

```yaml
# Example GitHub Actions workflow
- name: Security Audit
  run: |
    cargo run --bin security-audit
    
- name: Check for Critical Findings
  run: |
    if grep -q "CRITICAL" audit-report.json; then
      echo "Critical security findings detected!"
      exit 1
    fi
```

### 4. Reporting

Generate reports for different audiences:
- **Executive Summary**: High-level risk overview
- **Technical Report**: Detailed findings for developers
- **Compliance Report**: Mapping to frameworks
- **Remediation Plan**: Action items with timelines

## API Reference

### SecurityAuditor

```rust
impl SecurityAuditor {
    pub fn new() -> Self;
    pub fn with_config(config: SecurityConfig) -> Self;
    pub fn with_context(self, context: AuditContext) -> Self;
    pub async fn run_full_audit(&self) -> Result<SecurityReport>;
    pub async fn run_check(&self, check: SecurityCheck) -> Result<SecurityReport>;
    pub async fn run_category_audit(&self, category: SecurityCategory) -> Result<SecurityReport>;
    pub fn validate_config(&self) -> Vec<String>;
}
```

### SecurityReport

```rust
impl SecurityReport {
    pub fn new() -> Self;
    pub fn add_finding(&mut self, finding: SecurityFinding);
    pub fn has_critical_findings(&self) -> bool;
    pub fn has_high_or_critical(&self) -> bool;
    pub fn findings_by_level(&self, level: SecurityLevel) -> Vec<&SecurityFinding>;
    pub fn findings_by_category(&self, category: SecurityCategory) -> Vec<&SecurityFinding>;
}
```

## Troubleshooting

### Common Issues

**Permission Denied**
```
Error: Insufficient permissions to run security audit
```
Ensure the user has Admin or SuperAdmin role.

**No Findings Reported**
Check the minimum level configuration:
```rust
config.min_level = SecurityLevel::Info;  // Include all findings
```

**High Memory Usage**
For large systems, limit findings per category:
```rust
let generator = SecurityReportGenerator::new()
    .with_max_findings(100);
```

## Further Reading

- [OWASP Testing Guide](https://owasp.org/www-project-web-security-testing-guide/)
- [NIST Cybersecurity Framework](https://www.nist.gov/cyberframework)
- [CIS Controls](https://www.cisecurity.org/controls)
- [Security Audit Checklist](SECURITY_CHECKLIST.md)

## Support

For security-related questions or to report vulnerabilities:
- Security Team: security@rustok.io
- Issue Tracker: [GitHub Issues](https://github.com/rustok/security)
- Documentation: [docs.rustok.io](https://docs.rustok.io)
