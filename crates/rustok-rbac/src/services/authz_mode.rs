#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RbacAuthzMode {
    RelationOnly,
    DualRead,
}

impl RbacAuthzMode {
    pub fn parse(value: &str) -> Self {
        if value.trim().eq_ignore_ascii_case("dual_read") {
            return Self::DualRead;
        }

        Self::RelationOnly
    }

    pub fn from_env() -> Self {
        std::env::var("RUSTOK_RBAC_AUTHZ_MODE")
            .map(|raw| Self::parse(&raw))
            .unwrap_or(Self::RelationOnly)
    }

    pub fn is_dual_read(self) -> bool {
        self == Self::DualRead
    }
}

#[cfg(test)]
mod tests {
    use super::RbacAuthzMode;

    #[test]
    fn parse_dual_read_case_insensitive() {
        assert_eq!(RbacAuthzMode::parse("DUAL_READ"), RbacAuthzMode::DualRead);
    }

    #[test]
    fn parse_defaults_to_relation_only() {
        assert_eq!(RbacAuthzMode::parse("legacy"), RbacAuthzMode::RelationOnly);
    }
}
