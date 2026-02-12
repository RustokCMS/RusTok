//! Security Report Generator
//!
//! This module provides functionality for generating security reports
//! in various formats (JSON, Markdown, HTML, PDF-ready).

use super::models::{ComplianceFramework, SecurityFinding, SecurityLevel, SecurityReport};
use super::SecurityCategory;
use chrono::Utc;
use std::collections::HashMap;

/// Report generator for security audit reports
#[derive(Debug, Clone)]
pub struct SecurityReportGenerator {
    /// Include detailed evidence
    pub include_evidence: bool,
    /// Include remediation steps
    pub include_remediation: bool,
    /// Include compliance mapping
    pub include_compliance: bool,
    /// Maximum findings per category
    pub max_findings_per_category: Option<usize>,
}

impl Default for SecurityReportGenerator {
    fn default() -> Self {
        Self {
            include_evidence: true,
            include_remediation: true,
            include_compliance: true,
            max_findings_per_category: None,
        }
    }
}

impl SecurityReportGenerator {
    /// Creates a new report generator with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether to include evidence
    pub fn with_evidence(mut self, include: bool) -> Self {
        self.include_evidence = include;
        self
    }

    /// Sets whether to include remediation
    pub fn with_remediation(mut self, include: bool) -> Self {
        self.include_remediation = include;
        self
    }

    /// Sets whether to include compliance
    pub fn with_compliance(mut self, include: bool) -> Self {
        self.include_compliance = include;
        self
    }

    /// Sets the maximum findings per category
    pub fn with_max_findings(mut self, max: usize) -> Self {
        self.max_findings_per_category = Some(max);
        self
    }

