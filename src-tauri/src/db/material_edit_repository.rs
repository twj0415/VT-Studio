use crate::db::{Database, Repository};
use crate::domain::material_edit::{
    MaterialAnalysisSuggestionDto, StoryboardMaterialRequirementDto,
};
use crate::domain::media::AssetReferenceDto;
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct MaterialEditRepository<'db> {
    database: &'db Database,
}

impl<'db> MaterialEditRepository<'db> {
    pub fn new(database: &'db Database) -> Self {
        Self { database }
    }

    pub fn insert_analysis_suggestion(
        &self,
        record: NewMaterialAnalysisSuggestionRecord,
    ) -> Result<MaterialAnalysisSuggestionDto, String> {
        let suggestion_json =
            serde_json::to_string(&record.suggestion).map_err(|error| error.to_string())?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    INSERT INTO material_analysis_suggestions (
                        suggestion_id, project_id, asset_id, provider_id, model_id,
                        suggestion_json, status
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'waiting_user')
                    "#,
                    params![
                        record.suggestion_id,
                        record.project_id,
                        record.asset_id,
                        record.provider_id,
                        record.model_id,
                        suggestion_json,
                    ],
                )?;
                read_suggestion(connection, &record.suggestion_id)
            })
            .map_err(|error| error.to_string())?
            .ok_or_else(|| {
                "Material analysis suggestion was inserted but cannot be read.".to_string()
            })
    }

    pub fn list_analysis_suggestions(
        &self,
        project_id: &str,
    ) -> Result<Vec<MaterialAnalysisSuggestionDto>, String> {
        self.database
            .with_connection(|connection| list_suggestions(connection, project_id))
            .map_err(|error| error.to_string())
    }

    pub fn set_analysis_suggestion_status(
        &self,
        suggestion_id: &str,
        status: &str,
    ) -> Result<MaterialAnalysisSuggestionDto, String> {
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    UPDATE material_analysis_suggestions
                    SET status = ?1,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE suggestion_id = ?2
                    "#,
                    params![status, suggestion_id],
                )?;
                read_suggestion(connection, suggestion_id)
            })
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("Material analysis suggestion not found: {suggestion_id}"))
    }

    pub fn upsert_storyboard_material_requirement(
        &self,
        record: NewStoryboardMaterialRequirementRecord,
    ) -> Result<StoryboardMaterialRequirementDto, String> {
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    INSERT INTO storyboard_material_requirements (
                        item_id, project_id, requirement_status, no_material_reason,
                        confirmed_by_user, updated_at
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP)
                    ON CONFLICT(item_id) DO UPDATE SET
                        project_id = excluded.project_id,
                        requirement_status = excluded.requirement_status,
                        no_material_reason = excluded.no_material_reason,
                        confirmed_by_user = excluded.confirmed_by_user,
                        updated_at = CURRENT_TIMESTAMP
                    "#,
                    params![
                        record.item_id,
                        record.project_id,
                        record.requirement_status,
                        record.no_material_reason,
                        bool_to_i64(record.confirmed_by_user),
                    ],
                )?;
                read_requirement(connection, &record.item_id)
            })
            .map_err(|error| error.to_string())?
            .ok_or_else(|| {
                "Storyboard material requirement was saved but cannot be read.".to_string()
            })
    }

    pub fn create_storyboard_material_reference(
        &self,
        reference: NewStoryboardMaterialReferenceRecord,
    ) -> Result<AssetReferenceDto, String> {
        self.database
            .with_connection(|connection| {
                if let Some(existing) = read_storyboard_material_reference(
                    connection,
                    &reference.item_id,
                    &reference.asset_id,
                )? {
                    return Ok(Some(existing));
                }
                connection.execute(
                    r#"
                    INSERT INTO asset_references (
                        reference_id, asset_id, owner_kind, owner_id, usage_kind
                    )
                    VALUES (?1, ?2, 'storyboard_item', ?3, 'source_material')
                    "#,
                    params![
                        reference.reference_id,
                        reference.asset_id,
                        reference.item_id
                    ],
                )?;
                read_storyboard_material_reference(
                    connection,
                    &reference.item_id,
                    &reference.asset_id,
                )
            })
            .map_err(|error| error.to_string())?
            .ok_or_else(|| {
                "Storyboard material reference was saved but cannot be read.".to_string()
            })
    }

    pub fn list_storyboard_material_requirements(
        &self,
        project_id: &str,
    ) -> Result<Vec<StoryboardMaterialRequirementDto>, String> {
        self.database
            .with_connection(|connection| list_requirements(connection, project_id))
            .map_err(|error| error.to_string())
    }

    pub fn list_storyboard_material_references(
        &self,
        project_id: &str,
    ) -> Result<Vec<AssetReferenceDto>, String> {
        self.database
            .with_connection(|connection| {
                list_storyboard_material_references(connection, project_id)
            })
            .map_err(|error| error.to_string())
    }
}

impl Repository for MaterialEditRepository<'_> {
    fn database(&self) -> &Database {
        self.database
    }
}

#[derive(Debug, Clone)]
pub struct NewMaterialAnalysisSuggestionRecord {
    pub suggestion_id: String,
    pub project_id: String,
    pub asset_id: String,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub suggestion: Value,
}

