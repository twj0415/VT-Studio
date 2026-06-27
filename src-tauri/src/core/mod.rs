//! Shared application primitives and orchestration contracts.
//!
//! Keep Tauri commands out of this module. Commands call services; services can
//! depend on core contracts when TODO-05 introduces the task pipeline.

pub mod app_state;
pub mod error;
pub mod event;
pub mod result;
