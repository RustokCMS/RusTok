//! # RusToK Server Tasks
//!
//! Background tasks for maintenance and operations.
//! Run with: `cargo loco task --name <task_name>`

use loco_rs::task::Tasks;

mod cleanup;
mod create_oauth_app;

/// Register all available tasks
pub fn register(tasks: &mut Tasks) {
    // Maintenance tasks
    tasks.register(cleanup::CleanupTask);
    tasks.register(create_oauth_app::CreateOAuthAppTask);
}
