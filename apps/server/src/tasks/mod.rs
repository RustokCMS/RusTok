//! # RusToK Server Tasks
//!
//! Background tasks for maintenance and operations.
//! Run with: `cargo loco task --name <task_name>`

use loco_rs::{task::Tasks, Result};

mod cleanup;

/// Register all available tasks
pub fn register(tasks: &mut Tasks) -> Result<()> {
    // Maintenance tasks
    tasks.register(cleanup::CleanupTask)?;

    Ok(())
}
