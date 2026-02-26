//! # RusToK Server Tasks
//!
//! Background tasks for maintenance and operations.
//! Run with: `cargo loco task --name <task_name>`

use loco_rs::task::Tasks;

mod cleanup;
mod rbac_backfill;

/// Register all available tasks
pub fn register(tasks: &mut Tasks) {
    tasks.register(cleanup::CleanupTask);
    tasks.register(rbac_backfill::RbacBackfillTask);
}
