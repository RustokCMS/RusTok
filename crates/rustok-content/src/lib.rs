use async_trait::async_trait;
use rustok_core::permissions::{Action, Permission, Resource};
use rustok_core::{MigrationSource, RusToKModule};
use sea_orm_migration::MigrationTrait;

pub mod dto;
pub mod entities;
pub mod error;
pub mod services;
pub mod state_machine;

pub use dto::*;
pub use entities::{Body, Node, NodeTranslation};
pub use error::{ContentError, ContentResult};
pub use services::NodeService;
pub use state_machine::{Archived, ContentNode, Draft, Published, ToContentStatus};

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

    fn permissions(&self) -> Vec<Permission> {
        vec![
            // Pages
            Permission::new(Resource::Pages, Action::Create),
            Permission::new(Resource::Pages, Action::Read),
            Permission::new(Resource::Pages, Action::Update),
            Permission::new(Resource::Pages, Action::Delete),
            Permission::new(Resource::Pages, Action::List),
            // Posts
            Permission::new(Resource::Posts, Action::Create),
            Permission::new(Resource::Posts, Action::Read),
            Permission::new(Resource::Posts, Action::Update),
            Permission::new(Resource::Posts, Action::Delete),
            Permission::new(Resource::Posts, Action::List),
            // Media
            Permission::new(Resource::Media, Action::Create),
            Permission::new(Resource::Media, Action::Read),
            Permission::new(Resource::Media, Action::Update),
            Permission::new(Resource::Media, Action::Delete),
            Permission::new(Resource::Media, Action::List),
            // Comments
            Permission::new(Resource::Comments, Action::Create),
            Permission::new(Resource::Comments, Action::Read),
            Permission::new(Resource::Comments, Action::Update),
            Permission::new(Resource::Comments, Action::Delete),
            Permission::new(Resource::Comments, Action::List),
        ]
    }
}

impl MigrationSource for ContentModule {
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        Vec::new() // Migrations are currently handled by the main app
    }
}
