use crate::db::{Database, Repository};
use crate::security::secret_guard::reject_json_secrets;
use rusqlite::{params, Row};
use serde_json::{json, Value};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ProviderRecord {
    pub provider_id: String,
    pub vendor: String,
    pub kind: String,
    pub display_name: String,
    pub auth_type: String,
    pub key_alias: Option<String>,
    pub base_url: Option<String>,
    pub status: String,
    pub enabled: bool,
    pub config_json: Value,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ProviderModelRecord {
    pub model_id: String,
    pub provider_id: String,
    pub provider_model_id: String,
    pub display_name: String,
    pub capability: String,
    pub config_json: Value,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WorkflowPresetRecord {
    pub preset_id: String,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub name: String,
    pub kind: String,
    pub capability: String,
    pub config_json: Value,
    pub enabled: bool,
}

#[allow(dead_code)]
pub struct ProviderRepository<'db> {
    database: &'db Database,
}

#[allow(dead_code)]
impl<'db> ProviderRepository<'db> {
    pub fn new(database: &'db Database) -> Self {
        Self { database }
    }

    pub fn upsert_provider(&self, provider: &ProviderRecord) -> Result<(), String> {
        reject_json_secrets(&provider.config_json)?;
        let config_json =
            serde_json::to_string(&provider.config_json).map_err(|error| error.to_string())?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    INSERT INTO providers (
                        provider_id, vendor, kind, display_name, auth_type, key_alias,
                        base_url, status, enabled, config_json, updated_at
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, CURRENT_TIMESTAMP)
                    ON CONFLICT(provider_id) DO UPDATE SET
                        vendor = excluded.vendor,
                        kind = excluded.kind,
                        display_name = excluded.display_name,
                        auth_type = excluded.auth_type,
                        key_alias = excluded.key_alias,
                        base_url = excluded.base_url,
                        status = excluded.status,
                        enabled = excluded.enabled,
                        config_json = excluded.config_json,
                        updated_at = CURRENT_TIMESTAMP
                    "#,
                    params![
                        provider.provider_id,
                        provider.vendor,
                        provider.kind,
                        provider.display_name,
                        provider.auth_type,
                        provider.key_alias,
                        provider.base_url,
                        provider.status,
                        bool_to_i64(provider.enabled),
                        config_json,
                    ],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())
    }

    pub fn list_providers(&self) -> Result<Vec<ProviderRecord>, String> {
        self.database
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    r#"
                    SELECT provider_id, vendor, kind, display_name, auth_type, key_alias,
                           base_url, status, enabled, config_json
                    FROM providers
                    ORDER BY created_at ASC
                    "#,
                )?;
                let rows = statement.query_map([], provider_from_row)?;
                rows.collect()
            })
            .map_err(|error| error.to_string())
    }

    pub fn delete_provider(&self, provider_id: &str) -> Result<(), String> {
        self.database
            .with_connection(|connection| {
                connection.execute(
                    "DELETE FROM providers WHERE provider_id = ?1",
                    [provider_id],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())
    }

    pub fn upsert_provider_model(&self, model: &ProviderModelRecord) -> Result<(), String> {
        reject_json_secrets(&model.config_json)?;
        let config_json =
            serde_json::to_string(&model.config_json).map_err(|error| error.to_string())?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    INSERT INTO provider_models (
                        model_id, provider_id, provider_model_id, display_name,
                        capability, config_json, enabled, updated_at
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, CURRENT_TIMESTAMP)
                    ON CONFLICT(model_id) DO UPDATE SET
                        provider_id = excluded.provider_id,
                        provider_model_id = excluded.provider_model_id,
                        display_name = excluded.display_name,
                        capability = excluded.capability,
                        config_json = excluded.config_json,
                        enabled = excluded.enabled,
                        updated_at = CURRENT_TIMESTAMP
                    "#,
                    params![
                        model.model_id,
                        model.provider_id,
                        model.provider_model_id,
                        model.display_name,
                        model.capability,
                        config_json,
                        bool_to_i64(model.enabled),
                    ],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())
    }

    pub fn delete_provider_model(&self, model_id: &str) -> Result<(), String> {
        self.database
            .with_connection(|connection| {
                connection.execute(
                    "DELETE FROM provider_models WHERE model_id = ?1",
                    [model_id],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())
    }

    pub fn get_provider_model(
        &self,
        model_id: &str,
    ) -> Result<Option<ProviderModelRecord>, String> {
        self.database
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    r#"
                    SELECT model_id, provider_id, provider_model_id, display_name,
                           capability, config_json, enabled
                    FROM provider_models
                    WHERE model_id = ?1
                    "#,
                )?;
                match statement.query_row([model_id], provider_model_from_row) {
                    Ok(model) => Ok(Some(model)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(error) => Err(error),
                }
            })
            .map_err(|error| error.to_string())
    }

    pub fn list_provider_models(
        &self,
        provider_id: Option<&str>,
    ) -> Result<Vec<ProviderModelRecord>, String> {
        self.database
            .with_connection(|connection| {
                if let Some(provider_id) = provider_id {
                    let mut statement = connection.prepare(
                        r#"
                        SELECT model_id, provider_id, provider_model_id, display_name,
                               capability, config_json, enabled
                        FROM provider_models
                        WHERE provider_id = ?1
                        ORDER BY created_at ASC
                        "#,
                    )?;
                    let rows = statement.query_map([provider_id], provider_model_from_row)?;
                    return rows.collect();
                }

                let mut statement = connection.prepare(
                    r#"
                    SELECT model_id, provider_id, provider_model_id, display_name,
                           capability, config_json, enabled
                    FROM provider_models
                    ORDER BY created_at ASC
                    "#,
                )?;
                let rows = statement.query_map([], provider_model_from_row)?;
                rows.collect()
            })
            .map_err(|error| error.to_string())
    }

    pub fn upsert_workflow_preset(&self, preset: &WorkflowPresetRecord) -> Result<(), String> {
        reject_json_secrets(&preset.config_json)?;
        let config_json =
            serde_json::to_string(&preset.config_json).map_err(|error| error.to_string())?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    INSERT INTO workflow_presets (
                        preset_id, provider_id, model_id, name, kind, capability,
                        config_json, enabled, updated_at
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, CURRENT_TIMESTAMP)
                    ON CONFLICT(preset_id) DO UPDATE SET
                        provider_id = excluded.provider_id,
                        model_id = excluded.model_id,
                        name = excluded.name,
                        kind = excluded.kind,
                        capability = excluded.capability,
                        config_json = excluded.config_json,
                        enabled = excluded.enabled,
                        updated_at = CURRENT_TIMESTAMP
                    "#,
                    params![
                        preset.preset_id,
                        preset.provider_id,
                        preset.model_id,
                        preset.name,
                        preset.kind,
                        preset.capability,
                        config_json,
                        bool_to_i64(preset.enabled),
                    ],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())
    }

    pub fn delete_workflow_preset(&self, preset_id: &str) -> Result<(), String> {
        self.database
            .with_connection(|connection| {
                connection.execute(
                    "DELETE FROM workflow_presets WHERE preset_id = ?1",
                    [preset_id],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())
    }

    pub fn get_workflow_preset(
        &self,
        preset_id: &str,
    ) -> Result<Option<WorkflowPresetRecord>, String> {
        self.database
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    r#"
                    SELECT preset_id, provider_id, model_id, name, kind, capability,
                           config_json, enabled
                    FROM workflow_presets
                    WHERE preset_id = ?1
                    "#,
                )?;
                match statement.query_row([preset_id], workflow_preset_from_row) {
                    Ok(preset) => Ok(Some(preset)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(error) => Err(error),
                }
            })
            .map_err(|error| error.to_string())
    }

    pub fn list_workflow_presets(
        &self,
        capability: Option<&str>,
    ) -> Result<Vec<WorkflowPresetRecord>, String> {
        self.database
            .with_connection(|connection| {
                if let Some(capability) = capability {
                    let mut statement = connection.prepare(
                        r#"
                        SELECT preset_id, provider_id, model_id, name, kind, capability,
                               config_json, enabled
                        FROM workflow_presets
                        WHERE capability = ?1
                        ORDER BY created_at ASC
                        "#,
                    )?;
                    let rows = statement.query_map([capability], workflow_preset_from_row)?;
                    return rows.collect();
                }

                let mut statement = connection.prepare(
                    r#"
                    SELECT preset_id, provider_id, model_id, name, kind, capability,
                           config_json, enabled
                    FROM workflow_presets
                    ORDER BY created_at ASC
                    "#,
                )?;
                let rows = statement.query_map([], workflow_preset_from_row)?;
                rows.collect()
            })
            .map_err(|error| error.to_string())
    }

    pub fn list_workflow_presets_by_provider(
        &self,
        provider_id: Option<&str>,
    ) -> Result<Vec<WorkflowPresetRecord>, String> {
        self.database
            .with_connection(|connection| {
                if let Some(provider_id) = provider_id {
                    let mut statement = connection.prepare(
                        r#"
                        SELECT preset_id, provider_id, model_id, name, kind, capability,
                               config_json, enabled
                        FROM workflow_presets
                        WHERE provider_id = ?1
                        ORDER BY created_at ASC
                        "#,
                    )?;
                    let rows = statement.query_map([provider_id], workflow_preset_from_row)?;
                    return rows.collect();
                }

                let mut statement = connection.prepare(
                    r#"
                    SELECT preset_id, provider_id, model_id, name, kind, capability,
                           config_json, enabled
                    FROM workflow_presets
                    ORDER BY created_at ASC
                    "#,
                )?;
                let rows = statement.query_map([], workflow_preset_from_row)?;
                rows.collect()
            })
            .map_err(|error| error.to_string())
    }
}

