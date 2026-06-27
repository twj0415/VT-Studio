use crate::db::{Database, Repository};
use crate::domain::character::CharacterBibleDto;
use rusqlite::{params, OptionalExtension, Row};
use serde_json::{json, Value};

pub struct CharacterRepository<'db> {
    database: &'db Database,
}

#[derive(Debug, Clone)]
pub struct CharacterBibleRecordInput {
    pub character_bible_id: String,
    pub project_id: String,
    pub name: String,
    pub data: Value,
}

impl<'db> CharacterRepository<'db> {
    pub fn new(database: &'db Database) -> Self {
        Self { database }
    }

    pub fn list_project_character_bibles(
        &self,
        project_id: &str,
    ) -> Result<Vec<CharacterBibleDto>, String> {
        self.database
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    r#"
                    SELECT character_bible_id, project_id, name, data_json, created_at, updated_at
                    FROM character_bibles
                    WHERE project_id = ?1
                    ORDER BY created_at ASC, character_bible_id ASC
                    "#,
                )?;
                let rows = statement.query_map([project_id], row_to_character_bible)?;
                rows.collect()
            })
            .map_err(|error| error.to_string())
    }

    pub fn get_character_bible(
        &self,
        character_id: &str,
    ) -> Result<Option<CharacterBibleDto>, String> {
        self.database
            .with_connection(|connection| {
                connection
                    .query_row(
                        r#"
                        SELECT character_bible_id, project_id, name, data_json, created_at, updated_at
                        FROM character_bibles
                        WHERE character_bible_id = ?1
                        "#,
                        [character_id],
                        row_to_character_bible,
                    )
                    .optional()
            })
            .map_err(|error| error.to_string())
    }

    pub fn get_project_character_bible(
        &self,
        project_id: &str,
        character_id: &str,
    ) -> Result<Option<CharacterBibleDto>, String> {
        self.database
            .with_connection(|connection| {
                connection
                    .query_row(
                        r#"
                        SELECT character_bible_id, project_id, name, data_json, created_at, updated_at
                        FROM character_bibles
                        WHERE character_bible_id = ?1 AND project_id = ?2
                        "#,
                        params![character_id, project_id],
                        row_to_character_bible,
                    )
                    .optional()
            })
            .map_err(|error| error.to_string())
    }

    pub fn upsert_character_bible(
        &self,
        input: CharacterBibleRecordInput,
    ) -> Result<CharacterBibleDto, String> {
        let data = normalize_character_data(input.data);
        let data_json = serde_json::to_string(&data).map_err(|error| error.to_string())?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    INSERT INTO character_bibles (
                        character_bible_id, project_id, name, data_json, updated_at
                    )
                    VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)
                    ON CONFLICT(character_bible_id) DO UPDATE SET
                        project_id = excluded.project_id,
                        name = excluded.name,
                        data_json = excluded.data_json,
                        updated_at = CURRENT_TIMESTAMP
                    "#,
                    params![
                        input.character_bible_id,
                        input.project_id,
                        input.name,
                        data_json,
                    ],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get_character_bible(&input.character_bible_id)?
            .ok_or_else(|| {
                format!(
                    "Character Bible was saved but cannot be read: {}",
                    input.character_bible_id
                )
            })
    }

    pub fn delete_character_bible(&self, character_id: &str) -> Result<CharacterBibleDto, String> {
        let character = self
            .get_character_bible(character_id)?
            .ok_or_else(|| format!("Character Bible not found: {character_id}"))?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    "DELETE FROM character_bibles WHERE character_bible_id = ?1",
                    [character_id],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;
        Ok(character)
    }

    pub fn count_storyboard_references(&self, character_id: &str) -> Result<i64, String> {
        self.database
            .with_connection(|connection| {
                connection.query_row(
                    r#"
                    SELECT COUNT(*)
                    FROM storyboard_items
                    WHERE EXISTS (
                        SELECT 1
                        FROM json_each(storyboard_items.character_ids_json)
                        WHERE json_each.value = ?1
                    )
                    "#,
                    [character_id],
                    |row| row.get(0),
                )
            })
            .map_err(|error| error.to_string())
    }

    pub fn count_asset_references(&self, character_id: &str) -> Result<i64, String> {
        self.database
            .with_connection(|connection| {
                connection.query_row(
                    "SELECT COUNT(*) FROM asset_references WHERE owner_kind = 'character_bible' AND owner_id = ?1",
                    [character_id],
                    |row| row.get(0),
                )
            })
            .map_err(|error| error.to_string())
    }

    pub fn find_missing_character_ids(
        &self,
        project_id: &str,
        character_ids: &[String],
    ) -> Result<Vec<String>, String> {
        let mut missing = Vec::new();
        for character_id in character_ids {
            if self
                .get_project_character_bible(project_id, character_id)?
                .is_none()
            {
                missing.push(character_id.clone());
            }
        }
        Ok(missing)
    }
}

impl Repository for CharacterRepository<'_> {
    fn database(&self) -> &Database {
        self.database
    }
}

fn row_to_character_bible(row: &Row<'_>) -> Result<CharacterBibleDto, rusqlite::Error> {
    let character_bible_id: String = row.get(0)?;
    let data_json: String = row.get(3)?;
    let mut data = normalize_character_data(parse_json(&data_json));
    if let Some(object) = data.as_object_mut() {
        object.insert(
            "character_id".to_string(),
            Value::String(character_bible_id.clone()),
        );
    }

    Ok(CharacterBibleDto {
        character_bible_id: character_bible_id.clone(),
        project_id: row.get(1)?,
        name: row.get(2)?,
        character_id: character_bible_id,
        alias: read_string_array(&data, &["alias"]).unwrap_or_default(),
        age: read_string(&data, &["age"]).unwrap_or_default(),
        gender: read_string(&data, &["gender"]).unwrap_or_default(),
        appearance: read_string(&data, &["appearance", "face", "body"]).unwrap_or_default(),
        clothing: read_string(&data, &["clothing"]).unwrap_or_default(),
        personality: read_string(&data, &["personality"]).unwrap_or_default(),
        visual_prompt: read_string(&data, &["visual_prompt", "visualPrompt"]).unwrap_or_default(),
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
        lock_flags: data
            .get("lock_flags")
            .or_else(|| data.get("lockFlags"))
            .cloned()
            .unwrap_or_else(|| json!({})),
        data,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

pub fn normalize_character_data(data: Value) -> Value {
    let mut data = if data.is_object() { data } else { json!({}) };
    if let Some(object) = data.as_object_mut() {
        normalize_alias(object, "characterId", "character_id");
        normalize_alias(object, "visualPrompt", "visual_prompt");
        normalize_alias(object, "negativePrompt", "negative_prompt");
        normalize_alias(object, "referenceImagePath", "reference_image_path");
        normalize_alias(object, "referenceImagesJson", "reference_images_json");
        normalize_alias(object, "referenceImages", "reference_images");
        normalize_alias(object, "lockFlags", "lock_flags");
        if !object.contains_key("alias") {
            object.insert("alias".to_string(), json!([]));
        }
        if !object.contains_key("reference_images_json") {
            object.insert("reference_images_json".to_string(), json!([]));
        }
        if !object.contains_key("reference_images") {
            object.insert("reference_images".to_string(), json!([]));
        }
        if !object.contains_key("lock_flags") {
            object.insert("lock_flags".to_string(), json!({}));
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
        .map(str::trim)
        .filter(|value| !value.is_empty())
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
