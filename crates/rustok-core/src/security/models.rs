//! Security Audit Models
//!
//! This module defines the core types and models used in security auditing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Security severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityLevel {
    /// Informational only, no action required
    Info = 0,
    /// Low severity, should be addressed when convenient
    Low = 1,
    /// Medium severity, should be addressed in next sprint
    Medium = 2,
    /// High severity, requires immediate attention
    High = 3,
    /// Critical severity, requires immediate action
    Critical = 4,
}

impl SecurityLevel {
    /// Returns true if this level meets or exceeds the given level
    pub fn meets(&self, other: SecurityLevel) -> bool {
        *self >= other
    }

    /// Returns a user-friendly description of this level
    pub fn description(&self) -> &'static str {
        match self {
            SecurityLevel::Info => "Informational finding, no immediate action required",
            SecurityLevel::Low => "Low severity, address when convenient",
            SecurityLevel::Medium => "Medium severity, address in next maintenance window",
            SecurityLevel::High => "High severity, requires immediate attention",
            SecurityLevel::Critical => "Critical severity, requires immediate action",
        }
    }

    /// Returns the CSS color class for UI display
    pub fn color_class(&self) -> &'static str {
        match self {
            SecurityLevel::Info => "text-blue-500",
            SecurityLevel::Low => "text-green-500",
            SecurityLevel::Medium => "text-yellow-500",
            SecurityLevel::High => "text-orange-500",
            SecurityLevel::Critical => "text-red-500",
        }
    }

    /// Returns the HTTP status code equivalent
    pub fn http_status_code(&self) -> u16 {
        match self {
            SecurityLevel::Info => 200,
            SecurityLevel::Low => 200,
            SecurityLevel::Medium => 400,
            SecurityLevel::High => 403,
            SecurityLevel::Critical => 500,
        }
    }
}

impl fmt::Display for SecurityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SecurityLevel::Info => "info",
                SecurityLevel::Low => "low",
                SecurityLevel::Medium => "medium",
                SecurityLevel::High => "high",
                SecurityLevel::Critical => "critical",
            }
        )
    }
}

/// Security audit status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityStatus {
    /// Audit is pending
    Pending,
    /// Audit is in progress
    InProgress,
    /// Audit completed successfully
    Completed,
    /// Audit failed
    Failed,
    /// Audit was cancelled
    Cancelled,
}

/// A security finding from an audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFinding {
    /// Unique identifier
    pub id: String,
    /// Security category
    pub category: super::SecurityCategory,
    /// Severity level
    pub level: SecurityLevel,
    /// Finding title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Affected resources
    pub affected_resources: Vec<String>,
    /// Evidence or proof
    pub evidence: Vec<Evidence>,
    /// Compliance frameworks affected
    pub compliance_frameworks: Vec<ComplianceFramework>,
    /// CVSS score if applicable (0.0 - 10.0)
    pub cvss_score: Option<f32>,
    /// CVE identifier if applicable
    pub cve_id: Option<String>,
    /// CWE identifier if applicable
    pub cwe_id: Option<String>,
    /// When the finding was discovered
    pub discovered_at: DateTime<Utc>,
    /// Remediation information
    pub remediation: Option<super::Remediation>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl SecurityFinding {
    /// Creates a new security finding
    pub fn new(
        category: super::SecurityCategory,
        level: SecurityLevel,
        title: impl Into<String>,
    ) -> Self {
        Self {
            id: format!("FIND-{}-{:06}", level.to_string().to_uppercase(), Uuid::new_v4().as_u128() % 1_000_000),
            category,
            level,
            title: title.into(),
            description: String::new(),
            affected_resources: Vec::new(),
            evidence: Vec::new(),
            compliance_frameworks: Vec::new(),
            cvss_score: None,
            cve_id: None,
            cwe_id: None,
            discovered_at: Utc::now(),
            remediation: None,
            metadata: HashMap::new(),
        }
    }

    /// Adds a description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Adds an affected resource
    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.affected_resources.push(resource.into());
        self
    }

    /// Adds evidence
    pub fn with_evidence(mut self, evidence: Evidence) -> Self {
        self.evidence.push(evidence);
        self
    }

    /// Adds a compliance framework
    pub fn with_compliance(mut self, framework: ComplianceFramework) -> Self {
        self.compliance_frameworks.push(framework);
        self
    }

    /// Sets the CVSS score
    pub fn with_cvss(mut self, score: f32) -> Self {
        self.cvss_score = Some(score.clamp(0.0, 10.0));
        self
    }

    /// Sets the CVE identifier
    pub fn with_cve(mut self, cve: impl Into<String>) -> Self {
        self.cve_id = Some(cve.into());
        self
    }

    /// Sets the CWE identifier
    pub fn with_cwe(mut self, cwe: impl Into<String>) -> Self {
        self.cwe_id = Some(cwe.into());
        self
    }

    /// Sets the remediation
    pub fn with_remediation(mut self, remediation: super::Remediation) -> Self {
        self.remediation = Some(remediation);
        self
    }

    /// Adds metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Returns true if this finding is at or above the given level
    pub fn is_at_least(&self, level: SecurityLevel) -> bool {
        self.level >= level
    }

    /// Returns the risk rating based on CVSS score
    pub fn risk_rating(&self) -> &'static str {
        match self.cvss_score {
            None => "Unknown",
            Some(score) if score >= 9.0 => "Critical",
            Some(score) if score >= 7.0 => "High",
            Some(score) if score >= 4.0 => "Medium",
            Some(score) if score >= 0.1 => "Low",
            _ => "None",
        }
    }
}

