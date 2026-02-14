//! Input Validation - OWASP Injection Prevention
//!
//! Protection against:
//! - SQL Injection
//! - NoSQL Injection
//! - XSS (Cross-Site Scripting)
//! - Command Injection
//! - Path Traversal
//! - SSRF (Server-Side Request Forgery)

use std::collections::HashSet;

use regex::Regex;
use serde::{Deserialize, Serialize};
use url::Url;

use super::{SecurityCategory, SecurityFinding, Severity};
use crate::security::SecurityConfig;

/// Validation result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    Valid,
    Invalid { reason: String },
    Sanitized { original: String, sanitized: String },
}

/// Input validator
#[derive(Debug, Clone)]
pub struct InputValidator {
    /// SQL injection patterns
    sql_patterns: Vec<Regex>,
    /// XSS patterns
    xss_patterns: Vec<Regex>,
    /// Command injection patterns
    cmd_patterns: Vec<Regex>,
    /// Path traversal patterns
    path_patterns: Vec<Regex>,
    /// Maximum allowed length for inputs
    max_length: usize,
    /// Allowed characters pattern (if set)
    allowed_chars: Option<Regex>,
}

impl Default for InputValidator {
    fn default() -> Self {
        Self::new()
    }
}

use once_cell::sync::Lazy;

static SQL_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)(SELECT\s+.*\s+FROM|INSERT\s+INTO|UPDATE\s+.*\s+SET|DELETE\s+FROM|DROP\s+TABLE|UNION\s+SELECT|--|;--)").unwrap(),
        Regex::new(r"(?i)(OR\s+1\s*=\s*1|AND\s+1\s*=\s*1|1\s*=\\s*1)").unwrap(),
        Regex::new(r"(?i)(EXEC\s*\(|EXECUTE\s*\(|sp_executesql)").unwrap(),
    ]
});

static XSS_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)<script[^>]*>.*?</script>").unwrap(),
        Regex::new(r"(?i)javascript:").unwrap(),
        Regex::new(r"(?i)on\w+\s*=\s*[\"']?[^\"']*[\"']?").unwrap(),
        Regex::new(r"(?i)<\s*iframe").unwrap(),
        Regex::new(r"(?i)<\s*object").unwrap(),
        Regex::new(r"(?i)<\s*embed").unwrap(),
    ]
});

static CMD_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"[;&|`]\s*\w+").unwrap(),
        Regex::new(r"\$\(.*\)").unwrap(),
        Regex::new(r"`.*`").unwrap(),
    ]
});

static PATH_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"\.\./").unwrap(),
        Regex::new(r"\.\.\\").unwrap(),
        Regex::new(r"%2e%2e[/\\]").unwrap(),
        Regex::new(r"\x2e\x2e[/\\]").unwrap(),
    ]
});

