use async_trait::async_trait;
use rustok_core::{MigrationSource, RusToKModule};
use sea_orm_migration::MigrationTrait;

pub mod dto;
pub mod entities;
pub mod error;
pub mod services;

pub use dto::*;
pub use entities::{Body, Node, NodeTranslation};
pub use error::{ContentError, ContentResult};
pub use services::NodeService;

pub struct ContentModule;

#[async_trait]
impl RusToKModule for ContentModule {
    fn slug(&self) -> &'static str {
        "content"
    }

    fn name(&self) -> &'static str {
        "Content"
    }

    fn description(&self) -> &'static str {
        "Core CMS Module (Nodes, Bodies, Categories)"
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

impl MigrationSource for ContentModule {
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        Vec::new() // Migrations are currently handled by the main app
    }
}
