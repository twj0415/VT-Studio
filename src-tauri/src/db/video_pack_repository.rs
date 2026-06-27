use crate::db::{Database, Repository};
use crate::domain::video_pack::VideoPackDto;
use crate::security::secret_guard::reject_json_secrets;
use rusqlite::{params, OptionalExtension, Row};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct VideoPackRecord {
    pub pack_id: String,
    pub source_type: String,
    pub name: String,
    pub description: String,
    pub applicable_input_types: Vec<String>,
    pub content_category: Option<String>,
    pub default_tone: Option<String>,
    pub default_aspect_ratio: String,
    pub default_duration_seconds: u32,
    pub default_scene_count: u32,
    pub rule_refs: Value,
    pub recommended_executable_refs: Value,
    pub asset_refs: Value,
    pub is_enabled: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

pub struct VideoPackRepository<'db> {
    database: &'db Database,
}

impl<'db> VideoPackRepository<'db> {
    pub fn new(database: &'db Database) -> Self {
        Self { database }
    }

    pub fn create_pack_id() -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        format!("pack_{nanos}")
    }

    pub fn upsert(&self, pack: &VideoPackRecord) -> Result<VideoPackDto, String> {
        reject_json_secrets(&pack.rule_refs)?;
        reject_json_secrets(&pack.recommended_executable_refs)?;
        reject_json_secrets(&pack.asset_refs)?;
        let applicable_input_types_json = serde_json::to_string(&pack.applicable_input_types)
            .map_err(|error| error.to_string())?;
        let rule_refs_json =
            serde_json::to_string(&pack.rule_refs).map_err(|error| error.to_string())?;
        let recommended_executable_refs_json =
            serde_json::to_string(&pack.recommended_executable_refs)
                .map_err(|error| error.to_string())?;
        let asset_refs_json =
            serde_json::to_string(&pack.asset_refs).map_err(|error| error.to_string())?;

        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    INSERT INTO video_packs (
                        pack_id, source_type, name, description, applicable_input_types_json,
                        content_category, default_tone, default_aspect_ratio, default_duration_seconds,
                        default_scene_count, rule_refs_json, recommended_executable_refs_json,
                        asset_refs_json, is_enabled, updated_at
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, CURRENT_TIMESTAMP)
                    ON CONFLICT(pack_id) DO UPDATE SET
                        source_type = excluded.source_type,
                        name = excluded.name,
                        description = excluded.description,
                        applicable_input_types_json = excluded.applicable_input_types_json,
                        content_category = excluded.content_category,
                        default_tone = excluded.default_tone,
                        default_aspect_ratio = excluded.default_aspect_ratio,
                        default_duration_seconds = excluded.default_duration_seconds,
                        default_scene_count = excluded.default_scene_count,
                        rule_refs_json = excluded.rule_refs_json,
                        recommended_executable_refs_json = excluded.recommended_executable_refs_json,
                        asset_refs_json = excluded.asset_refs_json,
                        is_enabled = excluded.is_enabled,
                        updated_at = CURRENT_TIMESTAMP
                    "#,
                    params![
                        pack.pack_id,
                        pack.source_type,
                        pack.name,
                        pack.description,
                        applicable_input_types_json,
                        pack.content_category,
                        pack.default_tone,
                        pack.default_aspect_ratio,
                        pack.default_duration_seconds,
                        pack.default_scene_count,
                        rule_refs_json,
                        recommended_executable_refs_json,
                        asset_refs_json,
                        bool_to_i64(pack.is_enabled),
                    ],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get(&pack.pack_id)?
            .ok_or_else(|| format!("Video pack {} was saved but cannot be read.", pack.pack_id))
    }

    pub fn get(&self, pack_id: &str) -> Result<Option<VideoPackDto>, String> {
        self.database
            .with_connection(|connection| {
                connection
                    .query_row(
                        r#"
                        SELECT pack_id, source_type, name, description, applicable_input_types_json,
                               content_category, default_tone, default_aspect_ratio,
                               default_duration_seconds, default_scene_count, rule_refs_json,
                               recommended_executable_refs_json, asset_refs_json, is_enabled,
                               created_at, updated_at
                        FROM video_packs
                        WHERE pack_id = ?1
                        "#,
                        [pack_id],
                        |row| {
                            let record = record_from_row(row)?;
                            dto_with_reference_count(connection, record)
                        },
                    )
                    .optional()
            })
            .map_err(|error| error.to_string())
    }

    pub fn list(&self) -> Result<Vec<VideoPackDto>, String> {
        self.database
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    r#"
                    SELECT pack_id, source_type, name, description, applicable_input_types_json,
                           content_category, default_tone, default_aspect_ratio,
                           default_duration_seconds, default_scene_count, rule_refs_json,
                           recommended_executable_refs_json, asset_refs_json, is_enabled,
                           created_at, updated_at
                    FROM video_packs
                    ORDER BY source_type ASC, created_at ASC
                    "#,
                )?;
                let records = statement
                    .query_map([], record_from_row)?
                    .collect::<Result<Vec<_>, _>>()?;
                records
                    .into_iter()
                    .map(|record| dto_with_reference_count(connection, record))
                    .collect()
            })
            .map_err(|error| error.to_string())
    }

    pub fn delete(&self, pack_id: &str) -> Result<VideoPackDto, String> {
        let pack = self
            .get(pack_id)?
            .ok_or_else(|| format!("video_pack.not_found: {pack_id}"))?;
        self.database
            .with_connection(|connection| {
                connection.execute("DELETE FROM video_packs WHERE pack_id = ?1", [pack_id])?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;
        Ok(pack)
    }
}

impl Repository for VideoPackRepository<'_> {
    fn database(&self) -> &Database {
        self.database
    }
}

