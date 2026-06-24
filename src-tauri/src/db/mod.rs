//! Repository and migration boundary.
//!
//! Repositories own database persistence only. They must not call providers,
//! Tauri commands, FFmpeg, or UI-facing services directly.

use rusqlite::{Connection, TransactionBehavior};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub mod project_repository;
pub mod provider_repository;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Database {
    path: PathBuf,
    connection: Arc<Mutex<Connection>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AppliedMigration {
    pub version: i64,
    pub name: String,
    pub checksum: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum DbError {
    Io(std::io::Error),
    Sql(rusqlite::Error),
    MigrationChanged {
        version: i64,
        name: &'static str,
        expected_checksum: String,
        actual_checksum: String,
    },
    LockPoisoned,
}

#[derive(Debug, Clone, Copy)]
struct Migration {
    version: i64,
    name: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "repository_metadata",
        sql: r#"
        CREATE TABLE IF NOT EXISTS repository_meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        INSERT INTO repository_meta (key, value, updated_at)
        VALUES ('authority', 'sqlite', CURRENT_TIMESTAMP)
        ON CONFLICT(key) DO UPDATE SET
            value = excluded.value,
            updated_at = CURRENT_TIMESTAMP;
    "#,
    },
    Migration {
        version: 2,
        name: "core_tables_v1",
        sql: r#"
        CREATE TABLE IF NOT EXISTS projects (
            project_id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            workflow_type TEXT NOT NULL,
            input_type TEXT NOT NULL,
            input_process_mode TEXT NOT NULL,
            input_options_json TEXT NOT NULL DEFAULT '{}',
            source_text TEXT,
            source_text_path TEXT,
            aspect_ratio TEXT NOT NULL,
            target_scene_count INTEGER NOT NULL CHECK (target_scene_count > 0),
            segment_duration_seconds REAL NOT NULL CHECK (segment_duration_seconds > 0),
            style_prompt TEXT,
            tone TEXT,
            content_language TEXT NOT NULL,
            lifecycle TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS project_bibles (
            project_id TEXT PRIMARY KEY,
            summary TEXT NOT NULL DEFAULT '',
            data_json TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS style_bibles (
            style_bible_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            name TEXT NOT NULL,
            data_json TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS character_bibles (
            character_bible_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            name TEXT NOT NULL,
            data_json TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS location_bibles (
            location_bible_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            name TEXT NOT NULL,
            data_json TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS storyboard_items (
            item_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            item_index INTEGER NOT NULL,
            source_text TEXT NOT NULL DEFAULT '',
            narration_text TEXT NOT NULL DEFAULT '',
            visual_goal TEXT NOT NULL DEFAULT '',
            visual_description TEXT NOT NULL DEFAULT '',
            characters_json TEXT NOT NULL DEFAULT '[]',
            character_ids_json TEXT NOT NULL DEFAULT '[]',
            location_id TEXT,
            scene_description TEXT NOT NULL DEFAULT '',
            image_prompt TEXT NOT NULL DEFAULT '',
            negative_prompt TEXT NOT NULL DEFAULT '',
            video_prompt TEXT NOT NULL DEFAULT '',
            duration_seconds REAL NOT NULL CHECK (duration_seconds > 0),
            selected_image_id TEXT,
            selected_video_segment_id TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            lock_flags_json TEXT NOT NULL DEFAULT '{}',
            shot_size TEXT,
            camera_motion TEXT,
            composition TEXT,
            pace TEXT,
            transition_type TEXT,
            image_status TEXT NOT NULL DEFAULT 'pending',
            audio_status TEXT NOT NULL DEFAULT 'pending',
            video_status TEXT NOT NULL DEFAULT 'pending',
            subtitle_status TEXT NOT NULL DEFAULT 'pending',
            render_status TEXT NOT NULL DEFAULT 'pending',
            segment_status TEXT NOT NULL DEFAULT 'pending',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS image_candidates (
            image_id TEXT PRIMARY KEY,
            item_id TEXT NOT NULL,
            image_path TEXT NOT NULL,
            prompt TEXT NOT NULL,
            negative_prompt TEXT NOT NULL DEFAULT '',
            model TEXT NOT NULL,
            provider_model_id TEXT NOT NULL,
            workflow_preset_id TEXT,
            status TEXT NOT NULL,
            selected INTEGER NOT NULL DEFAULT 0 CHECK (selected IN (0, 1)),
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            derived_from_image_id TEXT,
            generation_context_snapshot_json TEXT NOT NULL DEFAULT '{}',
            FOREIGN KEY(item_id) REFERENCES storyboard_items(item_id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS video_segments (
            segment_id TEXT PRIMARY KEY,
            item_id TEXT NOT NULL,
            input_image_id TEXT NOT NULL,
            video_path TEXT NOT NULL,
            video_prompt TEXT NOT NULL,
            duration_seconds REAL NOT NULL CHECK (duration_seconds > 0),
            model TEXT NOT NULL,
            provider_model_id TEXT NOT NULL,
            workflow_preset_id TEXT,
            status TEXT NOT NULL,
            selected INTEGER NOT NULL DEFAULT 0 CHECK (selected IN (0, 1)),
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            generation_context_snapshot_json TEXT NOT NULL DEFAULT '{}',
            FOREIGN KEY(item_id) REFERENCES storyboard_items(item_id) ON DELETE CASCADE,
            FOREIGN KEY(input_image_id) REFERENCES image_candidates(image_id)
        );

        CREATE TABLE IF NOT EXISTS composition_tasks (
            task_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            segment_ids_json TEXT NOT NULL DEFAULT '[]',
            output_path TEXT NOT NULL,
            status TEXT NOT NULL,
            progress INTEGER NOT NULL DEFAULT 0 CHECK (progress >= 0 AND progress <= 100),
            error_json TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS assets (
            asset_id TEXT PRIMARY KEY,
            kind TEXT NOT NULL,
            relative_path TEXT NOT NULL,
            source_kind TEXT NOT NULL,
            mime_type TEXT,
            size_bytes INTEGER,
            checksum TEXT,
            is_builtin INTEGER NOT NULL DEFAULT 0 CHECK (is_builtin IN (0, 1)),
            lifecycle TEXT NOT NULL DEFAULT 'active',
            metadata_json TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(relative_path)
        );

        CREATE TABLE IF NOT EXISTS asset_references (
            reference_id TEXT PRIMARY KEY,
            asset_id TEXT NOT NULL,
            owner_kind TEXT NOT NULL,
            owner_id TEXT NOT NULL,
            usage_kind TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(asset_id) REFERENCES assets(asset_id) ON DELETE RESTRICT
        );

        CREATE TABLE IF NOT EXISTS tasks (
            task_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            task_kind TEXT NOT NULL,
            task_status TEXT NOT NULL,
            current_step TEXT,
            summary TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS task_steps (
            step_id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            step_name TEXT NOT NULL,
            status TEXT NOT NULL,
            output_json TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(task_id) REFERENCES tasks(task_id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS task_attempts (
            attempt_id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            step_id TEXT,
            attempt_index INTEGER NOT NULL,
            status TEXT NOT NULL,
            error_json TEXT,
            started_at TEXT,
            finished_at TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(task_id) REFERENCES tasks(task_id) ON DELETE CASCADE,
            FOREIGN KEY(step_id) REFERENCES task_steps(step_id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS artifacts (
            artifact_id TEXT PRIMARY KEY,
            task_id TEXT,
            step_id TEXT,
            project_id TEXT,
            kind TEXT NOT NULL,
            relative_path TEXT,
            data_json TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(task_id) REFERENCES tasks(task_id) ON DELETE SET NULL,
            FOREIGN KEY(step_id) REFERENCES task_steps(step_id) ON DELETE SET NULL,
            FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS providers (
            provider_id TEXT PRIMARY KEY,
            vendor TEXT NOT NULL,
            kind TEXT NOT NULL,
            display_name TEXT NOT NULL,
            auth_type TEXT NOT NULL,
            key_alias TEXT,
            base_url TEXT,
            status TEXT NOT NULL DEFAULT 'disabled',
            enabled INTEGER NOT NULL DEFAULT 0 CHECK (enabled IN (0, 1)),
            config_json TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS provider_models (
            model_id TEXT PRIMARY KEY,
            provider_id TEXT NOT NULL,
            provider_model_id TEXT NOT NULL,
            display_name TEXT NOT NULL,
            capability TEXT NOT NULL,
            config_json TEXT NOT NULL DEFAULT '{}',
            enabled INTEGER NOT NULL DEFAULT 0 CHECK (enabled IN (0, 1)),
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(provider_id) REFERENCES providers(provider_id) ON DELETE CASCADE,
            UNIQUE(provider_id, provider_model_id, capability)
        );

        CREATE TABLE IF NOT EXISTS workflow_presets (
            preset_id TEXT PRIMARY KEY,
            provider_id TEXT,
            model_id TEXT,
            name TEXT NOT NULL,
            kind TEXT NOT NULL,
            capability TEXT NOT NULL,
            config_json TEXT NOT NULL DEFAULT '{}',
            enabled INTEGER NOT NULL DEFAULT 0 CHECK (enabled IN (0, 1)),
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(provider_id) REFERENCES providers(provider_id) ON DELETE SET NULL,
            FOREIGN KEY(model_id) REFERENCES provider_models(model_id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS app_configs (
            config_key TEXT PRIMARY KEY,
            config_json TEXT NOT NULL,
            schema_version INTEGER NOT NULL DEFAULT 1,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS prompt_templates (
            template_id TEXT PRIMARY KEY,
            kind TEXT NOT NULL,
            name TEXT NOT NULL,
            content TEXT NOT NULL,
            variables_json TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS templates (
            template_id TEXT PRIMARY KEY,
            kind TEXT NOT NULL,
            name TEXT NOT NULL,
            manifest_json TEXT NOT NULL DEFAULT '{}',
            relative_path TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS histories (
            history_id TEXT PRIMARY KEY,
            project_id TEXT,
            entity_kind TEXT NOT NULL,
            entity_id TEXT NOT NULL,
            action TEXT NOT NULL,
            snapshot_json TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(project_id) REFERENCES projects(project_id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_projects_lifecycle ON projects(lifecycle);
        CREATE INDEX IF NOT EXISTS idx_storyboard_items_project ON storyboard_items(project_id, item_index);
        CREATE INDEX IF NOT EXISTS idx_image_candidates_item ON image_candidates(item_id);
        CREATE INDEX IF NOT EXISTS idx_video_segments_item ON video_segments(item_id);
        CREATE INDEX IF NOT EXISTS idx_tasks_project ON tasks(project_id, updated_at);
        CREATE INDEX IF NOT EXISTS idx_assets_kind ON assets(kind);
        CREATE INDEX IF NOT EXISTS idx_asset_references_owner ON asset_references(owner_kind, owner_id);
        CREATE INDEX IF NOT EXISTS idx_provider_models_provider ON provider_models(provider_id);
        CREATE INDEX IF NOT EXISTS idx_workflow_presets_capability ON workflow_presets(capability);
        "#,
    },
];

#[allow(dead_code)]
impl Database {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DbError> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let connection = Connection::open(&path)?;
        configure_connection(&connection)?;
        run_migrations(&connection)?;

        Ok(Self {
            path,
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn with_connection<T>(
        &self,
        action: impl FnOnce(&Connection) -> Result<T, rusqlite::Error>,
    ) -> Result<T, DbError> {
        let connection = self.connection.lock().map_err(|_| DbError::LockPoisoned)?;
        action(&connection).map_err(DbError::from)
    }

    pub fn transaction<T>(
        &self,
        action: impl FnOnce(&rusqlite::Transaction<'_>) -> Result<T, rusqlite::Error>,
    ) -> Result<T, DbError> {
        let mut connection = self.connection.lock().map_err(|_| DbError::LockPoisoned)?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let result = action(&transaction)?;
        transaction.commit()?;
        Ok(result)
    }

    pub fn applied_migrations(&self) -> Result<Vec<AppliedMigration>, DbError> {
        self.with_connection(list_applied_migrations)
    }
}

#[allow(dead_code)]
pub trait Repository {
    fn database(&self) -> &Database;
}

fn configure_connection(connection: &Connection) -> Result<(), rusqlite::Error> {
    connection.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;
        PRAGMA busy_timeout = 5000;
        PRAGMA synchronous = NORMAL;
        "#,
    )
}

fn run_migrations(connection: &Connection) -> Result<(), DbError> {
    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            checksum TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )?;

    for migration in MIGRATIONS {
        apply_migration(connection, *migration)?;
    }

    Ok(())
}

fn apply_migration(connection: &Connection, migration: Migration) -> Result<(), DbError> {
    let checksum = migration_checksum(migration.sql);
    let existing = connection.query_row(
        "SELECT checksum FROM schema_migrations WHERE version = ?1",
        [migration.version],
        |row| row.get::<_, String>(0),
    );

    match existing {
        Ok(actual_checksum) if actual_checksum == checksum => return Ok(()),
        Ok(actual_checksum) => {
            return Err(DbError::MigrationChanged {
                version: migration.version,
                name: migration.name,
                expected_checksum: checksum,
                actual_checksum,
            });
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {}
        Err(error) => return Err(DbError::Sql(error)),
    }

    let transaction = connection.unchecked_transaction()?;
    transaction.execute_batch(migration.sql)?;
    transaction.execute(
        "INSERT INTO schema_migrations (version, name, checksum) VALUES (?1, ?2, ?3)",
        (migration.version, migration.name, checksum),
    )?;
    transaction.commit()?;

    Ok(())
}

fn list_applied_migrations(
    connection: &Connection,
) -> Result<Vec<AppliedMigration>, rusqlite::Error> {
    let mut statement = connection
        .prepare("SELECT version, name, checksum FROM schema_migrations ORDER BY version ASC")?;
    let rows = statement.query_map([], |row| {
        Ok(AppliedMigration {
            version: row.get(0)?,
            name: row.get(1)?,
            checksum: row.get(2)?,
        })
    })?;

    rows.collect()
}

fn migration_checksum(sql: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in sql.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

impl fmt::Display for DbError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "Database filesystem error: {error}"),
            Self::Sql(error) => write!(formatter, "SQLite error: {error}"),
            Self::MigrationChanged {
                version,
                name,
                expected_checksum,
                actual_checksum,
            } => write!(
                formatter,
                "Migration {version} ({name}) changed after it was applied. expected={expected_checksum}, actual={actual_checksum}"
            ),
            Self::LockPoisoned => write!(formatter, "Database connection lock is poisoned"),
        }
    }
}

impl std::error::Error for DbError {}

impl From<std::io::Error> for DbError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<rusqlite::Error> for DbError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sql(value)
    }
}

#[cfg(test)]
mod tests {
    use super::project_repository::ProjectRepository;
    use super::provider_repository::{
        ProviderModelRecord, ProviderRecord, ProviderRepository, WorkflowPresetRecord,
    };
    use super::Database;
    use crate::domain::project::CreateProjectRequest;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn database_open_creates_file_and_runs_migrations_once() {
        let path = test_database_path("migrations_once");
        let database = Database::open(&path).expect("database should open");
        let migrations = database
            .applied_migrations()
            .expect("migrations should be readable");

        assert_eq!(migrations.len(), 2);
        assert_eq!(migrations[0].version, 1);
        assert_eq!(migrations[0].name, "repository_metadata");
        assert!(database.path().exists());

        drop(database);

        let database = Database::open(&path).expect("database should reopen");
        assert_eq!(
            database
                .applied_migrations()
                .expect("migrations should still be readable")
                .len(),
            2
        );

        cleanup(path);
    }

    #[test]
    fn transaction_rolls_back_when_action_fails() {
        let path = test_database_path("transaction_rollback");
        let database = Database::open(&path).expect("database should open");

        let result: Result<(), _> = database.transaction(|transaction| {
            transaction.execute(
                "INSERT INTO repository_meta (key, value) VALUES (?1, ?2)",
                ("rollback_probe", "created"),
            )?;
            Err(rusqlite::Error::InvalidQuery)
        });

        assert!(result.is_err());

        let count: i64 = database
            .with_connection(|connection| {
                connection.query_row(
                    "SELECT COUNT(*) FROM repository_meta WHERE key = 'rollback_probe'",
                    [],
                    |row| row.get(0),
                )
            })
            .expect("count should be readable");

        assert_eq!(count, 0);

        cleanup(path);
    }

    #[test]
    fn core_tables_migration_creates_required_tables() {
        let path = test_database_path("core_tables");
        let database = Database::open(&path).expect("database should open");
        let expected_tables = [
            "projects",
            "project_bibles",
            "style_bibles",
            "character_bibles",
            "location_bibles",
            "storyboard_items",
            "image_candidates",
            "video_segments",
            "composition_tasks",
            "assets",
            "asset_references",
            "tasks",
            "task_steps",
            "task_attempts",
            "artifacts",
            "providers",
            "provider_models",
            "workflow_presets",
            "app_configs",
            "prompt_templates",
            "templates",
            "histories",
        ];

        for table in expected_tables {
            let exists: i64 = database
                .with_connection(|connection| {
                    connection.query_row(
                        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                        [table],
                        |row| row.get(0),
                    )
                })
                .expect("table check should run");
            assert_eq!(exists, 1, "{table} should exist");
        }

        cleanup(path);
    }

    #[test]
    fn project_repository_persists_project_after_reopen() {
        let path = test_database_path("project_repository");
        let database = Database::open(&path).expect("database should open");
        let detail = ProjectRepository::new(&database)
            .create_with_id(
                ProjectRepository::create_project_id(),
                CreateProjectRequest {
                    title: "落库测试".to_string(),
                    workflow_type: "image_to_video".to_string(),
                    input_type: "topic".to_string(),
                    topic: Some("为什么要早睡".to_string()),
                    source_text: None,
                    source_text_path: None,
                    content_language: "zh-CN".to_string(),
                    tone: None,
                    aspect_ratio: "9:16".to_string(),
                    target_scene_count: 8,
                    segment_duration_seconds: 4.0,
                    style_prompt: Some("干净真实".to_string()),
                    input_process_mode: "generate".to_string(),
                    input_options: Some(json!({ "splitMode": "paragraph" })),
                },
            )
            .expect("project should be created");

        let project_id = detail.project.project_id;
        drop(database);

        let database = Database::open(&path).expect("database should reopen");
        let detail = ProjectRepository::new(&database)
            .get_detail(&project_id)
            .expect("project should be readable")
            .expect("project should exist");

        assert_eq!(detail.project.title, "落库测试");
        assert_eq!(detail.project.workflow_type, "image_to_video");
        assert_eq!(detail.project_bible.summary, "第一阶段默认项目设定集");
        assert!(detail.project.latest_task.is_some());

        cleanup(path);
    }

    #[test]
    fn provider_repository_upserts_models_and_workflow_presets() {
        let path = test_database_path("provider_repository");
        let database = Database::open(&path).expect("database should open");
        let repository = ProviderRepository::new(&database);

        repository
            .upsert_provider(&ProviderRecord {
                provider_id: "provider_mock".to_string(),
                vendor: "mock".to_string(),
                kind: "image".to_string(),
                display_name: "Mock Provider".to_string(),
                auth_type: "none".to_string(),
                key_alias: None,
                base_url: None,
                status: "available".to_string(),
                enabled: true,
                config_json: json!({ "mock": true }),
            })
            .expect("provider should upsert");
        repository
            .upsert_provider_model(&ProviderModelRecord {
                model_id: "model_mock_image".to_string(),
                provider_id: "provider_mock".to_string(),
                provider_model_id: "mock/image".to_string(),
                display_name: "Mock Image".to_string(),
                capability: "text_to_image".to_string(),
                config_json: json!({ "ratio": "9:16" }),
                enabled: true,
            })
            .expect("model should upsert");
        repository
            .upsert_workflow_preset(&WorkflowPresetRecord {
                preset_id: "preset_mock_image".to_string(),
                provider_id: Some("provider_mock".to_string()),
                model_id: Some("model_mock_image".to_string()),
                name: "Mock still".to_string(),
                kind: "image_to_video_still".to_string(),
                capability: "text_to_image".to_string(),
                config_json: json!({ "count": 4 }),
                enabled: true,
            })
            .expect("preset should upsert");

        assert_eq!(
            repository
                .list_providers()
                .expect("providers should list")
                .len(),
            1
        );
        assert_eq!(
            repository
                .list_provider_models(Some("provider_mock"))
                .expect("models should list")
                .len(),
            1
        );
        assert_eq!(
            repository
                .list_workflow_presets(Some("text_to_image"))
                .expect("presets should list")
                .len(),
            1
        );

        cleanup(path);
    }

    fn test_database_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vt-ai-short-video-maker-{name}-{}-{nanos}.sqlite3",
            std::process::id()
        ))
    }

    fn cleanup(path: PathBuf) {
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(path.with_extension("sqlite3-wal"));
        let _ = fs::remove_file(path.with_extension("sqlite3-shm"));
    }
}
