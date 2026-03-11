//! Event contracts crate for RusToK.
//!
//! This crate is introduced as the first extraction step from `rustok-core`.
//! Current implementation re-exports event primitives from `rustok-core`
//! to provide a stable dependency point for downstream modules.

pub use rustok_core::events::{DomainEvent, EventEnvelope};
pub use rustok_core::{DomainEvent as RootDomainEvent, EventEnvelope as RootEventEnvelope};


#[cfg(test)]
mod contract_tests;