/// Evidence for a security finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    /// Type of evidence
    pub evidence_type: EvidenceType,
    /// Description of the evidence
    pub description: String,
    /// Raw evidence data
    pub data: String,
    /// Timestamp when evidence was collected
    pub collected_at: DateTime<Utc>,
    /// Source of the evidence
    pub source: String,
}

impl Evidence {
    pub fn new(evidence_type: EvidenceType, description: impl Into<String>) -> Self {
        Self {
            evidence_type,
            description: description.into(),
            data: String::new(),
            collected_at: Utc::now(),
            source: String::new(),
        }
    }

    pub fn with_data(mut self, data: impl Into<String>) -> Self {
        self.data = data.into();
        self
    }

    pub fn from_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }
}

/// Types of evidence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    /// Log entry
    LogEntry,
    /// Configuration file
    Configuration,
    /// Code snippet
    Code,
    /// Network packet capture
    NetworkCapture,
    /// Database record
    DatabaseRecord,
    /// API response
    ApiResponse,
    /// Screenshot
    Screenshot,
    /// File hash
    FileHash,
    /// Certificate
    Certificate,
    /// Other evidence type
    Other,
}

/// Compliance frameworks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceFramework {
    /// SOC 2
    Soc2,
    /// ISO 27001
    Iso27001,
    /// GDPR
    Gdpr,
    /// HIPAA
    Hipaa,
    /// PCI DSS
    PciDss,
    /// NIST Cybersecurity Framework
    NistCsf,
    /// CIS Controls
    CisControls,
    /// OWASP Top 10
    OwaspTop10,
}

impl ComplianceFramework {
    pub fn name(&self) -> &'static str {
        match self {
            ComplianceFramework::Soc2 => "SOC 2",
            ComplianceFramework::Iso27001 => "ISO 27001",
            ComplianceFramework::Gdpr => "GDPR",
            ComplianceFramework::Hipaa => "HIPAA",
            ComplianceFramework::PciDss => "PCI DSS",
            ComplianceFramework::NistCsf => "NIST CSF",
            ComplianceFramework::CisControls => "CIS Controls",
            ComplianceFramework::OwaspTop10 => "OWASP Top 10",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ComplianceFramework::Soc2 => "Service Organization Control 2",
            ComplianceFramework::Iso27001 => "Information Security Management System",
            ComplianceFramework::Gdpr => "General Data Protection Regulation",
            ComplianceFramework::Hipaa => "Health Insurance Portability and Accountability Act",
            ComplianceFramework::PciDss => "Payment Card Industry Data Security Standard",
            ComplianceFramework::NistCsf => "NIST Cybersecurity Framework",
            ComplianceFramework::CisControls => "Center for Internet Security Controls",
            ComplianceFramework::OwaspTop10 => "OWASP Top 10 Security Risks",
        }
    }
}