    /// Generates a JSON report
    pub fn generate_json(&self, report: &SecurityReport) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(report)
    }

    /// Generates a compact JSON report (without evidence for large reports)
    pub fn generate_compact_json(&self, report: &SecurityReport) -> Result<String, serde_json::Error> {
        // Create a summary version with limited findings
        let mut compact = report.clone();
        
        if let Some(max) = self.max_findings_per_category {
            compact.findings.truncate(max);
        }

        if !self.include_evidence {
            for finding in &mut compact.findings {
                finding.evidence.clear();
            }
        }

        serde_json::to_string_pretty(&compact)
    }

    /// Generates a Markdown report
    pub fn generate_markdown(&self, report: &SecurityReport) -> String {
        let mut md = String::new();

        // Header
        md.push_str(&format!("# Security Audit Report\n\n"));
        md.push_str(&format!("**Report ID:** {}\n\n", report.id));
        md.push_str(&format!("**Version:** {}\n\n", report.version));
        md.push_str(&format!("**Generated:** {}\n\n", report.started_at.format("%Y-%m-%d %H:%M:%S UTC")));
        
        if let Some(completed) = report.completed_at {
            md.push_str(&format!("**Completed:** {}\n\n", completed.format("%Y-%m-%d %H:%M:%S UTC")));
        }

        // Executive Summary
        md.push_str("## Executive Summary\n\n");
        md.push_str(&self.generate_executive_summary(report));
        md.push('\n');

        // Risk Score
        md.push_str(&format!("### Overall Risk Score: {}/100\n\n", report.summary.risk_score));
        md.push_str(&self.risk_score_description(report.summary.risk_score));
        md.push('\n');

        // Summary Statistics
        md.push_str("## Summary Statistics\n\n");
        md.push_str(&self.generate_summary_table(report));
        md.push('\n');

        // Findings by Severity
        md.push_str("## Findings by Severity\n\n");
        md.push_str(&self.generate_findings_by_severity(report));
        md.push('\n');

        // Findings by Category
        md.push_str("## Findings by Category\n\n");
        md.push_str(&self.generate_findings_by_category(report));
        md.push('\n');

        // Detailed Findings
        md.push_str("## Detailed Findings\n\n");
        for finding in &report.findings {
            md.push_str(&self.generate_finding_detail(finding));
            md.push('\n');
        }

        // Compliance Summary
        if self.include_compliance && !report.compliance_summary.is_empty() {
            md.push_str("## Compliance Summary\n\n");
            md.push_str(&self.generate_compliance_summary(report));
            md.push('\n');
        }

        // Recommendations
        if self.include_remediation && !report.recommendations.is_empty() {
            md.push_str("## Recommendations\n\n");
            for rec in &report.recommendations {
                md.push_str(&self.generate_recommendation(rec));
                md.push('\n');
            }
        }

        // Appendix
        md.push_str("---\n\n");
        md.push_str("## Appendix\n\n");
        md.push_str(&format!("- Total Checks Performed: {}\n", report.summary.checks_performed));
        md.push_str(&format!("- Audit Duration: {} seconds\n", report.summary.duration_seconds));
        md.push_str(&format!("- Report Generated by: RusToK Security Audit Framework\n"));

        md
    }

    /// Generates an HTML report
    pub fn generate_html(&self, report: &SecurityReport) -> String {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"en\">\n");
        html.push_str("<head>\n");
        html.push_str("<meta charset=\"UTF-8\">\n");
        html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str(&format!("<title>Security Audit Report - {}</title>\n", report.id));
        html.push_str(&self.generate_html_styles());
        html.push_str("</head>\n");
        html.push_str("<body>\n");

        // Header
        html.push_str("<div class=\"container\">\n");
        html.push_str("<header>\n");
        html.push_str("<h1>Security Audit Report</h1>\n");
        html.push_str(&format!("<p class=\"meta\">Report ID: {}</p>\n", report.id));
        html.push_str(&format!("<p class=\"meta\">Generated: {}</p>\n", report.started_at.format("%Y-%m-%d %H:%M:%S UTC")));
        html.push_str("</header>\n");

        // Risk Score Card
        html.push_str(&self.generate_risk_score_card(report));

        // Summary Cards
        html.push_str(&self.generate_summary_cards(report));

        // Findings Table
        html.push_str("<h2>Security Findings</h2>\n");
        html.push_str(&self.generate_findings_table(report));

        // Detailed Findings
        html.push_str("<h2>Detailed Findings</h2>\n");
        for finding in &report.findings {
            html.push_str(&self.generate_finding_card(finding));
        }

        // Recommendations
        if self.include_remediation && !report.recommendations.is_empty() {
            html.push_str("<h2>Recommendations</h2>\n");
            for rec in &report.recommendations {
                html.push_str(&self.generate_recommendation_card(rec));
            }
        }

        html.push_str("</div>\n");
        html.push_str("</body>\n");
        html.push_str("</html>\n");

        html
    }

    /// Generates an executive summary
    fn generate_executive_summary(&self, report: &SecurityReport) -> String {
        let mut summary = String::new();

        if report.has_critical_findings() {
            summary.push_str("⚠️ **CRITICAL SECURITY ISSUES DETECTED**\n\n");
            summary.push_str(&format!(
                "This security audit has identified {} critical security finding(s) that require immediate attention. ",
                report.summary.critical_count
            ));
        } else if report.has_high_or_critical() {
            summary.push_str("⚠️ **HIGH SEVERITY ISSUES DETECTED**\n\n");
            summary.push_str(&format!(
                "This security audit has identified {} high severity finding(s) that should be addressed promptly. ",
                report.summary.high_count
            ));
        } else if report.summary.total_count > 0 {
            summary.push_str("✅ **SECURITY STATUS: ACCEPTABLE WITH RECOMMENDATIONS**\n\n");
            summary.push_str("This security audit found no critical or high severity issues. ");
            summary.push_str(&format!(
                "{} low/medium severity finding(s) should be addressed as part of regular maintenance. ",
                report.summary.medium_count + report.summary.low_count
            ));
        } else {
            summary.push_str("✅ **SECURITY STATUS: EXCELLENT**\n\n");
            summary.push_str("This security audit found no security issues. ");
        }

        summary.push_str(&format!(
            "A total of {} security checks were performed across {} categories.",
            report.summary.checks_performed,
            report.findings_by_category.len()
        ));

        summary
    }

    /// Generates a summary table for Markdown
    fn generate_summary_table(&self, report: &SecurityReport) -> String {
        let mut table = String::new();
        
        table.push_str("| Severity | Count | Status |\n");
        table.push_str("|----------|-------|--------|\n");
        table.push_str(&format!("| 🔴 Critical | {} | {} |\n", 
            report.summary.critical_count,
            if report.summary.critical_count > 0 { "Action Required" } else { "✓ Clear" }
        ));
        table.push_str(&format!("| 🟠 High | {} | {} |\n",
            report.summary.high_count,
            if report.summary.high_count > 0 { "Action Recommended" } else { "✓ Clear" }
        ));
        table.push_str(&format!("| 🟡 Medium | {} | {} |\n",
            report.summary.medium_count,
            if report.summary.medium_count > 0 { "Planned Remediation" } else { "✓ Clear" }
        ));
        table.push_str(&format!("| 🟢 Low | {} | {} |\n",
            report.summary.low_count,
            if report.summary.low_count > 0 { "Maintenance Item" } else { "✓ Clear" }
        ));
        table.push_str(&format!("| ℹ️ Info | {} | {} |\n",
            report.summary.info_count,
            "Informational"
        ));
        table.push_str(&format!("| **Total** | **{}** | |\n", report.summary.total_count));

        table
    }

    /// Generates findings grouped by severity
    fn generate_findings_by_severity(&self, report: &SecurityReport) -> String {
        let mut output = String::new();

        for level in [SecurityLevel::Critical, SecurityLevel::High, SecurityLevel::Medium, SecurityLevel::Low] {
            let findings: Vec<_> = report.findings.iter()
                .filter(|f| f.level == level)
                .collect();

            if !findings.is_empty() {
                output.push_str(&format!("### {} Severity ({} findings)\n\n", level, findings.len()));
                for finding in findings.iter().take(5) {
                    output.push_str(&format!("- **{}** - {}\n", finding.id, finding.title));
                }
                if findings.len() > 5 {
                    output.push_str(&format!("- ... and {} more\n", findings.len() - 5));
                }
                output.push('\n');
            }
        }

        output
    }

    /// Generates findings grouped by category
    fn generate_findings_by_category(&self, report: &SecurityReport) -> String {
        let mut output = String::new();

        let categories = [
            SecurityCategory::Authentication,
            SecurityCategory::Authorization,
            SecurityCategory::InputValidation,
            SecurityCategory::DataProtection,
            SecurityCategory::EventSystem,
            SecurityCategory::Infrastructure,
            SecurityCategory::TenantSecurity,
        ];

        for category in &categories {
            let findings: Vec<_> = report.findings.iter()
                .filter(|f| f.category == *category)
                .collect();

            if !findings.is_empty() {
                let highest = findings.iter()
                    .map(|f| f.level)
                    .max()
                    .unwrap_or(SecurityLevel::Info);

                output.push_str(&format!("### {:?} ({} findings, highest: {})\n\n", 
                    category, findings.len(), highest));
                
                for finding in findings.iter().take(3) {
                    output.push_str(&format!("- {}: {}\n", finding.id, finding.title));
                }
                if findings.len() > 3 {
                    output.push_str(&format!("- ... and {} more\n", findings.len() - 3));
                }
                output.push('\n');
            }
        }

        output
    }

    /// Generates detailed finding section
    fn generate_finding_detail(&self, finding: &SecurityFinding) -> String {
        let mut detail = String::new();

        detail.push_str(&format!("### {}: {}\n\n", finding.id, finding.title));
        detail.push_str(&format!("**Severity:** {}\n\n", finding.level));
        detail.push_str(&format!("**Category:** {:?}\n\n", finding.category));
        detail.push_str(&format!("**Description:** {}\n\n", finding.description));

        if !finding.affected_resources.is_empty() {
            detail.push_str("**Affected Resources:**\n");
            for resource in &finding.affected_resources {
                detail.push_str(&format!("- {}\n", resource));
            }
            detail.push('\n');
        }

        if let Some(cvss) = finding.cvss_score {
            detail.push_str(&format!("**CVSS Score:** {:.1}/10\n\n", cvss));
        }

        if let Some(cve) = &finding.cve_id {
            detail.push_str(&format!("**CVE:** {}\n\n", cve));
        }

        if let Some(cwe) = &finding.cwe_id {
            detail.push_str(&format!("**CWE:** {}\n\n", cwe));
        }

        if self.include_evidence && !finding.evidence.is_empty() {
            detail.push_str("**Evidence:**\n\n");
            for evidence in &finding.evidence {
                detail.push_str(&format!("- *{}*: {}\n", evidence.evidence_type, evidence.description));
                if !evidence.data.is_empty() {
                    detail.push_str(&format!("  ```\n  {}\n  ```\n", evidence.data));
                }
            }
            detail.push('\n');
        }

        if self.include_remediation {
            if let Some(remediation) = &finding.remediation {
                detail.push_str("**Remediation:**\n\n");
                detail.push_str(&format!("*{}* - {}\n\n", remediation.id, remediation.title));
                detail.push_str(&format!("{}\n\n", remediation.description));
                if !remediation.steps.is_empty() {
                    detail.push_str("Steps:\n");
                    for (i, step) in remediation.steps.iter().enumerate() {
                        detail.push_str(&format!("{}. {}\n", i + 1, step));
                    }
                    detail.push('\n');
                }
            }
        }

        detail.push_str(&format!("*Discovered: {}*\n", finding.discovered_at.format("%Y-%m-%d %H:%M:%S UTC")));

        detail
    }

    /// Generates compliance summary
    fn generate_compliance_summary(&self, report: &SecurityReport) -> String {
        let mut summary = String::new();

        for (framework, status) in &report.compliance_summary {
            summary.push_str(&format!("### {}\n\n", framework));
            summary.push_str(&format!("- Compliance: {:.1}%\n", status.compliance_percentage));
            summary.push_str(&format!("- Passing Controls: {}\n", status.passing_controls));
            summary.push_str(&format!("- Failing Controls: {}\n", status.failing_controls));
            summary.push('\n');
        }

        summary
    }

    /// Generates recommendation section
    fn generate_recommendation(&self, rec: &super::Remediation) -> String {
        let mut output = String::new();

        output.push_str(&format!("### {}: {}\n\n", rec.id, rec.title));
        output.push_str(&format!("**Priority:** {}\n\n", rec.priority));
        output.push_str(&format!("**Estimated Effort:** {} hours\n\n", rec.estimated_effort_hours));
        output.push_str(&format!("{}\n\n", rec.description));

        if !rec.steps.is_empty() {
            output.push_str("**Implementation Steps:**\n");
            for (i, step) in rec.steps.iter().enumerate() {
                output.push_str(&format!("{}. {}\n", i + 1, step));
            }
            output.push('\n');
        }

        if !rec.resources.is_empty() {
            output.push_str("**Resources:**\n");
            for resource in &rec.resources {
                output.push_str(&format!("- {}\n", resource));
            }
            output.push('\n');
        }

        output
    }

    /// Returns a description of the risk score
    fn risk_score_description(&self, score: u32) -> String {
        match score {
            0 => "✅ **Excellent**: No significant security risks identified.".to_string(),
            1..=20 => "✅ **Good**: Low security risk. Minor improvements recommended.".to_string(),
            21..=40 => "⚠️ **Moderate**: Medium security risk. Address findings in next maintenance window.".to_string(),
            41..=60 => "⚠️ **Elevated**: Elevated security risk. Prioritize remediation efforts.".to_string(),
            61..=80 => "🔴 **High**: High security risk. Immediate action recommended.".to_string(),
            81..=100 => "🔴 **Critical**: Critical security risk. Immediate action required.".to_string(),
            _ => "Unknown risk score.".to_string(),
        }
    }

    /// Generates HTML CSS styles
    fn generate_html_styles(&self) -> String {
        r#"
<style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; line-height: 1.6; color: #333; background: #f5f5f5; }
    .container { max-width: 1200px; margin: 0 auto; padding: 20px; }
    header { background: white; padding: 30px; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
    h1 { color: #1a1a1a; margin-bottom: 10px; }
    h2 { color: #333; margin: 30px 0 15px; padding-bottom: 10px; border-bottom: 2px solid #e0e0e0; }
    .meta { color: #666; font-size: 14px; }
    .risk-card { background: white; padding: 30px; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); text-align: center; }
    .risk-score { font-size: 72px; font-weight: bold; margin: 20px 0; }
    .risk-score.critical { color: #dc2626; }
    .risk-score.high { color: #ea580c; }
    .risk-score.medium { color: #ca8a04; }
    .risk-score.low { color: #16a34a; }
    .summary-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px; margin-bottom: 20px; }
    .summary-card { background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
    .summary-card h3 { font-size: 14px; color: #666; margin-bottom: 10px; }
    .summary-card .count { font-size: 36px; font-weight: bold; }
    .count.critical { color: #dc2626; }
    .count.high { color: #ea580c; }
    .count.medium { color: #ca8a04; }
    .count.low { color: #16a34a; }
    .count.info { color: #2563eb; }
    table { width: 100%; background: white; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 4px rgba(0,0,0,0.1); margin-bottom: 20px; }
    th, td { padding: 12px 15px; text-align: left; }
    th { background: #f8f9fa; font-weight: 600; color: #666; }
    tr:nth-child(even) { background: #f8f9fa; }
    .severity { font-weight: 600; padding: 4px 12px; border-radius: 4px; font-size: 12px; text-transform: uppercase; }
    .severity.critical { background: #fef2f2; color: #dc2626; }
    .severity.high { background: #fff7ed; color: #ea580c; }
    .severity.medium { background: #fefce8; color: #ca8a04; }
    .severity.low { background: #f0fdf4; color: #16a34a; }
    .finding-card { background: white; padding: 20px; border-radius: 8px; margin-bottom: 15px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); border-left: 4px solid #e0e0e0; }
    .finding-card.critical { border-left-color: #dc2626; }
    .finding-card.high { border-left-color: #ea580c; }
    .finding-card.medium { border-left-color: #ca8a04; }
    .finding-card.low { border-left-color: #16a34a; }
    .finding-card h3 { margin-bottom: 10px; }
    .finding-meta { color: #666; font-size: 14px; margin-bottom: 10px; }
    .recommendation-card { background: #f8f9fa; padding: 20px; border-radius: 8px; margin-bottom: 15px; border: 1px solid #e0e0e0; }
    code { background: #f4f4f4; padding: 2px 6px; border-radius: 3px; font-family: monospace; font-size: 14px; }
</style>
"#.to_string()
    }

    /// Generates risk score card for HTML
    fn generate_risk_score_card(&self, report: &SecurityReport) -> String {
        let score_class = match report.summary.risk_score {
            0..=20 => "low",
            21..=40 => "medium",
            41..=60 => "medium",
            61..=80 => "high",
            _ => "critical",
        };

        format!(
            r#"<div class="risk-card">
                <h2>Overall Risk Score</h2>
                <div class="risk-score {}">{}/100</div>
                <p>{}</p>
            </div>"#,
            score_class,
            report.summary.risk_score,
            self.risk_score_description(report.summary.risk_score)
        )
    }

    /// Generates summary cards for HTML
    fn generate_summary_cards(&self, report: &SecurityReport) -> String {
        let mut html = String::from("<div class=\"summary-grid\">");

        html.push_str(&format!(
            r#"<div class="summary-card">
                <h3>Critical</h3>
                <div class="count critical">{}</div>
            </div>"#,
            report.summary.critical_count
        ));

        html.push_str(&format!(
            r#"<div class="summary-card">
                <h3>High</h3>
                <div class="count high">{}</div>
            </div>"#,
            report.summary.high_count
        ));

        html.push_str(&format!(
            r#"<div class="summary-card">
                <h3>Medium</h3>
                <div class="count medium">{}</div>
            </div>"#,
            report.summary.medium_count
        ));

        html.push_str(&format!(
            r#"<div class="summary-card">
                <h3>Low</h3>
                <div class="count low">{}</div>
            </div>"#,
            report.summary.low_count
        ));

        html.push_str(&format!(
            r#"<div class="summary-card">
                <h3>Total Findings</h3>
                <div class="count info">{}</div>
            </div>"#,
            report.summary.total_count
        ));

        html.push_str("</div>");
        html
    }

    /// Generates findings table for HTML
    fn generate_findings_table(&self, report: &SecurityReport) -> String {
        let mut html = String::from(r#"<table>
            <thead>
                <tr>
                    <th>ID</th>
                    <th>Severity</th>
                    <th>Category</th>
                    <th>Title</th>
                </tr>
            </thead>
            <tbody>"#);

        for finding in &report.findings {
            let severity_class = finding.level.to_string().to_lowercase();
            html.push_str(&format!(
                r#"<tr>
                    <td>{}</td>
                    <td><span class="severity {}">{}</span></td>
                    <td>{:?}</td>
                    <td>{}</td>
                </tr>"#,
                finding.id,
                severity_class,
                finding.level,
                finding.category,
                finding.title
            ));
        }

        html.push_str("</tbody></table>");
        html
    }

    /// Generates finding card for HTML
    fn generate_finding_card(&self, finding: &SecurityFinding) -> String {
        let severity_class = finding.level.to_string().to_lowercase();
        
        let mut html = format!(
            r#"<div class="finding-card {}">
                <h3>{}: {}</h3>
                <div class="finding-meta">
                    <span class="severity {}">{}</span> | {:?} | Discovered: {}
                </div>
                <p>{}</p>"#,
            severity_class,
            finding.id,
            finding.title,
            severity_class,
            finding.level,
            finding.category,
            finding.discovered_at.format("%Y-%m-%d %H:%M:%S UTC"),
            finding.description
        );

        if !finding.affected_resources.is_empty() {
            html.push_str("<p><strong>Affected Resources:</strong></p><ul>");
            for resource in &finding.affected_resources {
                html.push_str(&format!("<li>{}</li>", resource));
            }
            html.push_str("</ul>");
        }

        html.push_str("</div>");
        html
    }

    /// Generates recommendation card for HTML
    fn generate_recommendation_card(&self, rec: &super::Remediation) -> String {
        let mut html = format!(
            r#"<div class="recommendation-card">
                <h3>{}: {}</h3>
                <p><strong>Priority:</strong> {} | <strong>Effort:</strong> {} hours</p>
                <p>{}</p>"#,
            rec.id,
            rec.title,
            rec.priority,
            rec.estimated_effort_hours,
            rec.description
        );

        if !rec.steps.is_empty() {
            html.push_str("<ol>");
            for step in &rec.steps {
                html.push_str(&format!("<li>{}</li>", step));
            }
            html.push_str("</ol>");
        }

        html.push_str("</div>");
        html
    }
}

/// Report format types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Json,
    Markdown,
    Html,
    Sarif,
    Csv,
}

/// Report export options
#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub format: ReportFormat,
    pub include_evidence: bool,
    pub include_remediation: bool,
    pub min_severity: SecurityLevel,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ReportFormat::Json,
            include_evidence: true,
            include_remediation: true,
            min_severity: SecurityLevel::Low,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_report() -> SecurityReport {
        let mut report = SecurityReport::new();
        report.add_finding(
            SecurityFinding::new(SecurityCategory::Authentication, SecurityLevel::High, "Test Finding")
                .with_description("This is a test finding")
        );
        report.complete();
        report
    }

    #[test]
    fn test_generate_json() {
        let generator = SecurityReportGenerator::new();
        let report = create_test_report();
        let json = generator.generate_json(&report);
        assert!(json.is_ok());
        assert!(json.unwrap().contains("Test Finding"));
    }

    #[test]
    fn test_generate_markdown() {
        let generator = SecurityReportGenerator::new();
        let report = create_test_report();
        let md = generator.generate_markdown(&report);
        assert!(md.contains("Security Audit Report"));
        assert!(md.contains("Test Finding"));
        assert!(md.contains("High"));
    }

    #[test]
    fn test_generate_html() {
        let generator = SecurityReportGenerator::new();
        let report = create_test_report();
        let html = generator.generate_html(&report);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Test Finding"));
        assert!(html.contains("risk-score"));
    }

    #[test]
    fn test_risk_score_description() {
        let generator = SecurityReportGenerator::new();
        assert!(generator.risk_score_description(0).contains("Excellent"));
        assert!(generator.risk_score_description(50).contains("Elevated"));
        assert!(generator.risk_score_description(90).contains("Critical"));
    }

    #[test]
    fn test_summary_table_generation() {
        let generator = SecurityReportGenerator::new();
        let report = create_test_report();
        let table = generator.generate_summary_table(&report);
        assert!(table.contains("|"));
        assert!(table.contains("High"));
    }
}
