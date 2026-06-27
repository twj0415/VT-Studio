use crate::db::{Database, Repository};
use crate::domain::media::MediaProbeDto;
use crate::domain::scene::{
    ImageCandidateDto, SceneDto, StoryboardDto, SubtitleChunkDto, VideoSegmentDto,
};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde_json::{json, Value};

pub struct SceneRepository<'db> {
    database: &'db Database,
}

#[derive(Debug, Clone)]
pub struct NewImageCandidateRecord {
    pub image_id: String,
    pub item_id: String,
    pub image_path: String,
    pub prompt: String,
    pub negative_prompt: String,
    pub model: String,
    pub provider_model_id: String,
    pub workflow_preset_id: Option<String>,
    pub status: String,
    pub selected: bool,
    pub derived_from_image_id: Option<String>,
    pub generation_context_snapshot: Value,
}

#[derive(Debug, Clone)]
pub struct NewVideoSegmentRecord {
    pub segment_id: String,
    pub item_id: String,
    pub input_image_id: String,
    pub video_path: String,
    pub video_prompt: String,
    pub duration_seconds: f64,
    pub model: String,
    pub provider_model_id: String,
    pub workflow_preset_id: Option<String>,
    pub status: String,
    pub selected: bool,
    pub generation_context_snapshot: Value,
}

impl<'db> SceneRepository<'db> {
    pub fn new(database: &'db Database) -> Self {
        Self { database }
    }

    pub fn get_storyboard(&self, project_id: &str) -> Result<Option<StoryboardDto>, String> {
        self.database
            .with_connection(|connection| read_storyboard(connection, project_id))
            .map_err(|error| error.to_string())
    }

