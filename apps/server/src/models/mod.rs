//! Database models for the server application

pub mod build;
pub mod release;

pub use build::Entity as Build;
pub use release::Entity as Release;
