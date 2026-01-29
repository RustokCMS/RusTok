//! Event-driven communication system for RusToK
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐     ┌───────────────┐     ┌─────────────────┐
//! │   Service   │────▶│   EventBus    │────▶│  EventDispatcher│
//! │  (publish)  │     │  (broadcast)  │     │   (handlers)    │
//! └─────────────┘     └───────────────┘     └─────────────────┘
//!                                                    │
//!                           ┌────────────────────────┼────────────┐
//!                           ▼                        ▼            ▼
//!                    ┌─────────────┐         ┌─────────────┐ ┌─────────┐
//!                    │  Indexer    │         │  Notifier   │ │  Audit  │
//!                    │  Handler    │         │  Handler    │ │ Handler │
//!                    └─────────────┘         └─────────────┘ └─────────┘
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use rustok_core::events::{EventBus, EventDispatcher, DomainEvent};
//!
//! let bus = EventBus::new(1024);
//! let mut dispatcher = EventDispatcher::new(bus.clone());
//! dispatcher.register(MyIndexerHandler::new());
//!
//! let running = dispatcher.start();
//! running
//!     .bus()
//!     .publish(tenant_id, Some(user_id), DomainEvent::NodeCreated { .. });
//! ```

pub mod bus;
pub mod handler;
pub mod types;

pub use bus::{EventBus, EventBusStats};
pub use handler::{
    DispatcherConfig, EventDispatcher, EventHandler, HandlerBuilder, HandlerResult,
    RunningDispatcher,
};
pub use types::{DomainEvent, EventEnvelope};
