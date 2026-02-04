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
            }
        }
    }

    fn register_validation_helpers(engine: &mut Engine) {
        engine.register_fn("validate_email", |email: &str| -> bool {
            email.contains('@') && email.contains('.')
        });
    }

    fn register_db_services(_engine: &mut Engine) {}

    fn register_external_services(_engine: &mut Engine) {}
}
