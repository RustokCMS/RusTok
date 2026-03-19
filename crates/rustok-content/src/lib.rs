use async_trait::async_trait;
use rustok_core::permissions::{Action, Permission, Resource};
use rustok_core::{MigrationSource, RusToKModule};
use sea_orm_migration::MigrationTrait;

pub mod dto;
pub mod entities;
pub mod error;
pub mod locale;
pub mod migrations;
pub mod services;
pub mod state_machine;

#[cfg(test)]
mod state_machine_proptest;

pub use dto::*;
pub use entities::{
    Body, Category, CategoryTranslation, Node, NodeTranslation, Tag, TagTranslation, Taggable,
};
pub use error::{ContentError, ContentResult};
pub use locale::{
    available_locales_from, resolve_by_locale, resolve_by_locale_with_fallback, ResolvedLocale,
    PLATFORM_FALLBACK_LOCALE,
};
pub use services::{
    CategoryService, ContentOrchestrationService, DemotePostToTopicInput, MergeTopicsInput,
    NodeService, OrchestrationResult, PromoteTopicToPostInput, SplitTopicInput, TagService,
};
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
            // Posts
            Permission::new(Resource::Posts, Action::Create),
            Permission::new(Resource::Posts, Action::Read),
            Permission::new(Resource::Posts, Action::Update),
            Permission::new(Resource::Posts, Action::Delete),
            Permission::new(Resource::Posts, Action::List),
            Permission::new(Resource::Posts, Action::Manage),
            // Nodes
            Permission::new(Resource::Nodes, Action::Create),
            Permission::new(Resource::Nodes, Action::Read),
            Permission::new(Resource::Nodes, Action::Update),
            Permission::new(Resource::Nodes, Action::Delete),
            Permission::new(Resource::Nodes, Action::List),
            Permission::new(Resource::Nodes, Action::Manage),
            // Media
            Permission::new(Resource::Media, Action::Create),
            Permission::new(Resource::Media, Action::Read),
            Permission::new(Resource::Media, Action::Update),
            Permission::new(Resource::Media, Action::Delete),
            Permission::new(Resource::Media, Action::List),
            Permission::new(Resource::Media, Action::Manage),
            // Comments
            Permission::new(Resource::Comments, Action::Create),
            Permission::new(Resource::Comments, Action::Read),
            Permission::new(Resource::Comments, Action::Update),
            Permission::new(Resource::Comments, Action::Delete),
            Permission::new(Resource::Comments, Action::List),
            Permission::new(Resource::Comments, Action::Manage),
            // Categories
            Permission::new(Resource::Categories, Action::Create),
            Permission::new(Resource::Categories, Action::Read),
            Permission::new(Resource::Categories, Action::Update),
            Permission::new(Resource::Categories, Action::Delete),
            Permission::new(Resource::Categories, Action::List),
            Permission::new(Resource::Categories, Action::Manage),
            // Tags
            Permission::new(Resource::Tags, Action::Create),
            Permission::new(Resource::Tags, Action::Read),
            Permission::new(Resource::Tags, Action::Update),
            Permission::new(Resource::Tags, Action::Delete),
            Permission::new(Resource::Tags, Action::List),
            Permission::new(Resource::Tags, Action::Manage),
        ]
    }
}

impl MigrationSource for ContentModule {
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        migrations::migrations()
    }
}

#[cfg(test)]
mod contract_tests;
