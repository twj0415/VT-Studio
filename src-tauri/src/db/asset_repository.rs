use crate::db::{Database, Repository};
use crate::domain::media::{AssetDto, AssetReferenceDto};
use rusqlite::{params, OptionalExtension, Row};
use serde_json::{json, Value};

pub struct AssetRepository<'db> {
    database: &'db Database,
}

pub struct NewAssetRecord {
    pub asset_id: String,
    pub kind: String,
    pub relative_path: String,
    pub source_kind: String,
    pub mime_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub checksum: Option<String>,
    pub is_builtin: bool,
    pub metadata: Value,
}

pub struct NewAssetReferenceRecord {
    pub reference_id: String,
    pub asset_id: String,
    pub owner_kind: String,
    pub owner_id: String,
    pub usage_kind: String,
}

pub struct NewGeneratedImageAssetRecord {
    pub asset: NewAssetRecord,
    pub reference: NewAssetReferenceRecord,
    pub project_id: String,
    pub reference_entry: Value,
}

impl<'db> AssetRepository<'db> {
    pub fn new(database: &'db Database) -> Self {
        Self { database }
    }

    pub fn insert_asset(&self, asset: &NewAssetRecord) -> Result<AssetDto, String> {
        let metadata_json =
            serde_json::to_string(&asset.metadata).map_err(|error| error.to_string())?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    INSERT INTO assets (
                        asset_id, kind, relative_path, source_kind, mime_type, size_bytes,
                        checksum, is_builtin, lifecycle, metadata_json, updated_at
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'active', ?9, CURRENT_TIMESTAMP)
                    "#,
                    params![
                        asset.asset_id,
                        asset.kind,
                        asset.relative_path,
                        asset.source_kind,
                        asset.mime_type,
                        asset.size_bytes,
                        asset.checksum,
                        bool_to_i64(asset.is_builtin),
                        metadata_json,
                    ],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get_asset(&asset.asset_id)?
            .ok_or_else(|| format!("Asset {} was inserted but cannot be read.", asset.asset_id))
    }

