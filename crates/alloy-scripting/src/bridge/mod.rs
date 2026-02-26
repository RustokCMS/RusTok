mod utils;

use crate::context::ExecutionPhase;
use rhai::Engine;

pub use utils::register_utils;

pub struct Bridge;

impl Bridge {
    pub fn register_for_phase(engine: &mut Engine, phase: ExecutionPhase) {
        register_utils(engine);

        match phase {
            ExecutionPhase::Before => {
                Self::register_validation_helpers(engine);
            }
            ExecutionPhase::After => {
                Self::register_db_services(engine);
            }
            ExecutionPhase::OnCommit => {
                Self::register_external_services(engine);
            }
            ExecutionPhase::Manual | ExecutionPhase::Scheduled => {
                Self::register_db_services(engine);
                Self::register_external_services(engine);
                Self::register_validation_helpers(engine);
            }
        }
    }

    fn register_validation_helpers(engine: &mut Engine) {
        engine.register_fn("validate_email", |email: &str| -> bool {
            let parts: Vec<&str> = email.splitn(2, '@').collect();
            if parts.len() != 2 {
                return false;
            }
            let local = parts[0];
            let domain = parts[1];
            !local.is_empty()
                && !domain.is_empty()
                && domain.contains('.')
                && !domain.starts_with('.')
                && !domain.ends_with('.')
                && domain.len() > 2
        });

        engine.register_fn("validate_required", |value: &str| -> bool {
            !value.trim().is_empty()
        });

        engine.register_fn("validate_min_length", |value: &str, min: i64| -> bool {
            value.len() as i64 >= min
        });

        engine.register_fn("validate_max_length", |value: &str, max: i64| -> bool {
            value.len() as i64 <= max
        });

        engine.register_fn("validate_range", |value: i64, min: i64, max: i64| -> bool {
            value >= min && value <= max
        });
    }

    fn register_db_services(_engine: &mut Engine) {
        // Placeholder for future DB service integration
        // Will be implemented when database access from scripts is needed
    }

    fn register_external_services(_engine: &mut Engine) {
        // Placeholder for future external service integration
        // Will be implemented when HTTP client, etc. are needed
    }
}