/// Complete security audit report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReport {
    /// Report identifier
    pub id: Uuid,
    /// Report version
    pub version: String,
    /// When the audit started
    pub started_at: DateTime<Utc>,
    /// When the audit completed
    pub completed_at: Option<DateTime<Utc>>,
    /// Overall security status
    pub overall_status: SecurityStatus,
    /// Summary statistics
    pub summary: SecuritySummary,
    /// All findings
    pub findings: Vec<SecurityFinding>,
    /// Findings grouped by category
    pub findings_by_category: HashMap<String, Vec<SecurityFinding>>,
    /// Findings grouped by level
    pub findings_by_level: HashMap<String, Vec<SecurityFinding>>,
    /// Compliance summary
    pub compliance_summary: HashMap<String, ComplianceStatus>,
    /// Recommendations
    pub recommendations: Vec<super::Remediation>,
    /// Audit metadata
    pub metadata: HashMap<String, String>,
}

impl SecurityReport {
    /// Creates a new security report
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            version: "1.0.0".to_string(),
            started_at: Utc::now(),
            completed_at: None,
            overall_status: SecurityStatus::Pending,
            summary: SecuritySummary::default(),
            findings: Vec::new(),
            findings_by_category: HashMap::new(),
            findings_by_level: HashMap::new(),
            compliance_summary: HashMap::new(),
            recommendations: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Adds a finding to the report
    pub fn add_finding(&mut self, finding: SecurityFinding) {
        // Update summary counts
        match finding.level {
            SecurityLevel::Critical => self.summary.critical_count += 1,
            SecurityLevel::High => self.summary.high_count += 1,
            SecurityLevel::Medium => self.summary.medium_count += 1,
            SecurityLevel::Low => self.summary.low_count += 1,
            SecurityLevel::Info => self.summary.info_count += 1,
        }
        self.summary.total_count += 1;

        // Add to category group
        let category_key = format!("{:?}", finding.category).to_lowercase();
        self.findings_by_category
            .entry(category_key)
            .or_default()
            .push(finding.clone());

        // Add to level group
        let level_key = finding.level.to_string();
        self.findings_by_level
            .entry(level_key)
            .or_default()
            .push(finding.clone());

        self.findings.push(finding);
    }

    /// Adds a recommendation
    pub fn add_recommendation(&mut self, recommendation: super::Remediation) {
        self.recommendations.push(recommendation);
    }

    /// Marks the report as completed
    pub fn complete(&mut self) {
        self.completed_at = Some(Utc::now());
        self.overall_status = SecurityStatus::Completed;
        self.calculate_risk_score();
    }

    /// Calculates the overall risk score (0-100)
    fn calculate_risk_score(&mut self) {
        if self.findings.is_empty() {
            self.summary.risk_score = 0;
            return;
        }

        let weighted_sum: u32 = self
            .findings
            .iter()
            .map(|f| match f.level {
                SecurityLevel::Critical => 100,
                SecurityLevel::High => 50,
                SecurityLevel::Medium => 20,
                SecurityLevel::Low => 5,
                SecurityLevel::Info => 0,
            })
            .sum();

        self.summary.risk_score = (weighted_sum / self.findings.len() as u32).min(100);
    }

    /// Returns true if there are critical findings
    pub fn has_critical_findings(&self) -> bool {
        self.summary.critical_count > 0
    }

    /// Returns true if there are high or critical findings
    pub fn has_high_or_critical(&self) -> bool {
        self.summary.critical_count > 0 || self.summary.high_count > 0
    }

    /// Returns findings filtered by level
    pub fn findings_by_level(&self, level: SecurityLevel) -> Vec<&SecurityFinding> {
        self.findings
            .iter()
            .filter(|f| f.level == level)
            .collect()
    }

    /// Returns findings filtered by category
    pub fn findings_by_category(
        &self,
        category: super::SecurityCategory,
    ) -> Vec<&SecurityFinding> {
        self.findings
            .iter()
            .filter(|f| f.category == category)
            .collect()
    }
}