    pub fn upsert_storyboard_item(&self, item: &SceneDto) -> Result<SceneDto, String> {
        let characters_json =
            serde_json::to_string(&item.characters).map_err(|error| error.to_string())?;
        let character_ids_json =
            serde_json::to_string(&item.character_ids).map_err(|error| error.to_string())?;
        let lock_flags_json =
            serde_json::to_string(&item.lock_flags_json).map_err(|error| error.to_string())?;
        let subtitle_chunks_json =
            serde_json::to_string(&item.subtitle_chunks).map_err(|error| error.to_string())?;
        let image_last_error_json = item
            .image_last_error_json
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| error.to_string())?;
        let audio_last_error_json = item
            .audio_last_error_json
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| error.to_string())?;
        let audio_probe_json = item
            .audio_probe
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| error.to_string())?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    INSERT INTO storyboard_items (
                        item_id, project_id, item_index, source_text, narration_text,
                        visual_goal, visual_description, characters_json, character_ids_json,
                        location_id, scene_description, image_prompt, negative_prompt,
                        video_prompt, duration_seconds, subtitle_chunks_json,
                        audio_path, audio_duration_seconds, audio_probe_json, selected_image_id,
                        selected_video_segment_id, status, lock_flags_json, shot_size,
                        camera_motion, composition, pace, transition_type, image_status,
                        image_last_error_json, image_retry_count, audio_status,
                        audio_last_error_json, audio_retry_count, video_status, subtitle_status,
                        render_status, segment_status, updated_at
                    )
                    VALUES (
                        ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                        ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24,
                        ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32, ?33, ?34, ?35,
                        ?36, ?37, ?38,
                        CURRENT_TIMESTAMP
                    )
                    ON CONFLICT(item_id) DO UPDATE SET
                        item_index = excluded.item_index,
                        source_text = excluded.source_text,
                        narration_text = excluded.narration_text,
                        visual_goal = excluded.visual_goal,
                        visual_description = excluded.visual_description,
                        characters_json = excluded.characters_json,
                        character_ids_json = excluded.character_ids_json,
                        location_id = excluded.location_id,
                        scene_description = excluded.scene_description,
                        image_prompt = excluded.image_prompt,
                        negative_prompt = excluded.negative_prompt,
                        video_prompt = excluded.video_prompt,
                        duration_seconds = excluded.duration_seconds,
                        subtitle_chunks_json = excluded.subtitle_chunks_json,
                        audio_path = excluded.audio_path,
                        audio_duration_seconds = excluded.audio_duration_seconds,
                        audio_probe_json = excluded.audio_probe_json,
                        selected_image_id = excluded.selected_image_id,
                        selected_video_segment_id = excluded.selected_video_segment_id,
                        status = excluded.status,
                        lock_flags_json = excluded.lock_flags_json,
                        shot_size = excluded.shot_size,
                        camera_motion = excluded.camera_motion,
                        composition = excluded.composition,
                        pace = excluded.pace,
                        transition_type = excluded.transition_type,
                        image_status = excluded.image_status,
                        image_last_error_json = excluded.image_last_error_json,
                        image_retry_count = excluded.image_retry_count,
                        audio_status = excluded.audio_status,
                        audio_last_error_json = excluded.audio_last_error_json,
                        audio_retry_count = excluded.audio_retry_count,
                        video_status = excluded.video_status,
                        subtitle_status = excluded.subtitle_status,
                        render_status = excluded.render_status,
                        segment_status = excluded.segment_status,
                        updated_at = CURRENT_TIMESTAMP
                    "#,
                    params![
                        item.item_id,
                        item.project_id,
                        item.index,
                        item.source_text,
                        item.narration_text,
                        item.visual_goal,
                        item.visual_description,
                        characters_json,
                        character_ids_json,
                        item.location_id,
                        item.scene_description,
                        item.image_prompt,
                        item.negative_prompt,
                        item.video_prompt,
                        item.duration_seconds,
                        subtitle_chunks_json,
                        item.audio_path,
                        item.audio_duration_seconds,
                        audio_probe_json,
                        item.selected_image_id,
                        item.selected_video_segment_id,
                        item.status,
                        lock_flags_json,
                        item.shot_size,
                        item.camera_motion,
                        item.composition,
                        item.pace,
                        item.transition_type,
                        item.image_status,
                        image_last_error_json,
                        item.image_retry_count,
                        item.audio_status,
                        audio_last_error_json,
                        item.audio_retry_count,
                        item.video_status,
                        item.subtitle_status,
                        item.render_status,
                        item.segment_status,
                    ],
                )?;
                read_storyboard_item(connection, &item.item_id)
            })
            .map_err(|error| error.to_string())?
            .ok_or_else(|| {
                format!(
                    "Storyboard item was saved but cannot be read: {}",
                    item.item_id
                )
            })
    }

    pub fn upsert_storyboard_items(&self, items: Vec<SceneDto>) -> Result<Vec<SceneDto>, String> {
        items
            .into_iter()
            .map(|item| self.upsert_storyboard_item(&item))
            .collect()
    }

    pub fn insert_image_candidates(
        &self,
        candidates: &[NewImageCandidateRecord],
    ) -> Result<Vec<ImageCandidateDto>, String> {
        if candidates.is_empty() {
            return Ok(vec![]);
        }

        self.database
            .transaction(|transaction| {
                for candidate in candidates {
                    let snapshot_json =
                        serde_json::to_string(&candidate.generation_context_snapshot)
                            .map_err(json_to_sql_error)?;
                    transaction.execute(
                        r#"
                        INSERT INTO image_candidates (
                            image_id, item_id, image_path, prompt, negative_prompt, model,
                            provider_model_id, workflow_preset_id, status, selected,
                            derived_from_image_id, generation_context_snapshot_json
                        )
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                        "#,
                        params![
                            candidate.image_id,
                            candidate.item_id,
                            candidate.image_path,
                            candidate.prompt,
                            candidate.negative_prompt,
                            candidate.model,
                            candidate.provider_model_id,
                            candidate.workflow_preset_id,
                            candidate.status,
                            bool_to_i64(candidate.selected),
                            candidate.derived_from_image_id,
                            snapshot_json,
                        ],
                    )?;
                }

                transaction.execute(
                    r#"
                    UPDATE storyboard_items
                    SET
                        image_status = 'succeeded',
                        image_last_error_json = NULL,
                        status = 'succeeded',
                        updated_at = CURRENT_TIMESTAMP
                    WHERE item_id = ?1
                    "#,
                    [candidates[0].item_id.as_str()],
                )?;

                Ok(())
            })
            .map_err(|error| error.to_string())?;

        let mut saved = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            saved.push(
                self.get_image_candidate(&candidate.image_id)?
                    .ok_or_else(|| {
                        format!(
                            "Image candidate was saved but cannot be read: {}",
                            candidate.image_id
                        )
                    })?,
            );
        }
        Ok(saved)
    }

    pub fn get_image_candidate(&self, image_id: &str) -> Result<Option<ImageCandidateDto>, String> {
        self.database
            .with_connection(|connection| read_image_candidate(connection, image_id))
            .map_err(|error| error.to_string())
    }

    pub fn insert_video_segments(
        &self,
        segments: &[NewVideoSegmentRecord],
    ) -> Result<Vec<VideoSegmentDto>, String> {
        if segments.is_empty() {
            return Ok(vec![]);
        }

        self.database
            .transaction(|transaction| {
                for segment in segments {
                    let snapshot_json = serde_json::to_string(&segment.generation_context_snapshot)
                        .map_err(json_to_sql_error)?;
                    transaction.execute(
                        r#"
                        INSERT INTO video_segments (
                            segment_id, item_id, input_image_id, video_path, video_prompt,
                            duration_seconds, model, provider_model_id, workflow_preset_id,
                            status, selected, generation_context_snapshot_json
                        )
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                        "#,
                        params![
                            segment.segment_id,
                            segment.item_id,
                            segment.input_image_id,
                            segment.video_path,
                            segment.video_prompt,
                            segment.duration_seconds,
                            segment.model,
                            segment.provider_model_id,
                            segment.workflow_preset_id,
                            segment.status,
                            bool_to_i64(segment.selected),
                            snapshot_json,
                        ],
                    )?;
                }

                transaction.execute(
                    r#"
                    UPDATE storyboard_items
                    SET
                        video_status = 'succeeded',
                        segment_status = 'succeeded',
                        updated_at = CURRENT_TIMESTAMP
                    WHERE item_id = ?1
                    "#,
                    [segments[0].item_id.as_str()],
                )?;

                Ok(())
            })
            .map_err(|error| error.to_string())?;

        let mut saved = Vec::with_capacity(segments.len());
        for segment in segments {
            saved.push(
                self.get_video_segment(&segment.segment_id)?
                    .ok_or_else(|| {
                        format!(
                            "Video segment was saved but cannot be read: {}",
                            segment.segment_id
                        )
                    })?,
            );
        }
        Ok(saved)
    }

    pub fn get_video_segment(&self, segment_id: &str) -> Result<Option<VideoSegmentDto>, String> {
        self.database
            .with_connection(|connection| read_video_segment(connection, segment_id))
            .map_err(|error| error.to_string())
    }

    pub fn update_video_segment_media_probe(
        &self,
        segment_id: &str,
        probe: &MediaProbeDto,
    ) -> Result<VideoSegmentDto, String> {
        self.database
            .with_connection(|connection| {
                let snapshot_json: String = connection.query_row(
                    "SELECT generation_context_snapshot_json FROM video_segments WHERE segment_id = ?1",
                    [segment_id],
                    |row| row.get(0),
                )?;
                let mut snapshot = parse_json(&snapshot_json);
                if !snapshot.is_object() {
                    snapshot = json!({});
                }
                snapshot["mediaProbe"] = serde_json::to_value(probe).map_err(json_to_sql_error)?;
                let next_snapshot_json =
                    serde_json::to_string(&snapshot).map_err(json_to_sql_error)?;
                connection.execute(
                    "UPDATE video_segments SET generation_context_snapshot_json = ?1 WHERE segment_id = ?2",
                    params![next_snapshot_json, segment_id],
                )?;
                read_video_segment(connection, segment_id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
            })
            .map_err(|error| error.to_string())
    }

    pub fn get_storyboard_item(&self, item_id: &str) -> Result<Option<SceneDto>, String> {
        self.database
            .with_connection(|connection| read_storyboard_item(connection, item_id))
            .map_err(|error| error.to_string())
    }

    pub fn latest_image_revision(&self, item_id: &str) -> Result<u32, String> {
        self.database
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    "SELECT generation_context_snapshot_json FROM image_candidates WHERE item_id = ?1",
                )?;
                let rows = statement.query_map([item_id], |row| row.get::<_, String>(0))?;
                let mut latest = 0_u32;
                for row in rows {
                    let snapshot = parse_json(&row?);
                    if let Some(revision) = snapshot.get("revision").and_then(Value::as_u64) {
                        latest = latest.max(revision as u32);
                    }
                }
                Ok(latest)
            })
            .map_err(|error| error.to_string())
    }

    pub fn latest_video_revision(&self, item_id: &str) -> Result<u32, String> {
        self.database
            .with_connection(|connection| {
                let mut statement = connection.prepare(
                    "SELECT generation_context_snapshot_json FROM video_segments WHERE item_id = ?1",
                )?;
                let rows = statement.query_map([item_id], |row| row.get::<_, String>(0))?;
                let mut latest = 0_u32;
                for row in rows {
                    let snapshot = parse_json(&row?);
                    if let Some(revision) = snapshot.get("revision").and_then(Value::as_u64) {
                        latest = latest.max(revision as u32);
                    }
                }
                Ok(latest)
            })
            .map_err(|error| error.to_string())
    }

    pub fn select_image_candidate(
        &self,
        item_id: &str,
        image_id: &str,
    ) -> Result<SceneDto, String> {
        self.database
            .transaction(|transaction| {
                let exists: Option<String> = transaction
                    .query_row(
                        "SELECT image_id FROM image_candidates WHERE item_id = ?1 AND image_id = ?2",
                        params![item_id, image_id],
                        |row| row.get(0),
                    )
                    .optional()?;
                if exists.is_none() {
                    return Err(rusqlite::Error::QueryReturnedNoRows);
                }
                transaction.execute(
                    "UPDATE image_candidates SET selected = CASE WHEN image_id = ?1 THEN 1 ELSE 0 END WHERE item_id = ?2",
                    params![image_id, item_id],
                )?;
                transaction.execute(
                    r#"
                    UPDATE storyboard_items
                    SET
                        selected_image_id = ?1,
                        selected_video_segment_id = NULL,
                        image_status = 'succeeded',
                        video_status = 'pending',
                        segment_status = 'pending',
                        render_status = 'pending',
                        updated_at = CURRENT_TIMESTAMP
                    WHERE item_id = ?2
                    "#,
                    params![image_id, item_id],
                )?;
                transaction.execute(
                    "UPDATE video_segments SET selected = 0 WHERE item_id = ?1",
                    params![item_id],
                )?;
                Ok(())
            })
            .map_err(|error| match error {
                crate::db::DbError::Sql(rusqlite::Error::QueryReturnedNoRows) => {
                    format!("Image candidate not found: {image_id}")
                }
                other => other.to_string(),
            })?;

        self.get_storyboard_item(item_id)?
            .ok_or_else(|| format!("Storyboard item not found: {item_id}"))
    }

    pub fn update_storyboard_item_audio(
        &self,
        item_id: &str,
        audio_path: &str,
        audio_duration_seconds: Option<f64>,
        audio_probe: Option<&MediaProbeDto>,
    ) -> Result<SceneDto, String> {
        let previous_duration = self
            .database
            .with_connection(|connection| {
                connection
                    .query_row(
                        "SELECT audio_duration_seconds FROM storyboard_items WHERE item_id = ?1",
                        [item_id],
                        |row| row.get::<_, Option<f64>>(0),
                    )
                    .optional()
            })
            .map_err(|error| error.to_string())?
            .flatten();
        let duration_changed = match (previous_duration, audio_duration_seconds) {
            (Some(previous), Some(next)) => (previous - next).abs() > 0.01,
            (None, Some(_)) | (Some(_), None) => true,
            (None, None) => false,
        };
        let audio_probe_json = audio_probe
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| error.to_string())?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    UPDATE storyboard_items
                    SET
                        audio_path = ?1,
                        audio_duration_seconds = ?2,
                        audio_probe_json = ?3,
                        audio_status = 'succeeded',
                        audio_last_error_json = NULL,
                        subtitle_status = CASE WHEN ?5 THEN 'pending' ELSE subtitle_status END,
                        render_status = CASE WHEN ?5 THEN 'pending' ELSE render_status END,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE item_id = ?4
                    "#,
                    params![
                        audio_path,
                        audio_duration_seconds,
                        audio_probe_json,
                        item_id,
                        duration_changed
                    ],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get_storyboard_item(item_id)?
            .ok_or_else(|| format!("Storyboard item not found: {item_id}"))
    }

    pub fn update_storyboard_item_subtitles(
        &self,
        item_id: &str,
        subtitle_chunks: &[SubtitleChunkDto],
    ) -> Result<SceneDto, String> {
        let subtitle_chunks_json =
            serde_json::to_string(subtitle_chunks).map_err(|error| error.to_string())?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    UPDATE storyboard_items
                    SET
                        subtitle_chunks_json = ?1,
                        subtitle_status = 'succeeded',
                        render_status = 'pending',
                        updated_at = CURRENT_TIMESTAMP
                    WHERE item_id = ?2
                    "#,
                    params![subtitle_chunks_json, item_id],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get_storyboard_item(item_id)?
            .ok_or_else(|| format!("Storyboard item not found: {item_id}"))
    }

    pub fn select_video_segment(
        &self,
        item_id: &str,
        segment_id: &str,
    ) -> Result<SceneDto, String> {
        self.database
            .transaction(|transaction| {
                let exists: Option<String> = transaction
                    .query_row(
                        "SELECT segment_id FROM video_segments WHERE item_id = ?1 AND segment_id = ?2",
                        params![item_id, segment_id],
                        |row| row.get(0),
                    )
                    .optional()?;
                if exists.is_none() {
                    return Err(rusqlite::Error::QueryReturnedNoRows);
                }
                transaction.execute(
                    "UPDATE video_segments SET selected = CASE WHEN segment_id = ?1 THEN 1 ELSE 0 END WHERE item_id = ?2",
                    params![segment_id, item_id],
                )?;
                transaction.execute(
                    r#"
                    UPDATE storyboard_items
                    SET
                        selected_video_segment_id = ?1,
                        video_status = 'succeeded',
                        segment_status = 'succeeded',
                        updated_at = CURRENT_TIMESTAMP
                    WHERE item_id = ?2
                    "#,
                    params![segment_id, item_id],
                )?;
                Ok(())
            })
            .map_err(|error| match error {
                crate::db::DbError::Sql(rusqlite::Error::QueryReturnedNoRows) => {
                    format!("Video segment not found: {segment_id}")
                }
                other => other.to_string(),
            })?;

        self.get_storyboard_item(item_id)?
            .ok_or_else(|| format!("Storyboard item not found: {item_id}"))
    }

    pub fn clear_historical_image_candidates(&self, item_id: &str) -> Result<SceneDto, String> {
        self.database
            .with_connection(|connection| {
                let latest = latest_revision_for_item(connection, item_id)?;
                connection.execute(
                    r#"
                    DELETE FROM image_candidates
                    WHERE item_id = ?1
                      AND image_id != COALESCE((SELECT selected_image_id FROM storyboard_items WHERE item_id = ?1), '')
                      AND CAST(COALESCE(json_extract(generation_context_snapshot_json, '$.revision'), 1) AS INTEGER) < ?2
                    "#,
                    params![item_id, latest],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get_storyboard_item(item_id)?
            .ok_or_else(|| format!("Storyboard item not found: {item_id}"))
    }

    pub fn clear_historical_video_segments(&self, item_id: &str) -> Result<SceneDto, String> {
        self.database
            .with_connection(|connection| {
                let latest = latest_video_revision_for_item(connection, item_id)?;
                connection.execute(
                    r#"
                    DELETE FROM video_segments
                    WHERE item_id = ?1
                      AND segment_id != COALESCE((SELECT selected_video_segment_id FROM storyboard_items WHERE item_id = ?1), '')
                      AND CAST(COALESCE(json_extract(generation_context_snapshot_json, '$.revision'), 1) AS INTEGER) < ?2
                    "#,
                    params![item_id, latest],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get_storyboard_item(item_id)?
            .ok_or_else(|| format!("Storyboard item not found: {item_id}"))
    }

    pub fn mark_image_generation_failed(
        &self,
        item_id: &str,
        error_json: Value,
    ) -> Result<SceneDto, String> {
        let error_json_text =
            serde_json::to_string(&error_json).map_err(|error| error.to_string())?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    UPDATE storyboard_items
                    SET
                        image_status = 'failed',
                        status = 'failed',
                        image_last_error_json = ?1,
                        image_retry_count = image_retry_count + 1,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE item_id = ?2
                    "#,
                    params![error_json_text, item_id],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get_storyboard_item(item_id)?
            .ok_or_else(|| format!("Storyboard item not found: {item_id}"))
    }

    pub fn mark_video_generation_failed(
        &self,
        item_id: &str,
        error_json: Value,
    ) -> Result<SceneDto, String> {
        let _error_json_text =
            serde_json::to_string(&error_json).map_err(|error| error.to_string())?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    UPDATE storyboard_items
                    SET
                        video_status = 'failed',
                        segment_status = 'failed',
                        updated_at = CURRENT_TIMESTAMP
                    WHERE item_id = ?1
                    "#,
                    params![item_id],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get_storyboard_item(item_id)?
            .ok_or_else(|| format!("Storyboard item not found: {item_id}"))
    }

    pub fn mark_audio_generation_failed(
        &self,
        item_id: &str,
        error_json: Value,
    ) -> Result<SceneDto, String> {
        let error_json_text =
            serde_json::to_string(&error_json).map_err(|error| error.to_string())?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    UPDATE storyboard_items
                    SET
                        audio_status = 'failed',
                        audio_last_error_json = ?1,
                        audio_retry_count = audio_retry_count + 1,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE item_id = ?2
                    "#,
                    params![error_json_text, item_id],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get_storyboard_item(item_id)?
            .ok_or_else(|| format!("Storyboard item not found: {item_id}"))
    }
}

impl Repository for SceneRepository<'_> {
    fn database(&self) -> &Database {
        self.database
    }
}

fn read_storyboard(
    connection: &Connection,
    project_id: &str,
) -> Result<Option<StoryboardDto>, rusqlite::Error> {
    let project_exists = connection
        .query_row(
            "SELECT project_id FROM projects WHERE project_id = ?1",
            [project_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .is_some();
    if !project_exists {
        return Ok(None);
    }

    let mut statement = connection.prepare(
        r#"
        SELECT
            item_id, project_id, item_index, source_text, narration_text, visual_goal,
            visual_description, characters_json, character_ids_json, location_id,
            scene_description, image_prompt, negative_prompt, video_prompt,
            duration_seconds, subtitle_chunks_json, audio_path, audio_duration_seconds,
            audio_probe_json, selected_image_id, selected_video_segment_id,
            status, lock_flags_json, shot_size, camera_motion, composition, pace,
            transition_type, image_status, audio_status, video_status, subtitle_status,
            render_status, segment_status, image_last_error_json, image_retry_count,
            audio_last_error_json, audio_retry_count
        FROM storyboard_items
        WHERE project_id = ?1
        ORDER BY item_index ASC, created_at ASC
        "#,
    )?;
    let rows = statement.query_map([project_id], |row| row_to_storyboard_item(connection, row))?;
    let items = rows.collect::<Result<Vec<_>, _>>()?;
    let confirmed_narrations = items
        .iter()
        .map(|item| crate::domain::scene::NarrationDto {
            index: item.index,
            text: if item.narration_text.trim().is_empty() {
                item.source_text.clone()
            } else {
                item.narration_text.clone()
            },
            locked: crate::services::scene_service::script_text_locked(item),
        })
        .collect::<Vec<_>>();

    Ok(Some(StoryboardDto {
        storyboard_id: format!("storyboard_{project_id}"),
        project_id: project_id.to_string(),
        confirmed_narrations,
        items,
        review_status: "succeeded".to_string(),
    }))
}

fn read_storyboard_item(
    connection: &Connection,
    item_id: &str,
) -> Result<Option<SceneDto>, rusqlite::Error> {
    connection
        .query_row(
            r#"
            SELECT
                item_id, project_id, item_index, source_text, narration_text, visual_goal,
                visual_description, characters_json, character_ids_json, location_id,
                scene_description, image_prompt, negative_prompt, video_prompt,
                duration_seconds, subtitle_chunks_json, audio_path, audio_duration_seconds,
                audio_probe_json, selected_image_id, selected_video_segment_id,
                status, lock_flags_json, shot_size, camera_motion, composition, pace,
                transition_type, image_status, audio_status, video_status, subtitle_status,
                render_status, segment_status, image_last_error_json, image_retry_count,
                audio_last_error_json, audio_retry_count
            FROM storyboard_items
            WHERE item_id = ?1
            "#,
            [item_id],
            |row| row_to_storyboard_item(connection, row),
        )
        .optional()
}

fn row_to_storyboard_item(
    connection: &Connection,
    row: &Row<'_>,
) -> Result<SceneDto, rusqlite::Error> {
    let item_id: String = row.get(0)?;
    let characters_json: String = row.get(7)?;
    let character_ids_json: String = row.get(8)?;
    let subtitle_chunks_json: String = row.get(15)?;
    let audio_probe_json: Option<String> = row.get(18)?;
    let lock_flags_json: String = row.get(22)?;
    let image_last_error_json: Option<String> = row.get(34)?;
    let audio_last_error_json: Option<String> = row.get(36)?;

    Ok(SceneDto {
        item_id: item_id.clone(),
        project_id: row.get(1)?,
        index: row.get::<_, i64>(2)? as u32,
        source_text: row.get(3)?,
        narration_text: row.get(4)?,
        visual_goal: row.get(5)?,
        visual_description: row.get(6)?,
        characters: parse_string_array(&characters_json),
        character_ids: parse_string_array(&character_ids_json),
        location_id: row.get(9)?,
        scene_description: row.get(10)?,
        image_prompt: row.get(11)?,
        negative_prompt: row.get(12)?,
        video_prompt: row.get(13)?,
        duration_seconds: row.get(14)?,
        subtitle_chunks: parse_subtitle_chunks(&subtitle_chunks_json),
        audio_path: row.get(16)?,
        audio_duration_seconds: row.get(17)?,
        audio_probe: audio_probe_json.and_then(|value| parse_media_probe(&value)),
        selected_image_id: row.get(19)?,
        selected_video_segment_id: row.get(20)?,
        status: row.get(21)?,
        lock_flags_json: parse_json(&lock_flags_json),
        shot_size: row.get(23)?,
        camera_motion: row.get(24)?,
        composition: row.get(25)?,
        pace: row.get(26)?,
        transition_type: row.get(27)?,
        image_status: row.get(28)?,
        audio_status: row.get(29)?,
        video_status: row.get(30)?,
        subtitle_status: row.get(31)?,
        render_status: row.get(32)?,
        segment_status: row.get(33)?,
        image_last_error_json: image_last_error_json.map(|value| parse_json(&value)),
        image_retry_count: row.get::<_, i64>(35)?.max(0) as u32,
        audio_last_error_json: audio_last_error_json.map(|value| parse_json(&value)),
        audio_retry_count: row.get::<_, i64>(37)?.max(0) as u32,
        image_candidates: read_image_candidates_for_item(connection, &item_id)?,
        video_segments: read_video_segments_for_item(connection, &item_id)?,
        downstream_reset_records: Some(vec![]),
    })
}

fn read_image_candidates_for_item(
    connection: &Connection,
    item_id: &str,
) -> Result<Vec<ImageCandidateDto>, rusqlite::Error> {
    let mut statement = connection.prepare(
        r#"
        SELECT
            image_id, item_id, image_path, prompt, negative_prompt, model,
            provider_model_id, workflow_preset_id, status, selected, created_at,
            derived_from_image_id, generation_context_snapshot_json
        FROM image_candidates
        WHERE item_id = ?1
        ORDER BY created_at ASC, image_id ASC
        "#,
    )?;
    let rows = statement.query_map([item_id], row_to_image_candidate)?;
    rows.collect()
}

fn read_image_candidate(
    connection: &Connection,
    image_id: &str,
) -> Result<Option<ImageCandidateDto>, rusqlite::Error> {
    connection
        .query_row(
            r#"
            SELECT
                image_id, item_id, image_path, prompt, negative_prompt, model,
                provider_model_id, workflow_preset_id, status, selected, created_at,
                derived_from_image_id, generation_context_snapshot_json
            FROM image_candidates
            WHERE image_id = ?1
            "#,
            [image_id],
            row_to_image_candidate,
        )
        .optional()
}

fn row_to_image_candidate(row: &Row<'_>) -> Result<ImageCandidateDto, rusqlite::Error> {
    let snapshot_json: String = row.get(12)?;
    Ok(ImageCandidateDto {
        image_id: row.get(0)?,
        item_id: row.get(1)?,
        image_path: row.get(2)?,
        prompt: row.get(3)?,
        negative_prompt: row.get(4)?,
        model: row.get(5)?,
        provider_model_id: row.get(6)?,
        workflow_preset_id: row.get(7)?,
        status: row.get(8)?,
        selected: int_to_bool(row.get(9)?),
        created_at: row.get(10)?,
        derived_from_image_id: row.get(11)?,
        generation_context_snapshot: parse_json(&snapshot_json),
    })
}

fn read_video_segments_for_item(
    connection: &Connection,
    item_id: &str,
) -> Result<Vec<VideoSegmentDto>, rusqlite::Error> {
    let mut statement = connection.prepare(
        r#"
        SELECT
            segment_id, item_id, input_image_id, video_path, video_prompt,
            duration_seconds, model, provider_model_id, workflow_preset_id, status,
            selected, created_at, generation_context_snapshot_json
        FROM video_segments
        WHERE item_id = ?1
        ORDER BY created_at ASC, segment_id ASC
        "#,
    )?;
    let rows = statement.query_map([item_id], |row| row_to_video_segment(row))?;
    rows.collect()
}

fn read_video_segment(
    connection: &Connection,
    segment_id: &str,
) -> Result<Option<VideoSegmentDto>, rusqlite::Error> {
    connection
        .query_row(
            r#"
            SELECT
                segment_id, item_id, input_image_id, video_path, video_prompt,
                duration_seconds, model, provider_model_id, workflow_preset_id, status,
                selected, created_at, generation_context_snapshot_json
            FROM video_segments
            WHERE segment_id = ?1
            "#,
            [segment_id],
            row_to_video_segment,
        )
        .optional()
}

fn row_to_video_segment(row: &Row<'_>) -> Result<VideoSegmentDto, rusqlite::Error> {
    let snapshot_json: String = row.get(12)?;
    let snapshot = parse_json(&snapshot_json);
    let media_probe = snapshot
        .get("mediaProbe")
        .cloned()
        .and_then(|value| serde_json::from_value::<MediaProbeDto>(value).ok());
    Ok(VideoSegmentDto {
        segment_id: row.get(0)?,
        item_id: row.get(1)?,
        input_image_id: row.get(2)?,
        video_path: row.get(3)?,
        video_prompt: row.get(4)?,
        duration_seconds: row.get(5)?,
        model: row.get(6)?,
        provider_model_id: row.get(7)?,
        workflow_preset_id: row.get(8)?,
        status: row.get(9)?,
        selected: int_to_bool(row.get(10)?),
        created_at: row.get(11)?,
        media_probe,
        generation_context_snapshot: snapshot,
    })
}

fn latest_revision_for_item(
    connection: &Connection,
    item_id: &str,
) -> Result<i64, rusqlite::Error> {
    connection.query_row(
        r#"
        SELECT CAST(COALESCE(MAX(json_extract(generation_context_snapshot_json, '$.revision')), 0) AS INTEGER)
        FROM image_candidates
        WHERE item_id = ?1
        "#,
        [item_id],
        |row| row.get(0),
    )
}

fn latest_video_revision_for_item(
    connection: &Connection,
    item_id: &str,
) -> Result<i64, rusqlite::Error> {
    connection.query_row(
        r#"
        SELECT CAST(COALESCE(MAX(json_extract(generation_context_snapshot_json, '$.revision')), 0) AS INTEGER)
        FROM video_segments
        WHERE item_id = ?1
        "#,
        [item_id],
        |row| row.get(0),
    )
}

fn parse_json(value: &str) -> Value {
    serde_json::from_str(value).unwrap_or_else(|_| json!({}))
}

fn parse_subtitle_chunks(value: &str) -> Vec<crate::domain::scene::SubtitleChunkDto> {
    serde_json::from_str(value).unwrap_or_default()
}

fn parse_media_probe(value: &str) -> Option<MediaProbeDto> {
    serde_json::from_str(value).ok()
}

fn parse_string_array(value: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(value).unwrap_or_default()
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

#[cfg(test)]
mod tests {
    use super::{NewImageCandidateRecord, NewVideoSegmentRecord, SceneRepository};
    use crate::db::Database;
    use crate::domain::media::MediaProbeDto;
    use crate::domain::scene::SceneDto;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn update_video_segment_media_probe_persists_in_snapshot() {
        let path = test_database_path("scene_repo_probe");
        let database = Database::open(&path).expect("database should open");
        insert_project(&database, "project_probe", "Probe project");
        let repository = SceneRepository::new(&database);
        repository
            .upsert_storyboard_item(&test_item())
            .expect("item should save");
        repository
            .insert_image_candidates(&[NewImageCandidateRecord {
                image_id: "image_probe".to_string(),
                item_id: "item_probe".to_string(),
                image_path: "projects/project_probe/images/image_probe.png".to_string(),
                prompt: "prompt".to_string(),
                negative_prompt: String::new(),
                model: "model".to_string(),
                provider_model_id: "model_probe".to_string(),
                workflow_preset_id: None,
                status: "succeeded".to_string(),
                selected: true,
                derived_from_image_id: None,
                generation_context_snapshot: json!({}),
            }])
            .expect("image should save");
        repository
            .insert_video_segments(&[NewVideoSegmentRecord {
                segment_id: "segment_probe".to_string(),
                item_id: "item_probe".to_string(),
                input_image_id: "image_probe".to_string(),
                video_path: "projects/project_probe/videos/segment_probe.mp4".to_string(),
                video_prompt: "move".to_string(),
                duration_seconds: 4.0,
                model: "model".to_string(),
                provider_model_id: "model_video".to_string(),
                workflow_preset_id: None,
                status: "succeeded".to_string(),
                selected: true,
                generation_context_snapshot: json!({ "revision": 1 }),
            }])
            .expect("video segment should save");

        let updated = repository
            .update_video_segment_media_probe(
                "segment_probe",
                &MediaProbeDto {
                    path: "projects/project_probe/videos/segment_probe.mp4".to_string(),
                    media_kind: "video".to_string(),
                    container: Some("mp4".to_string()),
                    format_name: Some("mp4".to_string()),
                    duration_seconds: 4.0,
                    width: Some(720),
                    height: Some(1280),
                    fps: Some(30.0),
                    video_codec: Some("h264".to_string()),
                    pixel_format: Some("yuv420p".to_string()),
                    audio_codec: None,
                    sample_rate: None,
                    channels: None,
                    bit_rate: Some(1_000_000),
                    has_video_stream: true,
                    has_audio_stream: false,
                },
            )
            .expect("probe should persist");

        assert_eq!(
            updated.media_probe.as_ref().unwrap().video_codec.as_deref(),
            Some("h264")
        );
        assert_eq!(
            updated.generation_context_snapshot["mediaProbe"]["path"],
            "projects/project_probe/videos/segment_probe.mp4"
        );
        assert_eq!(updated.generation_context_snapshot["revision"], 1);

        cleanup(path);
    }

    #[test]
    fn update_storyboard_item_audio_persists_probe_and_marks_dependents_pending() {
        let path = test_database_path("scene_repo_audio_probe");
        let database = Database::open(&path).expect("database should open");
        insert_project(&database, "project_audio_probe", "Audio probe project");
        let repository = SceneRepository::new(&database);
        let mut item = test_item();
        item.project_id = "project_audio_probe".to_string();
        item.item_id = "item_audio_probe".to_string();
        item.audio_duration_seconds = Some(3.0);
        item.subtitle_status = "succeeded".to_string();
        item.render_status = "succeeded".to_string();
        repository
            .upsert_storyboard_item(&item)
            .expect("item should save");

        let updated = repository
            .update_storyboard_item_audio(
                "item_audio_probe",
                "projects/project_audio_probe/audio/item_audio_probe/voice.mp3",
                Some(4.25),
                Some(&MediaProbeDto {
                    path: "projects/project_audio_probe/audio/item_audio_probe/voice.mp3"
                        .to_string(),
                    media_kind: "audio".to_string(),
                    container: Some("mp3".to_string()),
                    format_name: Some("mp3".to_string()),
                    duration_seconds: 4.25,
                    width: None,
                    height: None,
                    fps: None,
                    video_codec: None,
                    pixel_format: None,
                    audio_codec: Some("mp3".to_string()),
                    sample_rate: Some(44_100),
                    channels: Some(2),
                    bit_rate: Some(128_000),
                    has_video_stream: false,
                    has_audio_stream: true,
                }),
            )
            .expect("audio probe should persist");

        assert_eq!(updated.audio_duration_seconds, Some(4.25));
        assert_eq!(updated.audio_probe.as_ref().unwrap().channels, Some(2));
        assert_eq!(updated.subtitle_status, "pending");
        assert_eq!(updated.render_status, "pending");
        cleanup(path);
    }

    #[test]
    fn select_image_candidate_resets_video_selection_and_dependents() {
        let path = test_database_path("scene_repo_select_image_reset");
        let database = Database::open(&path).expect("database should open");
        insert_project(&database, "project_select_reset", "Select reset project");
        let repository = SceneRepository::new(&database);
        let mut item = test_item();
        item.project_id = "project_select_reset".to_string();
        item.item_id = "item_select_reset".to_string();
        repository
            .upsert_storyboard_item(&item)
            .expect("item should save");
        repository
            .insert_image_candidates(&[
                NewImageCandidateRecord {
                    image_id: "image_before".to_string(),
                    item_id: "item_select_reset".to_string(),
                    image_path: "projects/project_select_reset/images/image_before.png".to_string(),
                    prompt: "before".to_string(),
                    negative_prompt: String::new(),
                    model: "model".to_string(),
                    provider_model_id: "model_before".to_string(),
                    workflow_preset_id: None,
                    status: "succeeded".to_string(),
                    selected: true,
                    derived_from_image_id: None,
                    generation_context_snapshot: json!({ "revision": 1 }),
                },
                NewImageCandidateRecord {
                    image_id: "image_after".to_string(),
                    item_id: "item_select_reset".to_string(),
                    image_path: "projects/project_select_reset/images/image_after.png".to_string(),
                    prompt: "after".to_string(),
                    negative_prompt: String::new(),
                    model: "model".to_string(),
                    provider_model_id: "model_after".to_string(),
                    workflow_preset_id: None,
                    status: "succeeded".to_string(),
                    selected: false,
                    derived_from_image_id: Some("image_before".to_string()),
                    generation_context_snapshot: json!({ "revision": 2 }),
                },
            ])
            .expect("images should save");
        repository
            .insert_video_segments(&[NewVideoSegmentRecord {
                segment_id: "segment_before".to_string(),
                item_id: "item_select_reset".to_string(),
                input_image_id: "image_before".to_string(),
                video_path: "projects/project_select_reset/videos/segment_before.mp4".to_string(),
                video_prompt: "move".to_string(),
                duration_seconds: 4.0,
                model: "video".to_string(),
                provider_model_id: "model_video".to_string(),
                workflow_preset_id: None,
                status: "succeeded".to_string(),
                selected: true,
                generation_context_snapshot: json!({ "revision": 1 }),
            }])
            .expect("segment should save");
        repository
            .select_video_segment("item_select_reset", "segment_before")
            .expect("segment should select");

        let updated = repository
            .select_image_candidate("item_select_reset", "image_after")
            .expect("image should select");

        assert_eq!(updated.selected_image_id.as_deref(), Some("image_after"));
        assert!(updated.selected_video_segment_id.is_none());
        assert_eq!(updated.video_status, "pending");
        assert_eq!(updated.segment_status, "pending");
        assert_eq!(updated.render_status, "pending");
        assert!(updated
            .video_segments
            .iter()
            .all(|segment| !segment.selected));
        cleanup(path);
    }

    fn insert_project(database: &Database, project_id: &str, title: &str) {
        database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    INSERT INTO projects (
                        project_id, title, workflow_type, input_type, input_process_mode,
                        aspect_ratio, target_scene_count, segment_duration_seconds,
                        content_language, lifecycle
                    )
                    VALUES (?1, ?2, 'image_to_video', 'topic', 'generate', '9:16', 1, 4, 'zh-CN', 'draft')
                    "#,
                    (project_id, title),
                )
            })
            .expect("project fixture should save");
    }

    fn test_item() -> SceneDto {
        SceneDto {
            item_id: "item_probe".to_string(),
            project_id: "project_probe".to_string(),
            index: 1,
            source_text: "source".to_string(),
            narration_text: "source".to_string(),
            visual_goal: "goal".to_string(),
            visual_description: "visual".to_string(),
            characters: vec![],
            character_ids: vec![],
            location_id: None,
            scene_description: "scene".to_string(),
            image_prompt: "prompt".to_string(),
            negative_prompt: String::new(),
            video_prompt: "move".to_string(),
            duration_seconds: 4.0,
            subtitle_chunks: vec![],
            audio_path: None,
            audio_duration_seconds: None,
            audio_probe: None,
            selected_image_id: None,
            selected_video_segment_id: None,
            status: "pending".to_string(),
            lock_flags_json: json!({}),
            shot_size: None,
            camera_motion: None,
            composition: None,
            pace: None,
            transition_type: None,
            image_status: "pending".to_string(),
            image_last_error_json: None,
            image_retry_count: 0,
            audio_status: "pending".to_string(),
            audio_last_error_json: None,
            audio_retry_count: 0,
            video_status: "pending".to_string(),
            subtitle_status: "pending".to_string(),
            render_status: "pending".to_string(),
            segment_status: "pending".to_string(),
            image_candidates: vec![],
            video_segments: vec![],
            downstream_reset_records: Some(vec![]),
        }
    }

    fn test_database_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vt-ai-short-video-maker-{name}-{}-{nanos}.sqlite3",
            std::process::id()
        ))
    }

    fn cleanup(path: PathBuf) {
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(path.with_extension("sqlite3-wal"));
        let _ = fs::remove_file(path.with_extension("sqlite3-shm"));
    }
}