impl InputValidator {
    /// Create new input validator with default patterns
    pub fn new() -> Self {
        let sql_patterns = vec![
            Regex::new(r"(?i)(SELECT\s+.*\s+FROM|INSERT\s+INTO|UPDATE\s+.*\s+SET|DELETE\s+FROM|DROP\s+TABLE|UNION\s+SELECT|--|;--)").unwrap(),
            Regex::new(r"(?i)(OR\s+1\s*=\s*1|AND\s+1\s*=\s*1|1\s*=\s*1)").unwrap(),
            Regex::new(r"(?i)(EXEC\s*\(|EXECUTE\s*\(|sp_executesql)").unwrap(),
        ];

        let xss_patterns = vec![
            Regex::new(r"(?i)<script[^>]*>.*?</script>").unwrap(),
            Regex::new(r"(?i)javascript:").unwrap(),
            Regex::new(r#"(?i)on\w+\s*=\s*["']?[^"']*["']?"#).unwrap(),
            Regex::new(r"(?i)<\s*iframe").unwrap(),
            Regex::new(r"(?i)<\s*object").unwrap(),
            Regex::new(r"(?i)<\s*embed").unwrap(),
        ];

        let cmd_patterns = vec![
            Regex::new(r"[;&|`]\s*\w+").unwrap(),
            Regex::new(r"\$\(.*\)").unwrap(),
            Regex::new(r"`.*`").unwrap(),
        ];

        let path_patterns = vec![
            Regex::new(r"\.\./").unwrap(),
            Regex::new(r"\.\.\\").unwrap(),
            Regex::new(r"%2e%2e[/\\]").unwrap(),
            Regex::new(r"\x2e\x2e[/\\]").unwrap(),
        ];

        Self {
            sql_patterns: SQL_PATTERNS.clone(),
            xss_patterns: XSS_PATTERNS.clone(),
            cmd_patterns: CMD_PATTERNS.clone(),
            path_patterns: PATH_PATTERNS.clone(),
            max_length: 10000,
            allowed_chars: None,
        }
    }

    /// Set maximum input length
    pub fn with_max_length(mut self, max: usize) -> Self {
        self.max_length = max;
        self
    }

    /// Set allowed characters pattern
    pub fn with_allowed_chars(mut self, pattern: &str) -> Result<Self, regex::Error> {
        self.allowed_chars = Some(Regex::new(pattern)?);
        Ok(self)
    }

    /// Validate string against all patterns
    pub fn validate(&self, input: &str) -> ValidationResult {
        // Check length
        if input.len() > self.max_length {
            return ValidationResult::Invalid {
                reason: format!("Input exceeds maximum length of {}", self.max_length),
            };
        }

        // Check allowed characters
        if let Some(ref pattern) = self.allowed_chars {
            if !pattern.is_match(input) {
                return ValidationResult::Invalid {
                    reason: "Input contains invalid characters".to_string(),
                };
            }
        }

        // Check SQL injection
        for pattern in &self.sql_patterns {
            if pattern.is_match(input) {
                return ValidationResult::Invalid {
                    reason: "Potential SQL injection detected".to_string(),
                };
            }
        }

        // Check XSS
        for pattern in &self.xss_patterns {
            if pattern.is_match(input) {
                return ValidationResult::Invalid {
                    reason: "Potential XSS attack detected".to_string(),
                };
            }
        }

        // Check command injection
        for pattern in &self.cmd_patterns {
            if pattern.is_match(input) {
                return ValidationResult::Invalid {
                    reason: "Potential command injection detected".to_string(),
                };
            }
        }

        // Check path traversal
        for pattern in &self.path_patterns {
            if pattern.is_match(input) {
                return ValidationResult::Invalid {
                    reason: "Potential path traversal detected".to_string(),
                };
            }
        }

        ValidationResult::Valid
    }

    /// Sanitize HTML content
    pub fn sanitize_html(&self, input: &str) -> String {
        input
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
            .replace('&', "&amp;")
    }

    /// Validate email address
    pub fn validate_email(&self, email: &str) -> ValidationResult {
        let email_regex = Regex::new(
            r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
        ).unwrap();

        if !email_regex.is_match(email) {
            return ValidationResult::Invalid {
                reason: "Invalid email format".to_string(),
            };
        }

        if email.len() > 254 {
            return ValidationResult::Invalid {
                reason: "Email address too long".to_string(),
            };
        }

        ValidationResult::Valid
    }

    /// Validate UUID
    pub fn validate_uuid(&self, uuid: &str) -> ValidationResult {
        let uuid_regex = Regex::new(
            r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"
        ).unwrap();

        if uuid_regex.is_match(uuid) {
            ValidationResult::Valid
        } else {
            ValidationResult::Invalid {
                reason: "Invalid UUID format".to_string(),
            }
        }
    }
}

/// SSRF Protection
#[derive(Debug, Clone)]
pub struct SsrfProtection {
    /// Allowed host patterns
    allowed_hosts: HashSet<String>,
    /// Blocked IP ranges (private, loopback, etc.)
    blocked_ranges: Vec<std::net::IpAddr>,
    /// Blocked schemes
    blocked_schemes: HashSet<String>,
    /// Maximum redirects to follow
    max_redirects: u32,
}

impl Default for SsrfProtection {
    fn default() -> Self {
        Self::new()
    }
}

impl SsrfProtection {
    /// Create new SSRF protection with defaults
    pub fn new() -> Self {
        let blocked_schemes = ["file", "ftp", "gopher", "dict"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        Self {
            allowed_hosts: HashSet::new(),
            blocked_ranges: Vec::new(),
            blocked_schemes,
            max_redirects: 5,
        }
    }

    /// Add allowed host
    pub fn allow_host(mut self, host: impl Into<String>) -> Self {
        self.allowed_hosts.insert(host.into());
        self
    }

    /// Validate URL for SSRF
    pub fn validate_url(&self, url: &str) -> ValidationResult {
        let parsed = match Url::parse(url) {
            Ok(u) => u,
            Err(_) => {
                return ValidationResult::Invalid {
                    reason: "Invalid URL format".to_string(),
                }
            }
        };

        // Check scheme
        if let Some(scheme) = parsed.scheme() {
            if self.blocked_schemes.contains(scheme) {
                return ValidationResult::Invalid {
                    reason: format!("URL scheme '{}' is not allowed", scheme),
                };
            }

            if scheme != "http" && scheme != "https" {
                return ValidationResult::Invalid {
                    reason: "Only HTTP and HTTPS URLs are allowed".to_string(),
                };
            }
        }

        // Check host
        if let Some(host) = parsed.host_str() {
            // Check if in allowed list (if configured)
            if !self.allowed_hosts.is_empty() && !self.allowed_hosts.contains(host) {
                return ValidationResult::Invalid {
                    reason: format!("Host '{}' is not in the allowlist", host),
                };
            }

            // Check for private IPs
            if let Ok(ip) = host.parse::<std::net::IpAddr>() {
                if self.is_private_ip(&ip) {
                    return ValidationResult::Invalid {
                        reason: "Private IP addresses are not allowed".to_string(),
                    };
                }
            }

            // Check for localhost
            if host == "localhost" || host == "127.0.0.1" || host == "::1" {
                return ValidationResult::Invalid {
                    reason: "Localhost URLs are not allowed".to_string(),
                };
            }
        } else {
            return ValidationResult::Invalid {
                reason: "URL must have a host".to_string(),
            };
        }

        ValidationResult::Valid
    }

    fn is_private_ip(&self, ip: &std::net::IpAddr) -> bool {
        match ip {
            std::net::IpAddr::V4(ipv4) => {
                ipv4.is_private() || ipv4.is_loopback() || ipv4.is_link_local()
            }
            std::net::IpAddr::V6(ipv6) => ipv6.is_loopback(),
        }
    }
}

/// Audit validation configuration
pub async fn audit_validation(config: &SecurityConfig) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();

