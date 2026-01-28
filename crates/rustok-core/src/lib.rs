pub mod error;
pub mod auth;
pub mod id;
pub mod module;
pub mod types;
pub use error::{Error, Result};
pub use id::generate_id;
pub use module::RusToKModule;
pub use types::{UserRole, UserStatus};

pub mod prelude {
    pub use crate::error::{Error, Result};
    pub use crate::id::generate_id;
    pub use crate::types::{UserRole, UserStatus};
    pub use uuid::Uuid;
}