    pub fn list_assets(
        &self,
        kind: Option<&str>,
        include_deleted: bool,
    ) -> Result<Vec<AssetDto>, String> {
        self.database
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    r#"
                    SELECT asset_id, kind, relative_path, source_kind, mime_type, size_bytes,
                           checksum, is_builtin, lifecycle, metadata_json, created_at, updated_at
                    FROM assets
                    ORDER BY created_at DESC
                    "#,
                )?;
                let rows = statement.query_map([], asset_from_row)?;
                let mut assets = rows.collect::<Result<Vec<_>, _>>()?;
                if let Some(kind) = kind {
                    assets.retain(|asset| asset.kind == kind);
                }
                if !include_deleted {
                    assets.retain(|asset| asset.lifecycle != "deleted");
                }
                Ok(assets)
            })
            .map_err(|error| error.to_string())
    }

    pub fn get_asset(&self, asset_id: &str) -> Result<Option<AssetDto>, String> {
        self.database
            .with_connection(|connection| {
                connection
                    .query_row(
                        r#"
                        SELECT asset_id, kind, relative_path, source_kind, mime_type, size_bytes,
                               checksum, is_builtin, lifecycle, metadata_json, created_at, updated_at
                        FROM assets
                        WHERE asset_id = ?1
                        "#,
                        [asset_id],
                        asset_from_row,
                    )
                    .optional()
            })
            .map_err(|error| error.to_string())
    }

    pub fn mark_deleted(&self, asset_id: &str) -> Result<AssetDto, String> {
        self.database
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE assets SET lifecycle = 'deleted', updated_at = CURRENT_TIMESTAMP WHERE asset_id = ?1",
                    [asset_id],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get_asset(asset_id)?
            .ok_or_else(|| format!("Asset not found: {asset_id}"))
    }

    pub fn create_reference(
        &self,
        reference: &NewAssetReferenceRecord,
    ) -> Result<AssetReferenceDto, String> {
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    INSERT INTO asset_references (
                        reference_id, asset_id, owner_kind, owner_id, usage_kind
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5)
                    "#,
                    params![
                        reference.reference_id,
                        reference.asset_id,
                        reference.owner_kind,
                        reference.owner_id,
                        reference.usage_kind,
                    ],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get_reference(&reference.reference_id)?.ok_or_else(|| {
            format!(
                "Asset reference {} was inserted but cannot be read.",
                reference.reference_id
            )
        })
    }

    pub fn create_reference_and_append_to_style_bible(
        &self,
        reference: &NewAssetReferenceRecord,
        reference_entry: &Value,
    ) -> Result<AssetReferenceDto, String> {
        self.database
            .transaction(|transaction| {
                transaction.execute(
                    r#"
                    INSERT INTO asset_references (
                        reference_id, asset_id, owner_kind, owner_id, usage_kind
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5)
                    "#,
                    params![
                        reference.reference_id,
                        reference.asset_id,
                        reference.owner_kind,
                        reference.owner_id,
                        reference.usage_kind,
                    ],
                )?;
                update_bible_reference_images(
                    transaction,
                    "style_bibles",
                    "style_bible_id",
                    &reference.owner_id,
                    reference_entry,
                    true,
                )?;
                let saved = transaction.query_row(
                    r#"
                    SELECT reference_id, asset_id, owner_kind, owner_id, usage_kind, created_at
                    FROM asset_references
                    WHERE reference_id = ?1
                    "#,
                    [reference.reference_id.as_str()],
                    reference_from_row,
                )?;
                Ok(saved)
            })
            .map_err(|error| error.to_string())
    }

    pub fn create_reference_and_append_to_character_bible(
        &self,
        reference: &NewAssetReferenceRecord,
        reference_entry: &Value,
    ) -> Result<AssetReferenceDto, String> {
        self.database
            .transaction(|transaction| {
                transaction.execute(
                    r#"
                    INSERT INTO asset_references (
                        reference_id, asset_id, owner_kind, owner_id, usage_kind
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5)
                    "#,
                    params![
                        reference.reference_id,
                        reference.asset_id,
                        reference.owner_kind,
                        reference.owner_id,
                        reference.usage_kind,
                    ],
                )?;
                update_bible_reference_images(
                    transaction,
                    "character_bibles",
                    "character_bible_id",
                    &reference.owner_id,
                    reference_entry,
                    true,
                )?;
                let saved = transaction.query_row(
                    r#"
                    SELECT reference_id, asset_id, owner_kind, owner_id, usage_kind, created_at
                    FROM asset_references
                    WHERE reference_id = ?1
                    "#,
                    [reference.reference_id.as_str()],
                    reference_from_row,
                )?;
                Ok(saved)
            })
            .map_err(|error| error.to_string())
    }

    pub fn create_reference_and_append_to_location_bible(
        &self,
        reference: &NewAssetReferenceRecord,
        reference_entry: &Value,
    ) -> Result<AssetReferenceDto, String> {
        self.database
            .transaction(|transaction| {
                transaction.execute(
                    r#"
                    INSERT INTO asset_references (
                        reference_id, asset_id, owner_kind, owner_id, usage_kind
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5)
                    "#,
                    params![
                        reference.reference_id,
                        reference.asset_id,
                        reference.owner_kind,
                        reference.owner_id,
                        reference.usage_kind,
                    ],
                )?;
                update_bible_reference_images(
                    transaction,
                    "location_bibles",
                    "location_bible_id",
                    &reference.owner_id,
                    reference_entry,
                    true,
                )?;
                let saved = transaction.query_row(
                    r#"
                    SELECT reference_id, asset_id, owner_kind, owner_id, usage_kind, created_at
                    FROM asset_references
                    WHERE reference_id = ?1
                    "#,
                    [reference.reference_id.as_str()],
                    reference_from_row,
                )?;
                Ok(saved)
            })
            .map_err(|error| error.to_string())
    }

    pub fn insert_generated_image_asset(
        &self,
        record: &NewGeneratedImageAssetRecord,
    ) -> Result<(AssetDto, AssetReferenceDto), String> {
        let metadata_json =
            serde_json::to_string(&record.asset.metadata).map_err(|error| error.to_string())?;
        self.database
            .transaction(|transaction| {
                ensure_owner_belongs_to_project(
                    transaction,
                    &record.project_id,
                    &record.reference.owner_kind,
                    &record.reference.owner_id,
                )?;
                transaction.execute(
                    r#"
                    INSERT INTO assets (
                        asset_id, kind, relative_path, source_kind, mime_type, size_bytes,
                        checksum, is_builtin, lifecycle, metadata_json, updated_at
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'active', ?9, CURRENT_TIMESTAMP)
                    "#,
                    params![
                        record.asset.asset_id,
                        record.asset.kind,
                        record.asset.relative_path,
                        record.asset.source_kind,
                        record.asset.mime_type,
                        record.asset.size_bytes,
                        record.asset.checksum,
                        bool_to_i64(record.asset.is_builtin),
                        metadata_json,
                    ],
                )?;
                transaction.execute(
                    r#"
                    INSERT INTO asset_references (
                        reference_id, asset_id, owner_kind, owner_id, usage_kind
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5)
                    "#,
                    params![
                        record.reference.reference_id,
                        record.reference.asset_id,
                        record.reference.owner_kind,
                        record.reference.owner_id,
                        record.reference.usage_kind,
                    ],
                )?;
                append_reference_to_owner_bible(
                    transaction,
                    &record.reference.owner_kind,
                    &record.reference.owner_id,
                    &record.reference_entry,
                )?;

                let asset = transaction.query_row(
                    r#"
                    SELECT asset_id, kind, relative_path, source_kind, mime_type, size_bytes,
                           checksum, is_builtin, lifecycle, metadata_json, created_at, updated_at
                    FROM assets
                    WHERE asset_id = ?1
                    "#,
                    [record.asset.asset_id.as_str()],
                    asset_from_row,
                )?;
                let reference = transaction.query_row(
                    r#"
                    SELECT reference_id, asset_id, owner_kind, owner_id, usage_kind, created_at
                    FROM asset_references
                    WHERE reference_id = ?1
                    "#,
                    [record.reference.reference_id.as_str()],
                    reference_from_row,
                )?;
                Ok((asset, reference))
            })
            .map_err(|error| error.to_string())
    }

    pub fn list_references(&self, asset_id: &str) -> Result<Vec<AssetReferenceDto>, String> {
        self.database
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    r#"
                    SELECT reference_id, asset_id, owner_kind, owner_id, usage_kind, created_at
                    FROM asset_references
                    WHERE asset_id = ?1
                    ORDER BY created_at ASC
                    "#,
                )?;
                let rows = statement.query_map([asset_id], reference_from_row)?;
                rows.collect()
            })
            .map_err(|error| error.to_string())
    }

    pub fn get_reference(&self, reference_id: &str) -> Result<Option<AssetReferenceDto>, String> {
        self.database
            .with_connection(|connection| {
                connection
                    .query_row(
                        r#"
                        SELECT reference_id, asset_id, owner_kind, owner_id, usage_kind, created_at
                        FROM asset_references
                        WHERE reference_id = ?1
                        "#,
                        [reference_id],
                        reference_from_row,
                    )
                    .optional()
            })
            .map_err(|error| error.to_string())
    }

    pub fn delete_reference(&self, reference_id: &str) -> Result<AssetReferenceDto, String> {
        let reference = self
            .get_reference(reference_id)?
            .ok_or_else(|| format!("Asset reference not found: {reference_id}"))?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    "DELETE FROM asset_references WHERE reference_id = ?1",
                    [reference_id],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;
        Ok(reference)
    }

    pub fn count_references(&self, asset_id: &str) -> Result<i64, String> {
        self.database
            .with_connection(|connection| {
                connection.query_row(
                    "SELECT COUNT(*) FROM asset_references WHERE asset_id = ?1",
                    [asset_id],
                    |row| row.get(0),
                )
            })
            .map_err(|error| error.to_string())
    }

    pub fn collect_project_asset_paths(&self, project_id: &str) -> Result<Vec<String>, String> {
        self.database
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    r#"
                    SELECT DISTINCT assets.relative_path
                    FROM assets
                    JOIN asset_references ON asset_references.asset_id = assets.asset_id
                    WHERE (asset_references.owner_kind = 'project' AND asset_references.owner_id = ?1)
                       OR asset_references.owner_id IN (
                           SELECT item_id FROM storyboard_items WHERE project_id = ?1
                       )
                       OR asset_references.owner_id IN (
                           SELECT character_bible_id FROM character_bibles WHERE project_id = ?1
                       )
                       OR asset_references.owner_id IN (
                           SELECT style_bible_id FROM style_bibles WHERE project_id = ?1
                       )
                       OR asset_references.owner_id IN (
                           SELECT location_bible_id FROM location_bibles WHERE project_id = ?1
                       )
                       OR asset_references.owner_id IN (
                           SELECT task_id FROM tasks WHERE project_id = ?1
                       )
                       OR (
                           asset_references.owner_kind = 'video_pack'
                           AND asset_references.owner_id LIKE ?1 || ':%'
                       )
                    ORDER BY assets.relative_path ASC
                    "#,
                )?;
                let rows = statement.query_map([project_id], |row| row.get::<_, String>(0))?;
                rows.collect()
            })
            .map_err(|error| error.to_string())
    }
}

