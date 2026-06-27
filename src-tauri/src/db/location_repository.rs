use crate::db::{Database, Repository};
use crate::domain::location::LocationBibleDto;
use rusqlite::{params, OptionalExtension, Row};
use serde_json::{json, Value};

pub struct LocationRepository<'db> {
    database: &'db Database,
}

#[derive(Debug, Clone)]
pub struct LocationBibleRecordInput {
    pub location_bible_id: String,
    pub project_id: String,
    pub name: String,
    pub data: Value,
}

impl<'db> LocationRepository<'db> {
    pub fn new(database: &'db Database) -> Self {
        Self { database }
    }

    pub fn list_project_location_bibles(
        &self,
        project_id: &str,
    ) -> Result<Vec<LocationBibleDto>, String> {
        self.database
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    r#"
                    SELECT location_bible_id, project_id, name, data_json, created_at, updated_at
                    FROM location_bibles
                    WHERE project_id = ?1
                    ORDER BY created_at ASC, location_bible_id ASC
                    "#,
                )?;
                let rows = statement.query_map([project_id], row_to_location_bible)?;
                rows.collect()
            })
            .map_err(|error| error.to_string())
    }

    pub fn get_location_bible(
        &self,
        location_id: &str,
    ) -> Result<Option<LocationBibleDto>, String> {
        self.database
            .with_connection(|connection| {
                connection
                    .query_row(
                        r#"
                        SELECT location_bible_id, project_id, name, data_json, created_at, updated_at
                        FROM location_bibles
                        WHERE location_bible_id = ?1
                        "#,
                        [location_id],
                        row_to_location_bible,
                    )
                    .optional()
            })
            .map_err(|error| error.to_string())
    }

    pub fn get_project_location_bible(
        &self,
        project_id: &str,
        location_id: &str,
    ) -> Result<Option<LocationBibleDto>, String> {
        self.database
            .with_connection(|connection| {
                connection
                    .query_row(
                        r#"
                        SELECT location_bible_id, project_id, name, data_json, created_at, updated_at
                        FROM location_bibles
                        WHERE location_bible_id = ?1 AND project_id = ?2
                        "#,
                        params![location_id, project_id],
                        row_to_location_bible,
                    )
                    .optional()
            })
            .map_err(|error| error.to_string())
    }

    pub fn upsert_location_bible(
        &self,
        input: LocationBibleRecordInput,
    ) -> Result<LocationBibleDto, String> {
        let data = normalize_location_data(input.data);
        let data_json = serde_json::to_string(&data).map_err(|error| error.to_string())?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    INSERT INTO location_bibles (
                        location_bible_id, project_id, name, data_json, updated_at
                    )
                    VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)
                    ON CONFLICT(location_bible_id) DO UPDATE SET
                        project_id = excluded.project_id,
                        name = excluded.name,
                        data_json = excluded.data_json,
                        updated_at = CURRENT_TIMESTAMP
                    "#,
                    params![
                        input.location_bible_id,
                        input.project_id,
                        input.name,
                        data_json
                    ],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get_location_bible(&input.location_bible_id)?
            .ok_or_else(|| {
                format!(
                    "Location Bible was saved but cannot be read: {}",
                    input.location_bible_id
                )
            })
    }

    pub fn delete_location_bible(&self, location_id: &str) -> Result<LocationBibleDto, String> {
        let location = self
            .get_location_bible(location_id)?
            .ok_or_else(|| format!("Location Bible not found: {location_id}"))?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    "DELETE FROM location_bibles WHERE location_bible_id = ?1",
                    [location_id],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;
        Ok(location)
    }

    pub fn count_storyboard_references(&self, location_id: &str) -> Result<i64, String> {
        self.database
            .with_connection(|connection| {
                connection.query_row(
                    "SELECT COUNT(*) FROM storyboard_items WHERE location_id = ?1",
                    [location_id],
                    |row| row.get(0),
                )
            })
            .map_err(|error| error.to_string())
    }

    pub fn count_asset_references(&self, location_id: &str) -> Result<i64, String> {
        self.database
            .with_connection(|connection| {
                connection.query_row(
                    "SELECT COUNT(*) FROM asset_references WHERE owner_kind = 'location_bible' AND owner_id = ?1",
                    [location_id],
                    |row| row.get(0),
                )
            })
            .map_err(|error| error.to_string())
    }
}

impl Repository for LocationRepository<'_> {
    fn database(&self) -> &Database {
        self.database
    }
}

fn row_to_location_bible(row: &Row<'_>) -> Result<LocationBibleDto, rusqlite::Error> {
    let location_bible_id: String = row.get(0)?;
    let data_json: String = row.get(3)?;
    let mut data = normalize_location_data(parse_json(&data_json));
    if let Some(object) = data.as_object_mut() {
        object.insert(
            "location_id".to_string(),
            Value::String(location_bible_id.clone()),
        );
    }

    Ok(LocationBibleDto {
        location_bible_id: location_bible_id.clone(),
        project_id: row.get(1)?,
        name: row.get(2)?,
        location_id: location_bible_id,
        space_description: read_string(
            &data,
            &["space_description", "spaceDescription", "description"],
        )
        .unwrap_or_default(),
        lighting: read_string(&data, &["lighting"]).unwrap_or_default(),
        time_of_day: read_string(&data, &["time_of_day", "timeOfDay"]).unwrap_or_default(),
        props: read_string_array(&data, &["props"]).unwrap_or_default(),
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
        variants: read_value_array(&data, &["variants"]).unwrap_or_default(),
        data,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

pub fn normalize_location_data(data: Value) -> Value {
    let mut data = if data.is_object() { data } else { json!({}) };
    if let Some(object) = data.as_object_mut() {
        normalize_alias(object, "locationId", "location_id");
        normalize_alias(object, "spaceDescription", "space_description");
        normalize_alias(object, "timeOfDay", "time_of_day");
        normalize_alias(object, "visualPrompt", "visual_prompt");
        normalize_alias(object, "negativePrompt", "negative_prompt");
        normalize_alias(object, "referenceImagePath", "reference_image_path");
        normalize_alias(object, "referenceImagesJson", "reference_images_json");
        normalize_alias(object, "referenceImages", "reference_images");
        if !object.contains_key("props") {
            object.insert("props".to_string(), json!([]));
        }
        if !object.contains_key("reference_images_json") {
            object.insert("reference_images_json".to_string(), json!([]));
        }
        if !object.contains_key("reference_images") {
            object.insert("reference_images".to_string(), json!([]));
        }
        if !object.contains_key("variants") {
            object.insert("variants".to_string(), json!([]));
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
