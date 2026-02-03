use async_trait::async_trait;
use rustok_core::{MigrationSource, RusToKModule};
use sea_orm_migration::MigrationTrait;

pub mod dto;
pub mod entities;
pub mod error;
pub mod services;

pub use dto::{CreateThreadInput, ThreadListItem, ThreadResponse, UpdateThreadInput};
pub use services::ThreadService;

pub struct ForumModule;

#[async_trait]
impl RusToKModule for ForumModule {
    fn slug(&self) -> &'static str {
        "forum"
    }

    fn name(&self) -> &'static str {
        "Forum"
    }

    fn description(&self) -> &'static str {
        "Threads, replies, moderation, and forum workflows"
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

impl MigrationSource for ForumModule {
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        Vec::new()
    }
}
