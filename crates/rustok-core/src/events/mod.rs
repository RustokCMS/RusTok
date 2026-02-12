pub mod backpressure;
mod bus;
mod handler;
mod memory;
mod schema;
mod transport;
mod types;
pub mod validation;

pub use backpressure::{
    BackpressureConfig, BackpressureController, BackpressureError, BackpressureMetrics,
    BackpressureState,
};
pub use bus::{EventBus, EventBusStats};
pub use handler::{
    DispatcherConfig, EventDispatcher, EventHandler, HandlerBuilder, HandlerResult,
    RunningDispatcher,
};
pub use memory::MemoryTransport;
pub use schema::{event_schema, EventSchema, FieldSchema, EVENT_SCHEMAS};
pub use transport::{EventTransport, ReliabilityLevel};
pub use types::{DomainEvent, EventEnvelope};
pub use validation::{EventValidationError, ValidateEvent};
