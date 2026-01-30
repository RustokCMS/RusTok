use async_trait::async_trait;
use rustok_core::{MigrationSource, RusToKModule};
use sea_orm_migration::MigrationTrait;

pub mod dto;
pub mod entities;
pub mod services;

pub use dto::CreatePostInput;
pub use services::PostService;

pub struct BlogModule;

#[async_trait]
impl RusToKModule for BlogModule {
    fn slug(&self) -> &'static str {
        "blog"
    }

    fn name(&self) -> &'static str {
        "Blog"
    }

    fn description(&self) -> &'static str {
        "Posts, Pages, Comments"
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

impl MigrationSource for BlogModule {
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        Vec::new()
    }
}
