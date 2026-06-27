//! Security boundary for paths, secrets, and export scanning.
//!
//! File access must go through StorageService and PathGuard once TODO-04 lands.
//! Secrets must stay out of DTOs, logs, config files, and export packages.

pub mod path_guard;
pub mod secret_guard;