impl Repository for ProviderRepository<'_> {
    fn database(&self) -> &Database {
        self.database
    }
}

fn provider_from_row(row: &Row<'_>) -> Result<ProviderRecord, rusqlite::Error> {
    let config_json: String = row.get(9)?;
    Ok(ProviderRecord {
        provider_id: row.get(0)?,
        vendor: row.get(1)?,
        kind: row.get(2)?,
        display_name: row.get(3)?,
        auth_type: row.get(4)?,
        key_alias: row.get(5)?,
        base_url: row.get(6)?,
        status: row.get(7)?,
        enabled: int_to_bool(row.get(8)?),
        config_json: parse_json(&config_json),
    })
}

fn provider_model_from_row(row: &Row<'_>) -> Result<ProviderModelRecord, rusqlite::Error> {
    let config_json: String = row.get(5)?;
    Ok(ProviderModelRecord {
        model_id: row.get(0)?,
        provider_id: row.get(1)?,
        provider_model_id: row.get(2)?,
        display_name: row.get(3)?,
        capability: row.get(4)?,
        config_json: parse_json(&config_json),
        enabled: int_to_bool(row.get(6)?),
    })
}

fn workflow_preset_from_row(row: &Row<'_>) -> Result<WorkflowPresetRecord, rusqlite::Error> {
    let config_json: String = row.get(6)?;
    Ok(WorkflowPresetRecord {
        preset_id: row.get(0)?,
        provider_id: row.get(1)?,
        model_id: row.get(2)?,
        name: row.get(3)?,
        kind: row.get(4)?,
        capability: row.get(5)?,
        config_json: parse_json(&config_json),
        enabled: int_to_bool(row.get(7)?),
    })
}

fn parse_json(value: &str) -> Value {
    serde_json::from_str(value).unwrap_or_else(|_| json!({}))
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
