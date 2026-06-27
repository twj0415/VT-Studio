use crate::db::{Database, Repository};
use crate::domain::style::{StyleBibleDto, StylePresetDto};
use rusqlite::{params, OptionalExtension, Row};
use serde_json::{json, Value};

const STYLE_PRESETS_CONFIG_KEY: &str = "style_presets";

pub struct StyleRepository<'db> {
    database: &'db Database,
}

#[derive(Debug, Clone)]
pub struct StyleBibleRecordInput {
    pub style_bible_id: String,
    pub project_id: String,
    pub name: String,
    pub data: Value,
}

impl<'db> StyleRepository<'db> {
    pub fn new(database: &'db Database) -> Self {
        Self { database }
    }

    pub fn get_project_style_bible(
        &self,
        project_id: &str,
    ) -> Result<Option<StyleBibleDto>, String> {
        self.database
            .with_connection(|connection| {
                connection
                    .query_row(
                        r#"
                        SELECT style_bible_id, project_id, name, data_json, created_at, updated_at
                        FROM style_bibles
                        WHERE project_id = ?1
                        ORDER BY created_at ASC
                        LIMIT 1
                        "#,
                        [project_id],
                        row_to_style_bible,
                    )
                    .optional()
            })
            .map_err(|error| error.to_string())
    }

    pub fn get_style_bible(&self, style_bible_id: &str) -> Result<Option<StyleBibleDto>, String> {
        self.database
            .with_connection(|connection| {
                connection
                    .query_row(
                        r#"
                        SELECT style_bible_id, project_id, name, data_json, created_at, updated_at
                        FROM style_bibles
                        WHERE style_bible_id = ?1
                        "#,
                        [style_bible_id],
                        row_to_style_bible,
                    )
                    .optional()
            })
            .map_err(|error| error.to_string())
    }

    pub fn upsert_style_bible(
        &self,
        input: StyleBibleRecordInput,
    ) -> Result<StyleBibleDto, String> {
        let data_json = serde_json::to_string(&input.data).map_err(|error| error.to_string())?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    INSERT INTO style_bibles (style_bible_id, project_id, name, data_json, updated_at)
                    VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)
                    ON CONFLICT(style_bible_id) DO UPDATE SET
                        project_id = excluded.project_id,
                        name = excluded.name,
                        data_json = excluded.data_json,
                        updated_at = CURRENT_TIMESTAMP
                    "#,
                    params![
                        input.style_bible_id,
                        input.project_id,
                        input.name,
                        data_json,
                    ],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get_style_bible(&input.style_bible_id)?.ok_or_else(|| {
            format!(
                "Style Bible was saved but cannot be read: {}",
                input.style_bible_id
            )
        })
    }

    pub fn list_user_style_presets(&self) -> Result<Vec<StylePresetDto>, String> {
        let value = self
            .database
            .with_connection(|connection| {
                connection
                    .query_row(
                        "SELECT config_json FROM app_configs WHERE config_key = ?1",
                        [STYLE_PRESETS_CONFIG_KEY],
                        |row| row.get::<_, String>(0),
                    )
                    .optional()
            })
            .map_err(|error| error.to_string())?;
        let Some(value) = value else {
            return Ok(vec![]);
        };
        let parsed = parse_json(&value);
        let presets = parsed
            .get("items")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| serde_json::from_value::<StylePresetDto>(value).ok())
            .collect::<Vec<_>>();
        Ok(presets)
    }

    pub fn upsert_user_style_preset(&self, preset: StylePresetDto) -> Result<(), String> {
        let mut presets = self.list_user_style_presets()?;
        presets.retain(|item| item.preset_id != preset.preset_id);
        presets.push(preset);
        presets.sort_by(|left, right| left.name.cmp(&right.name));
        let value = json!({ "items": presets });
        let config_json = serde_json::to_string(&value).map_err(|error| error.to_string())?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    INSERT INTO app_configs (config_key, config_json, schema_version, updated_at)
                    VALUES (?1, ?2, 1, CURRENT_TIMESTAMP)
                    ON CONFLICT(config_key) DO UPDATE SET
                        config_json = excluded.config_json,
                        schema_version = excluded.schema_version,
                        updated_at = CURRENT_TIMESTAMP
                    "#,
                    params![STYLE_PRESETS_CONFIG_KEY, config_json],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())
    }
}

impl Repository for StyleRepository<'_> {
    fn database(&self) -> &Database {
        self.database
    }
}

fn row_to_style_bible(row: &Row<'_>) -> Result<StyleBibleDto, rusqlite::Error> {
    let data_json: String = row.get(3)?;
    let data = normalize_style_data(parse_json(&data_json));
    Ok(StyleBibleDto {
        style_bible_id: row.get(0)?,
        project_id: row.get(1)?,
        name: row.get(2)?,
        style_prompt: read_string(&data, &["style_prompt", "stylePrompt"]).unwrap_or_default(),
        color_palette: read_string_array(&data, &["color_palette", "colorPalette"])
            .unwrap_or_default(),
        lighting: read_string(&data, &["lighting"]).unwrap_or_default(),
        composition: read_string(&data, &["composition"]).unwrap_or_default(),
        negative_prompt: read_string(&data, &["negative_prompt", "negativePrompt"])
            .unwrap_or_default(),
        reference_image_path: read_string(&data, &["reference_image_path", "referenceImagePath"]),
        reference_images: read_value_array(
            &data,
            &[
                "reference_images_json",
                "referenceImagesJson",
                "reference_images",
                "referenceImages",
            ],
        )
        .unwrap_or_default(),
        data,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

pub fn normalize_style_data(data: Value) -> Value {
    let mut data = if data.is_object() { data } else { json!({}) };
    if let Some(object) = data.as_object_mut() {
        normalize_alias(object, "stylePrompt", "style_prompt");
        normalize_alias(object, "colorPalette", "color_palette");
        normalize_alias(object, "negativePrompt", "negative_prompt");
        normalize_alias(object, "referenceImagePath", "reference_image_path");
        normalize_alias(object, "referenceImagesJson", "reference_images_json");
        normalize_alias(object, "referenceImages", "reference_images");
        if !object.contains_key("reference_images_json") {
            object.insert("reference_images_json".to_string(), json!([]));
        }
        if !object.contains_key("reference_images") {
            object.insert("reference_images".to_string(), json!([]));
        }
    }
    data
}

fn normalize_alias(object: &mut serde_json::Map<String, Value>, from_key: &str, to_key: &str) {
    if object.contains_key(to_key) {
        return;
    }
    if let Some(value) = object.remove(from_key) {
        object.insert(to_key.to_string(), value);
    }
}

fn read_string(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
        .map(str::to_string)
}

fn read_string_array(value: &Value, keys: &[&str]) -> Option<Vec<String>> {
    keys.iter().find_map(|key| {
        value.get(*key).and_then(Value::as_array).map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
    })
}

fn read_value_array(value: &Value, keys: &[&str]) -> Option<Vec<Value>> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_array).cloned())
}

fn parse_json(value: &str) -> Value {
    serde_json::from_str(value).unwrap_or_else(|_| json!({}))
}