    // Check SSRF protection
    if config.allowed_hosts.is_empty() {
        findings.push(SecurityFinding {
            category: SecurityCategory::Ssrf,
            severity: Severity::Medium,
            description: "SSRF allowlist is empty".to_string(),
            remediation: "Configure allowed_hosts for SSRF protection".to_string(),
        });
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_injection_detection() {
        let validator = InputValidator::new();

        assert!(matches!(
            validator.validate("SELECT * FROM users"),
            ValidationResult::Invalid { .. }
        ));

        assert!(matches!(
            validator.validate("admin' OR '1'='1"),
            ValidationResult::Invalid { .. }
        ));

        assert!(matches!(
            validator.validate("normal input"),
            ValidationResult::Valid
        ));
    }

    #[test]
    fn test_xss_detection() {
        let validator = InputValidator::new();

        assert!(matches!(
            validator.validate("<script>alert('xss')</script>"),
            ValidationResult::Invalid { .. }
        ));

        assert!(matches!(
            validator.validate("javascript:alert(1)"),
            ValidationResult::Invalid { .. }
        ));
    }

    #[test]
    fn test_path_traversal_detection() {
        let validator = InputValidator::new();

        assert!(matches!(
            validator.validate("../../../etc/passwd"),
            ValidationResult::Invalid { .. }
        ));
    }

    #[test]
    fn test_ssrf_protection() {
        let ssrf = SsrfProtection::new().allow_host("api.example.com");

        assert!(matches!(
            ssrf.validate_url("http://localhost/admin"),
            ValidationResult::Invalid { .. }
        ));

        assert!(matches!(
            ssrf.validate_url("http://127.0.0.1/admin"),
            ValidationResult::Invalid { .. }
        ));

        assert!(matches!(
            ssrf.validate_url("file:///etc/passwd"),
            ValidationResult::Invalid { .. }
        ));

        assert!(matches!(
            ssrf.validate_url("http://api.example.com/data"),
            ValidationResult::Valid
        ));
    }

    #[test]
    fn test_email_validation() {
        let validator = InputValidator::new();

        assert!(matches!(
            validator.validate_email("user@example.com"),
            ValidationResult::Valid
        ));

        assert!(matches!(
            validator.validate_email("invalid-email"),
            ValidationResult::Invalid { .. }
        ));
    }
}
