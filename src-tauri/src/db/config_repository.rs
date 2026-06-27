use crate::db::{Database, Repository};
use rusqlite::params;
use serde_json::{json, Value};

pub struct ConfigRepository<'db> {
    database: &'db Database,
}

impl<'db> ConfigRepository<'db> {
    pub fn new(database: &'db Database) -> Self {
        Self { database }
    }

    pub fn get_config(&self, config_key: &str) -> Result<Option<Value>, String> {
        self.database
            .with_connection(|connection| {
                let result = connection.query_row(
                    "SELECT config_json FROM app_configs WHERE config_key = ?1",
                    [config_key],
                    |row| row.get::<_, String>(0),
                );

                match result {
                    Ok(config_json) => serde_json::from_str(&config_json)
                        .map(Some)
                        .map_err(|_| rusqlite::Error::InvalidQuery),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(error) => Err(error),
                }
            })
            .map_err(|error| error.to_string())
    }

    pub fn upsert_config(
        &self,
        config_key: &str,
        config_json: &Value,
        schema_version: i64,
    ) -> Result<(), String> {
        let config_json = serde_json::to_string(config_json).map_err(|error| error.to_string())?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    INSERT INTO app_configs (config_key, config_json, schema_version, updated_at)
                    VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)
                    ON CONFLICT(config_key) DO UPDATE SET
                        config_json = excluded.config_json,
                        schema_version = excluded.schema_version,
                        updated_at = CURRENT_TIMESTAMP
                    "#,
                    params![config_key, config_json, schema_version],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())
    }

    pub fn ensure_defaults(&self) -> Result<(), String> {
        if self.get_config("app")?.is_none() {
            self.upsert_config(
                "app",
                &json!({
                    "app_locale": "zh-CN",
                    "theme_preset": "graphite",
                    "layout_density": "comfortable"
                }),
                1,
            )?;
        }

        if self.get_config("pipeline")?.is_none() {
            self.upsert_config(
                "pipeline",
                &json!({
                    "default_content_category": "knowledge",
                    "default_aspect_ratio": "vertical_9_16",
                    "default_content_language": "zh-CN",
                    "default_target_duration_seconds": 60,
                    "default_target_scene_count": 8,
                    "max_concurrent_provider_calls": 2,
                    "max_concurrent_ffmpeg_jobs": 2,
                    "retry_max_attempts": 3,
                    "retry_backoff_seconds": [2, 5, 10],
                    "image_prompt_batch_size": 10,
                    "max_prompt_length": 1000
                }),
                1,
            )?;
        }

        if self.get_config("ui")?.is_none() {
            self.upsert_config(
                "ui",
                &json!({
                    "show_advanced_options": false,
                    "auto_open_task_detail": true,
                    "confirm_before_costly_generation": true,
                    "default_project_view": "grid",
                    "timeline_density": "comfortable"
                }),
                1,
            )?;
        }

        if self.get_config("export")?.is_none() {
            self.upsert_config(
                "export",
                &json!({
                    "format": "mp4",
                    "fps": 30,
                    "video_codec": "libx264",
                    "audio_codec": "aac",
                    "audio_bitrate": "192k",
                    "crf": 18,
                    "preset": "veryfast",
                    "pix_fmt": "yuv420p",
                    "include_cover": true,
                    "include_subtitles_json": true
                }),
                1,
            )?;
        }

        Ok(())
    }
}

impl Repository for ConfigRepository<'_> {
    fn database(&self) -> &Database {
        self.database
    }
}
