use crate::db::{Database, Repository};
use crate::domain::long_content::LongContentPlanDto;
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct LongContentRepository<'db> {
    database: &'db Database,
}

impl<'db> LongContentRepository<'db> {
    pub fn new(database: &'db Database) -> Self {
        Self { database }
    }

    pub fn insert_plan(
        &self,
        record: NewLongContentPlanRecord,
    ) -> Result<LongContentPlanDto, String> {
        let chapter_ids_json =
            serde_json::to_string(&record.chapter_ids).map_err(|error| error.to_string())?;
        let content_json =
            serde_json::to_string(&record.content).map_err(|error| error.to_string())?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    INSERT INTO long_content_plans (
                        plan_id, project_id, plan_kind, parent_plan_id,
                        chapter_ids_json, content_json, status, schema_version
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'waiting_user', ?7)
                    "#,
                    params![
                        record.plan_id,
                        record.project_id,
                        record.plan_kind,
                        record.parent_plan_id,
                        chapter_ids_json,
                        content_json,
                        record.schema_version,
                    ],
                )?;
                read_plan(connection, &record.plan_id)
            })
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "Long content plan was inserted but cannot be read back.".to_string())
    }

    pub fn list_plans(
        &self,
        project_id: &str,
        plan_kind: Option<&str>,
    ) -> Result<Vec<LongContentPlanDto>, String> {
        self.database
            .with_connection(|connection| read_project_plans(connection, project_id, plan_kind))
            .map_err(|error| error.to_string())
    }

    pub fn set_plan_status(
        &self,
        plan_id: &str,
        status: &str,
    ) -> Result<LongContentPlanDto, String> {
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    UPDATE long_content_plans
                    SET status = ?1,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE plan_id = ?2
                    "#,
                    params![status, plan_id],
                )?;
                read_plan(connection, plan_id)
            })
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("Long content plan not found: {plan_id}"))
    }
}

impl Repository for LongContentRepository<'_> {
    fn database(&self) -> &Database {
        self.database
    }
}

#[derive(Debug, Clone)]
pub struct NewLongContentPlanRecord {
    pub plan_id: String,
    pub project_id: String,
    pub plan_kind: String,
    pub parent_plan_id: Option<String>,
    pub chapter_ids: Vec<String>,
    pub content: Value,
    pub schema_version: u32,
}

impl NewLongContentPlanRecord {
    pub fn new(
        project_id: String,
        plan_kind: String,
        parent_plan_id: Option<String>,
        chapter_ids: Vec<String>,
        content: Value,
        schema_version: u32,
    ) -> Self {
        Self {
            plan_id: create_id("long_plan"),
            project_id,
            plan_kind,
            parent_plan_id,
            chapter_ids,
            content,
            schema_version,
        }
    }
}

fn read_project_plans(
    connection: &Connection,
    project_id: &str,
    plan_kind: Option<&str>,
) -> Result<Vec<LongContentPlanDto>, rusqlite::Error> {
    let mut sql = r#"
        SELECT
            plan_id, project_id, plan_kind, parent_plan_id, chapter_ids_json,
            content_json, status, schema_version, created_at, updated_at
        FROM long_content_plans
        WHERE project_id = ?1
    "#
    .to_string();
    if plan_kind.is_some() {
        sql.push_str(" AND plan_kind = ?2");
    }
    sql.push_str(" ORDER BY created_at ASC");

    let mut statement = connection.prepare(&sql)?;
    let rows = if let Some(plan_kind) = plan_kind {
        statement.query_map(params![project_id, plan_kind], row_to_plan)?
    } else {
        statement.query_map(params![project_id], row_to_plan)?
    };
    rows.collect()
}

fn read_plan(
    connection: &Connection,
    plan_id: &str,
) -> Result<Option<LongContentPlanDto>, rusqlite::Error> {
    connection
        .query_row(
            r#"
            SELECT
                plan_id, project_id, plan_kind, parent_plan_id, chapter_ids_json,
                content_json, status, schema_version, created_at, updated_at
            FROM long_content_plans
            WHERE plan_id = ?1
            "#,
            [plan_id],
            row_to_plan,
        )
        .optional()
}

fn row_to_plan(row: &Row<'_>) -> Result<LongContentPlanDto, rusqlite::Error> {
    let chapter_ids_json: String = row.get(4)?;
    let content_json: String = row.get(5)?;
    Ok(LongContentPlanDto {
        plan_id: row.get(0)?,
        project_id: row.get(1)?,
        plan_kind: row.get(2)?,
        parent_plan_id: row.get(3)?,
        chapter_ids: serde_json::from_str(&chapter_ids_json).unwrap_or_default(),
        content: serde_json::from_str(&content_json).unwrap_or_default(),
        status: row.get(6)?,
        schema_version: row.get::<_, i64>(7)? as u32,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn create_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{prefix}_{nanos}")
}
