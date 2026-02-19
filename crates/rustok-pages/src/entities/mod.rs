//! SeaORM entities for pages module
//!
//! The pages module uses `rustok-content` tables for storage.
//! This module re-exports the relevant entities for convenience.

// Re-export content entities that pages module uses
pub use rustok_content::entities::{Body, Node, NodeTranslation};

/// Page entity (re-export from content Node with page kind)
pub type Page = Node;

/// Block entity (re-export from content Node with block kind)
pub type Block = Node;

/// Menu entity (re-export from content Node with menu kind)
pub type Menu = Node;
