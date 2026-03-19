#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthzEngine {
    Casbin,
}

#[cfg(test)]
mod tests {
    use super::AuthzEngine;

    #[test]
    fn exposes_single_runtime_engine() {
        assert_eq!(AuthzEngine::Casbin, AuthzEngine::Casbin);
    }
}