impl Default for SecurityReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Security summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySummary {
    /// Total number of findings
    pub total_count: usize,
    /// Critical findings count
    pub critical_count: usize,
    /// High findings count
    pub high_count: usize,
    /// Medium findings count
    pub medium_count: usize,
    /// Low findings count
    pub low_count: usize,
    /// Info findings count
    pub info_count: usize,
    /// Overall risk score (0-100)
    pub risk_score: u32,
    /// Number of checks performed
    pub checks_performed: usize,
    /// Duration of the audit in seconds
    pub duration_seconds: u64,
}

impl Default for SecuritySummary {
    fn default() -> Self {
        Self {
            total_count: 0,
            critical_count: 0,
            high_count: 0,
            medium_count: 0,
            low_count: 0,
            info_count: 0,
            risk_score: 0,
            checks_performed: 0,
            duration_seconds: 0,
        }
    }
}

/// Compliance status for a specific framework
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    /// Compliance framework
    pub framework: ComplianceFramework,
    /// Overall compliance percentage (0-100)
    pub compliance_percentage: f32,
    /// Number of passing controls
    pub passing_controls: usize,
    /// Number of failing controls
    pub failing_controls: usize,
    /// Number of not applicable controls
    pub na_controls: usize,
    /// Detailed control results
    pub control_results: Vec<ControlResult>,
}

/// Individual control result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlResult {
    /// Control identifier
    pub control_id: String,
    /// Control name
    pub name: String,
    /// Control description
    pub description: String,
    /// Whether the control passed
    pub passed: bool,
    /// Finding IDs related to this control
    pub related_findings: Vec<String>,
    /// Evidence for the result
    pub evidence: String,
}

use std::fmt;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_level_ordering() {
        assert!(SecurityLevel::Critical > SecurityLevel::High);
        assert!(SecurityLevel::High > SecurityLevel::Medium);
        assert!(SecurityLevel::Medium > SecurityLevel::Low);
        assert!(SecurityLevel::Low > SecurityLevel::Info);
        assert!(SecurityLevel::Critical.meets(SecurityLevel::High));
    }

    #[test]
    fn test_security_finding_builder() {
        let finding = SecurityFinding::new(
            super::super::SecurityCategory::Authentication,
            SecurityLevel::High,
            "Weak Password Policy",
        )
        .with_description("The password policy does not meet security standards")
        .with_resource("auth::password_policy")
        .with_cvss(7.5);

        assert_eq!(finding.level, SecurityLevel::High);
        assert_eq!(finding.cvss_score, Some(7.5));
        assert_eq!(finding.affected_resources.len(), 1);
    }

    #[test]
    fn test_security_report_add_finding() {
        let mut report = SecurityReport::new();
        let finding = SecurityFinding::new(
            super::super::SecurityCategory::Authentication,
            SecurityLevel::Critical,
            "Test Finding",
        );

        report.add_finding(finding);

        assert_eq!(report.summary.total_count, 1);
        assert_eq!(report.summary.critical_count, 1);
        assert!(report.has_critical_findings());
    }

    #[test]
    fn test_risk_score_calculation() {
        let mut report = SecurityReport::new();
        report.add_finding(SecurityFinding::new(
            super::super::SecurityCategory::Authentication,
            SecurityLevel::Critical,
            "Critical",
        ));
        report.add_finding(SecurityFinding::new(
            super::super::SecurityCategory::Authentication,
            SecurityLevel::Low,
            "Low",
        ));

        report.complete();

        assert!(report.summary.risk_score > 0);
        assert!(report.summary.risk_score <= 100);
    }

    #[test]
    fn test_evidence_builder() {
        let evidence = Evidence::new(EvidenceType::LogEntry, "Suspicious login attempt")
            .with_data("2024-01-01 00:00:00 Failed login from 192.168.1.1")
            .from_source("/var/log/auth.log");

        assert_eq!(evidence.evidence_type, EvidenceType::LogEntry);
        assert!(!evidence.data.is_empty());
    }
}