impl NewMaterialAnalysisSuggestionRecord {
    pub fn new(
        project_id: String,
        asset_id: String,
        provider_id: Option<String>,
        model_id: Option<String>,
        suggestion: Value,
    ) -> Self {
        Self {
            suggestion_id: create_id("material_suggestion"),
            project_id,
            asset_id,
            provider_id,
            model_id,
            suggestion,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NewStoryboardMaterialRequirementRecord {
    pub item_id: String,
    pub project_id: String,
    pub requirement_status: String,
    pub no_material_reason: Option<String>,
    pub confirmed_by_user: bool,
}

#[derive(Debug, Clone)]
pub struct NewStoryboardMaterialReferenceRecord {
    pub reference_id: String,
    pub item_id: String,
    pub asset_id: String,
}

impl NewStoryboardMaterialReferenceRecord {
    pub fn new(item_id: String, asset_id: String) -> Self {
        Self {
            reference_id: create_id("asset_ref"),
            item_id,
            asset_id,
        }
    }
}

fn list_suggestions(
    connection: &Connection,
    project_id: &str,
) -> Result<Vec<MaterialAnalysisSuggestionDto>, rusqlite::Error> {
    let mut statement = connection.prepare(
        r#"
        SELECT suggestion_id, project_id, asset_id, provider_id, model_id,
               suggestion_json, status, created_at, updated_at
        FROM material_analysis_suggestions
        WHERE project_id = ?1
        ORDER BY created_at ASC
        "#,
    )?;
    let rows = statement.query_map([project_id], row_to_suggestion)?;
    rows.collect()
}

fn read_suggestion(
    connection: &Connection,
    suggestion_id: &str,
) -> Result<Option<MaterialAnalysisSuggestionDto>, rusqlite::Error> {
    connection
        .query_row(
            r#"
            SELECT suggestion_id, project_id, asset_id, provider_id, model_id,
                   suggestion_json, status, created_at, updated_at
            FROM material_analysis_suggestions
            WHERE suggestion_id = ?1
            "#,
            [suggestion_id],
            row_to_suggestion,
        )
        .optional()
}

fn row_to_suggestion(row: &Row<'_>) -> Result<MaterialAnalysisSuggestionDto, rusqlite::Error> {
    let suggestion_json: String = row.get(5)?;
    Ok(MaterialAnalysisSuggestionDto {
        suggestion_id: row.get(0)?,
        project_id: row.get(1)?,
        asset_id: row.get(2)?,
        provider_id: row.get(3)?,
        model_id: row.get(4)?,
        suggestion: serde_json::from_str(&suggestion_json).unwrap_or_default(),
        status: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn list_requirements(
    connection: &Connection,
    project_id: &str,
) -> Result<Vec<StoryboardMaterialRequirementDto>, rusqlite::Error> {
    let mut statement = connection.prepare(
        r#"
        SELECT item_id, project_id, requirement_status, no_material_reason,
               confirmed_by_user, created_at, updated_at
        FROM storyboard_material_requirements
        WHERE project_id = ?1
        ORDER BY created_at ASC
        "#,
    )?;
    let rows = statement.query_map([project_id], row_to_requirement)?;
    rows.collect()
}

fn read_requirement(
    connection: &Connection,
    item_id: &str,
) -> Result<Option<StoryboardMaterialRequirementDto>, rusqlite::Error> {
    connection
        .query_row(
            r#"
            SELECT item_id, project_id, requirement_status, no_material_reason,
                   confirmed_by_user, created_at, updated_at
            FROM storyboard_material_requirements
            WHERE item_id = ?1
            "#,
            [item_id],
            row_to_requirement,
        )
        .optional()
}

fn row_to_requirement(row: &Row<'_>) -> Result<StoryboardMaterialRequirementDto, rusqlite::Error> {
    Ok(StoryboardMaterialRequirementDto {
        item_id: row.get(0)?,
        project_id: row.get(1)?,
        requirement_status: row.get(2)?,
        no_material_reason: row.get(3)?,
        confirmed_by_user: row.get::<_, i64>(4)? == 1,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

fn list_storyboard_material_references(
    connection: &Connection,
    project_id: &str,
) -> Result<Vec<AssetReferenceDto>, rusqlite::Error> {
    let mut statement = connection.prepare(
        r#"
        SELECT
            asset_references.reference_id,
            asset_references.asset_id,
            asset_references.owner_kind,
            asset_references.owner_id,
            asset_references.usage_kind,
            asset_references.created_at
        FROM asset_references
        JOIN storyboard_items ON storyboard_items.item_id = asset_references.owner_id
        WHERE storyboard_items.project_id = ?1
          AND asset_references.owner_kind = 'storyboard_item'
          AND asset_references.usage_kind = 'source_material'
        ORDER BY storyboard_items.item_index ASC, asset_references.created_at ASC
        "#,
    )?;
    let rows = statement.query_map([project_id], row_to_asset_reference)?;
    rows.collect()
}

fn read_storyboard_material_reference(
    connection: &Connection,
    item_id: &str,
    asset_id: &str,
) -> Result<Option<AssetReferenceDto>, rusqlite::Error> {
    connection
        .query_row(
            r#"
            SELECT reference_id, asset_id, owner_kind, owner_id, usage_kind, created_at
            FROM asset_references
            WHERE owner_kind = 'storyboard_item'
              AND owner_id = ?1
              AND asset_id = ?2
              AND usage_kind = 'source_material'
            "#,
            params![item_id, asset_id],
            row_to_asset_reference,
        )
        .optional()
}

fn row_to_asset_reference(row: &Row<'_>) -> Result<AssetReferenceDto, rusqlite::Error> {
    Ok(AssetReferenceDto {
        reference_id: row.get(0)?,
        asset_id: row.get(1)?,
        owner_kind: row.get(2)?,
        owner_id: row.get(3)?,
        usage_kind: row.get(4)?,
        created_at: row.get(5)?,
    })
}

fn bool_to_i64(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

fn create_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{prefix}_{nanos}")
}