impl Repository for AssetRepository<'_> {
    fn database(&self) -> &Database {
        self.database
    }
}

fn asset_from_row(row: &Row<'_>) -> Result<AssetDto, rusqlite::Error> {
    let metadata_json: String = row.get(9)?;
    Ok(AssetDto {
        asset_id: row.get(0)?,
        kind: row.get(1)?,
        relative_path: row.get(2)?,
        source_kind: row.get(3)?,
        mime_type: row.get(4)?,
        size_bytes: row.get(5)?,
        checksum: row.get(6)?,
        is_builtin: int_to_bool(row.get(7)?),
        lifecycle: row.get(8)?,
        metadata: parse_json(&metadata_json),
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

fn reference_from_row(row: &Row<'_>) -> Result<AssetReferenceDto, rusqlite::Error> {
    Ok(AssetReferenceDto {
        reference_id: row.get(0)?,
        asset_id: row.get(1)?,
        owner_kind: row.get(2)?,
        owner_id: row.get(3)?,
        usage_kind: row.get(4)?,
        created_at: row.get(5)?,
    })
}

fn parse_json(value: &str) -> Value {
    serde_json::from_str(value).unwrap_or_else(|_| json!({}))
}

fn ensure_owner_belongs_to_project(
    transaction: &rusqlite::Transaction<'_>,
    project_id: &str,
    owner_kind: &str,
    owner_id: &str,
) -> Result<(), rusqlite::Error> {
    let exists = match owner_kind {
        "project" => {
            if owner_id != project_id {
                false
            } else {
                row_exists(
                    transaction,
                    "SELECT 1 FROM projects WHERE project_id = ?1",
                    [project_id],
                )?
            }
        }
        "storyboard_item" => row_exists(
            transaction,
            "SELECT 1 FROM storyboard_items WHERE item_id = ?1 AND project_id = ?2",
            (owner_id, project_id),
        )?,
        "character_bible" => row_exists(
            transaction,
            "SELECT 1 FROM character_bibles WHERE character_bible_id = ?1 AND project_id = ?2",
            (owner_id, project_id),
        )?,
        "style_bible" => row_exists(
            transaction,
            "SELECT 1 FROM style_bibles WHERE style_bible_id = ?1 AND project_id = ?2",
            (owner_id, project_id),
        )?,
        "location_bible" => row_exists(
            transaction,
            "SELECT 1 FROM location_bibles WHERE location_bible_id = ?1 AND project_id = ?2",
            (owner_id, project_id),
        )?,
        _ => false,
    };

    if exists {
        Ok(())
    } else {
        Err(sql_error(format!(
            "Asset owner {owner_kind}/{owner_id} does not belong to project {project_id}."
        )))
    }
}

fn append_reference_to_owner_bible(
    transaction: &rusqlite::Transaction<'_>,
    owner_kind: &str,
    owner_id: &str,
    reference_entry: &Value,
) -> Result<(), rusqlite::Error> {
    match owner_kind {
        "character_bible" => update_bible_reference_images(
            transaction,
            "character_bibles",
            "character_bible_id",
            owner_id,
            reference_entry,
            false,
        ),
        "location_bible" => update_bible_reference_images(
            transaction,
            "location_bibles",
            "location_bible_id",
            owner_id,
            reference_entry,
            false,
        ),
        "style_bible" => update_bible_reference_images(
            transaction,
            "style_bibles",
            "style_bible_id",
            owner_id,
            reference_entry,
            true,
        ),
        _ => Ok(()),
    }
}

fn update_bible_reference_images(
    transaction: &rusqlite::Transaction<'_>,
    table: &str,
    id_column: &str,
    owner_id: &str,
    reference_entry: &Value,
    set_primary_path: bool,
) -> Result<(), rusqlite::Error> {
    let data_json: String = transaction.query_row(
        &format!("SELECT data_json FROM {table} WHERE {id_column} = ?1"),
        [owner_id],
        |row| row.get(0),
    )?;
    let mut data = parse_json(&data_json);
    if !data.is_object() {
        data = json!({});
    }

    let relative_path = reference_entry
        .get("relativePath")
        .or_else(|| reference_entry.get("relative_path"))
        .and_then(Value::as_str)
        .map(str::to_string);

    let object = data
        .as_object_mut()
        .ok_or_else(|| sql_error("Bible data_json must be an object.".to_string()))?;
    let reference_images = object
        .entry("reference_images_json".to_string())
        .or_insert_with(|| json!([]));
    if !reference_images.is_array() {
        *reference_images = json!([]);
    }
    reference_images
        .as_array_mut()
        .ok_or_else(|| sql_error("Bible reference_images_json must be an array.".to_string()))?
        .push(reference_entry.clone());

    let reference_images_compat = object
        .entry("reference_images".to_string())
        .or_insert_with(|| json!([]));
    if !reference_images_compat.is_array() {
        *reference_images_compat = json!([]);
    }
    reference_images_compat
        .as_array_mut()
        .ok_or_else(|| sql_error("Bible reference_images must be an array.".to_string()))?
        .push(reference_entry.clone());

    if set_primary_path {
        if let Some(relative_path) = relative_path {
            object.insert(
                "reference_image_path".to_string(),
                Value::String(relative_path),
            );
        }
    }

    let next_json = serde_json::to_string(&data).map_err(json_to_sql_error)?;
    transaction.execute(
        &format!("UPDATE {table} SET data_json = ?1, updated_at = CURRENT_TIMESTAMP WHERE {id_column} = ?2"),
        params![next_json, owner_id],
    )?;
    Ok(())
}

fn row_exists<P: rusqlite::Params>(
    transaction: &rusqlite::Transaction<'_>,
    sql: &str,
    params: P,
) -> Result<bool, rusqlite::Error> {
    transaction
        .query_row(sql, params, |_| Ok(()))
        .optional()
        .map(|value| value.is_some())
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

fn json_to_sql_error(error: serde_json::Error) -> rusqlite::Error {
    rusqlite::Error::ToSqlConversionFailure(Box::new(error))
}

fn sql_error(message: String) -> rusqlite::Error {
    rusqlite::Error::ToSqlConversionFailure(Box::new(SimpleSqlError(message)))
}

#[derive(Debug)]
struct SimpleSqlError(String);

impl std::fmt::Display for SimpleSqlError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for SimpleSqlError {}
