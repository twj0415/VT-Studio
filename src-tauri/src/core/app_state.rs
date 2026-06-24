use crate::db::Database;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub struct AppState {
    database: Database,
    workspace_root: PathBuf,
}

#[allow(dead_code)]
impl AppState {
    pub fn new(database: Database, workspace_root: PathBuf) -> Self {
        Self {
            database,
            workspace_root,
        }
    }

    pub fn database(&self) -> &Database {
        &self.database
    }

    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }
}
