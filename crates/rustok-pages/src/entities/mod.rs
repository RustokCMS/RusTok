//! SeaORM entities for pages module.

pub mod menu;
pub mod menu_item;
pub mod menu_item_translation;
pub mod menu_translation;
pub mod page;
pub mod page_block;
pub mod page_body;
pub mod page_translation;

pub use menu::Entity as Menu;
pub use menu_item::Entity as MenuItem;
pub use page::Entity as Page;
pub use page_block::Entity as Block;
