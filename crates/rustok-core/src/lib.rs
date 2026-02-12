pub mod auth;
pub mod cache;
pub mod circuit_breaker;
pub mod context;
pub mod error;
pub mod events;
pub mod i18n;
pub mod id;
pub mod migrations;
pub mod module;
pub mod permissions;
pub mod rbac;
pub mod registry;
pub mod resilience;
pub mod scripting;
pub mod security;
pub mod state_machine;
pub mod tenant_validation;
pub mod tracing;
pub mod types;
pub use auth::{
    AuthError, AuthService, AuthTokens, RegisterInput, User, UserRepository, UsersMigration,
};
#[cfg(feature = "redis-cache")]
pub use cache::RedisCacheBackend;
pub use cache::{CacheStats, InMemoryCacheBackend};
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError};
pub use context::{AppContext, CacheBackend, SearchBackend};
pub use error::{
    Error, ErrorContext, ErrorKind, ErrorResponse, FieldError, Result, RichError,
    ValidationErrorBuilder,
};
pub use events::{
    event_schema, DispatcherConfig, DomainEvent, EventBus, EventBusStats, EventDispatcher,
    EventEnvelope, EventHandler, EventSchema, EventTransport, FieldSchema, HandlerBuilder,
    HandlerResult, MemoryTransport, ReliabilityLevel, RunningDispatcher, EVENT_SCHEMAS,
};
pub use i18n::{extract_locale_from_header, translate, Locale};
pub use id::generate_id;
pub use migrations::ModuleMigration;
pub use module::{EventListener, MigrationSource};
pub use module::{HealthStatus, ModuleContext, RusToKModule};
pub use permissions::{Action, Permission, Resource};
pub use rbac::{PermissionScope, Rbac, SecurityContext};
pub use registry::ModuleRegistry;
pub use resilience::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError, CircuitState, RetryPolicy,
    RetryStrategy,
};
pub use scripting::ScriptingContext;
pub use security::{
    AuditContext, AuditScope, BatchSecurityAuditor, ComponentAuditResult, DataRetentionPolicy,
    PasswordPolicy, RateLimitPolicy, Remediation, SecurityAuditor, SecurityCategory,
    SecurityConfig, SecurityMonitor, SecurityPolicies, SecurityReportGenerator, SessionPolicy,
};
pub use types::{UserRole, UserStatus};

pub mod prelude {
    pub use crate::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError};
    pub use crate::error::{Error, Result};
    pub use crate::events::{
        event_schema, DispatcherConfig, DomainEvent, EventBus, EventBusStats, EventDispatcher,
        EventEnvelope, EventHandler, EventSchema, EventTransport, FieldSchema, HandlerBuilder,
        HandlerResult, MemoryTransport, ReliabilityLevel, RunningDispatcher, EVENT_SCHEMAS,
    };
    pub use crate::id::generate_id;
    pub use crate::module::HealthStatus;
    pub use crate::permissions::{Action, Permission, Resource};
    pub use crate::rbac::{PermissionScope, Rbac, SecurityContext};
    pub use crate::security::{
        SecurityAuditor, SecurityCategory, SecurityConfig, SecurityFinding, SecurityLevel,
        SecurityReport, SecurityStatus,
    };
    pub use crate::types::{UserRole, UserStatus};
    #[cfg(feature = "redis-cache")]
    pub use crate::RedisCacheBackend;
    pub use crate::{AppContext, CacheBackend, CacheStats, InMemoryCacheBackend, SearchBackend};
    pub use uuid::Uuid;
}