fn record_from_row(row: &Row<'_>) -> Result<VideoPackRecord, rusqlite::Error> {
    let applicable_input_types_json: String = row.get(4)?;
    let rule_refs_json: String = row.get(10)?;
    let recommended_executable_refs_json: String = row.get(11)?;
    let asset_refs_json: String = row.get(12)?;
    Ok(VideoPackRecord {
        pack_id: row.get(0)?,
        source_type: row.get(1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        applicable_input_types: parse_string_array(&applicable_input_types_json),
        content_category: row.get(5)?,
        default_tone: row.get(6)?,
        default_aspect_ratio: row.get(7)?,
        default_duration_seconds: row.get::<_, i64>(8)?.max(1) as u32,
        default_scene_count: row.get::<_, i64>(9)?.max(1) as u32,
        rule_refs: parse_json(&rule_refs_json, json!({})),
        recommended_executable_refs: parse_json(&recommended_executable_refs_json, json!({})),
        asset_refs: parse_json(&asset_refs_json, json!([])),
        is_enabled: int_to_bool(row.get(13)?),
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
    })
}

fn dto_with_reference_count(
    connection: &rusqlite::Connection,
    record: VideoPackRecord,
) -> Result<VideoPackDto, rusqlite::Error> {
    let project_reference_count: i64 = connection.query_row(
        r#"
        SELECT COUNT(*)
        FROM projects
        WHERE active_pack_id = ?1
           OR rule_refs_json LIKE '%' || ?1 || '%'
           OR executable_refs_json LIKE '%' || ?1 || '%'
        "#,
        [record.pack_id.as_str()],
        |row| row.get(0),
    )?;

    Ok(VideoPackDto {
        pack_id: record.pack_id,
        source_type: record.source_type,
        name: record.name,
        description: record.description,
        applicable_input_types: record.applicable_input_types,
        content_category: record.content_category,
        default_tone: record.default_tone,
        default_aspect_ratio: record.default_aspect_ratio,
        default_duration_seconds: record.default_duration_seconds,
        default_scene_count: record.default_scene_count,
        rule_refs: record.rule_refs,
        recommended_executable_refs: record.recommended_executable_refs,
        asset_refs: record.asset_refs,
        is_enabled: record.is_enabled,
        created_at: record.created_at,
        updated_at: record.updated_at,
        project_reference_count: project_reference_count.max(0) as u32,
    })
}

fn parse_string_array(value: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(value).unwrap_or_default()
}

fn parse_json(value: &str, fallback: Value) -> Value {
    serde_json::from_str(value).unwrap_or(fallback)
}

fn bool_to_i64(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

fn int_to_bool(value: i64) -> bool {
    value != 0
}
