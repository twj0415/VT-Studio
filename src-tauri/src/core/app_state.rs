use crate::db::Database;
use crate::services::keyring_service::KeyringService;
use crate::services::task_cancellation::ProcessHandleRegistry;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub struct AppState {
    database: Database,
    workspace_root: PathBuf,
    keyring_service: KeyringService,
    process_handle_registry: ProcessHandleRegistry,
}

#[allow(dead_code)]
impl AppState {
    pub fn new(
        database: Database,
        workspace_root: PathBuf,
        keyring_service: KeyringService,
    ) -> Self {
        Self {
            database,
            workspace_root,
            keyring_service,
            process_handle_registry: ProcessHandleRegistry::new(),
        }
    }

    pub fn database(&self) -> &Database {
        &self.database
    }

    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    pub fn keyring_service(&self) -> &KeyringService {
        &self.keyring_service
    }

    pub fn process_handle_registry(&self) -> &ProcessHandleRegistry {
        &self.process_handle_registry
    }
}
