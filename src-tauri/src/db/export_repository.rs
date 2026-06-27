use crate::db::{Database, Repository};
use crate::domain::export::ExportRecordDto;
use crate::security::path_guard::PathGuard;
use rusqlite::{params, OptionalExtension};
use serde_json::Value;

pub struct ExportRepository<'db> {
    database: &'db Database,
}

#[derive(Debug, Clone)]
pub struct NewExportRecord {
    pub export_id: String,
    pub project_id: String,
    pub composition_task_id: Option<String>,
    pub export_kind: String,
    pub source_relative_path: Option<String>,
    pub target_relative_path: Option<String>,
    pub status: String,
    pub error_json: Option<Value>,
    pub metadata_json: Value,
}

impl<'db> ExportRepository<'db> {
    pub fn new(database: &'db Database) -> Self {
        Self { database }
    }

    pub fn insert_export_record(&self, record: NewExportRecord) -> Result<ExportRecordDto, String> {
        validate_export_record(&record)?;
        let error_json = record
            .error_json
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| error.to_string())?;
        let metadata_json =
            serde_json::to_string(&record.metadata_json).map_err(|error| error.to_string())?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    INSERT INTO export_records (
                        export_id, project_id, composition_task_id, export_kind,
                        source_relative_path, target_relative_path, status,
                        started_at, finished_at, error_json, metadata_json, updated_at
                    )
                    VALUES (
                        ?1, ?2, ?3, ?4, ?5, ?6, ?7,
                        CURRENT_TIMESTAMP,
                        CASE WHEN ?7 IN ('succeeded', 'failed') THEN CURRENT_TIMESTAMP ELSE NULL END,
                        ?8, ?9, CURRENT_TIMESTAMP
                    )
                    "#,
                    params![
                        record.export_id,
                        record.project_id,
                        record.composition_task_id,
                        record.export_kind,
                        record.source_relative_path,
                        record.target_relative_path,
                        record.status,
                        error_json,
                        metadata_json,
                    ],
                )?;
                read_export_record(connection, &record.export_id)?
                    .ok_or(rusqlite::Error::QueryReturnedNoRows)
            })
            .map_err(|error| error.to_string())
    }

    pub fn list_project_exports(&self, project_id: &str) -> Result<Vec<ExportRecordDto>, String> {
        self.database
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    r#"
                    SELECT
                        export_id, project_id, composition_task_id, export_kind,
                        source_relative_path, target_relative_path, status,
                        started_at, finished_at, error_json, metadata_json,
                        created_at, updated_at
                    FROM export_records
                    WHERE project_id = ?1
                    ORDER BY created_at DESC, export_id DESC
                    "#,
                )?;
                let rows = statement.query_map([project_id], row_to_export_record)?;
                rows.collect()
            })
            .map_err(|error| error.to_string())
    }

    pub fn get_export_record(&self, export_id: &str) -> Result<Option<ExportRecordDto>, String> {
        self.database
            .with_connection(|connection| read_export_record(connection, export_id))
            .map_err(|error| error.to_string())
    }
}

impl Repository for ExportRepository<'_> {
    fn database(&self) -> &Database {
        self.database
    }
}

fn validate_export_record(record: &NewExportRecord) -> Result<(), String> {
    if record.project_id.trim().is_empty() {
        return Err("project_id is required for export record.".to_string());
    }
    if !matches!(
        record.export_kind.as_str(),
        "final_video" | "project_package"
    ) {
        return Err("unsupported export kind.".to_string());
    }
    if !matches!(record.status.as_str(), "running" | "succeeded" | "failed") {
        return Err("invalid export status.".to_string());
    }
    if let Some(source) = record.source_relative_path.as_deref() {
        let normalized = PathGuard::validate_relative_path(source)?;
        if !normalized.starts_with("outputs/") {
            return Err("export source must be inside outputs bucket.".to_string());
        }
    }
    if let Some(target) = record.target_relative_path.as_deref() {
        let normalized = PathGuard::validate_relative_path(target)?;
        let allowed = match record.export_kind.as_str() {
            "final_video" => normalized.starts_with("outputs/user_exports/"),
            "project_package" => normalized.starts_with("outputs/project_packages/"),
            _ => false,
        };
        if !allowed {
            return Err(
                "export target must be inside the controlled output bucket for its export kind."
                    .to_string(),
            );
        }
    }
    Ok(())
}

fn read_export_record(
    connection: &rusqlite::Connection,
    export_id: &str,
) -> Result<Option<ExportRecordDto>, rusqlite::Error> {
    connection
        .query_row(
            r#"
            SELECT
                export_id, project_id, composition_task_id, export_kind,
                source_relative_path, target_relative_path, status,
                started_at, finished_at, error_json, metadata_json,
                created_at, updated_at
            FROM export_records
            WHERE export_id = ?1
            "#,
            [export_id],
            row_to_export_record,
        )
        .optional()
}

fn row_to_export_record(row: &rusqlite::Row<'_>) -> Result<ExportRecordDto, rusqlite::Error> {
    let error_json: Option<String> = row.get(9)?;
    let metadata_json: String = row.get(10)?;
    Ok(ExportRecordDto {
        export_id: row.get(0)?,
        project_id: row.get(1)?,
        composition_task_id: row.get(2)?,
        export_kind: row.get(3)?,
        source_relative_path: row.get(4)?,
        target_relative_path: row.get(5)?,
        status: row.get(6)?,
        started_at: row.get(7)?,
        finished_at: row.get(8)?,
        error_json: error_json.and_then(|value| serde_json::from_str::<Value>(&value).ok()),
        metadata_json: serde_json::from_str::<Value>(&metadata_json)
            .unwrap_or_else(|_| Value::Object(Default::default())),
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}
