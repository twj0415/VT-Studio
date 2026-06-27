use crate::db::{Database, Repository};
use crate::domain::novel::NovelChapterDto;
use rusqlite::{params, Connection, Row};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct NovelRepository<'db> {
    database: &'db Database,
}

impl<'db> NovelRepository<'db> {
    pub fn new(database: &'db Database) -> Self {
        Self { database }
    }

    pub fn replace_project_chapters(
        &self,
        project_id: &str,
        chapters: Vec<NewNovelChapterRecord>,
    ) -> Result<Vec<NovelChapterDto>, String> {
        self.database
            .transaction(|transaction| {
                transaction.execute(
                    "DELETE FROM novel_chapters WHERE project_id = ?1",
                    [project_id],
                )?;
                for chapter in chapters {
                    transaction.execute(
                        r#"
                        INSERT INTO novel_chapters (
                            novel_chapter_id, project_id, chapter_index, volume_title,
                            chapter_title, chapter_content, structured_event_json,
                            event_status, error_reason, retry_count
                        )
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6, '{}', 'pending', NULL, 0)
                        "#,
                        params![
                            chapter.novel_chapter_id,
                            project_id,
                            chapter.chapter_index,
                            chapter.volume_title,
                            chapter.chapter_title,
                            chapter.chapter_content,
                        ],
                    )?;
                }
                Ok(())
            })
            .map_err(|error| error.to_string())?;
        self.list_project_chapters(project_id)
    }

    pub fn list_project_chapters(&self, project_id: &str) -> Result<Vec<NovelChapterDto>, String> {
        self.database
            .with_connection(|connection| read_project_chapters(connection, project_id))
            .map_err(|error| error.to_string())
    }

    pub fn mark_event_succeeded(
        &self,
        novel_chapter_id: &str,
        structured_event: Value,
    ) -> Result<NovelChapterDto, String> {
        let structured_event_json =
            serde_json::to_string(&structured_event).map_err(|error| error.to_string())?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    UPDATE novel_chapters
                    SET structured_event_json = ?1,
                        event_status = 'succeeded',
                        error_reason = NULL,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE novel_chapter_id = ?2
                    "#,
                    params![structured_event_json, novel_chapter_id],
                )?;
                read_chapter(connection, novel_chapter_id)
            })
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("Novel chapter not found: {novel_chapter_id}"))
    }

    pub fn mark_event_failed(
        &self,
        novel_chapter_id: &str,
        error_reason: &str,
    ) -> Result<NovelChapterDto, String> {
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    UPDATE novel_chapters
                    SET event_status = 'failed',
                        error_reason = ?1,
                        retry_count = retry_count + 1,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE novel_chapter_id = ?2
                    "#,
                    params![error_reason, novel_chapter_id],
                )?;
                read_chapter(connection, novel_chapter_id)
            })
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("Novel chapter not found: {novel_chapter_id}"))
    }

    pub fn reset_event_for_retry(&self, novel_chapter_id: &str) -> Result<NovelChapterDto, String> {
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    UPDATE novel_chapters
                    SET event_status = 'pending',
                        error_reason = NULL,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE novel_chapter_id = ?1
                    "#,
                    [novel_chapter_id],
                )?;
                read_chapter(connection, novel_chapter_id)
            })
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("Novel chapter not found: {novel_chapter_id}"))
    }
}

impl Repository for NovelRepository<'_> {
    fn database(&self) -> &Database {
        self.database
    }
}

#[derive(Debug, Clone)]
pub struct NewNovelChapterRecord {
    pub novel_chapter_id: String,
    pub chapter_index: u32,
    pub volume_title: Option<String>,
    pub chapter_title: String,
    pub chapter_content: String,
}

impl NewNovelChapterRecord {
    pub fn new(chapter_index: u32, chapter_title: String, chapter_content: String) -> Self {
        Self {
            novel_chapter_id: create_id("chapter"),
            chapter_index,
            volume_title: None,
            chapter_title,
            chapter_content,
        }
    }
}

fn read_project_chapters(
    connection: &Connection,
    project_id: &str,
) -> Result<Vec<NovelChapterDto>, rusqlite::Error> {
    let mut statement = connection.prepare(
        r#"
        SELECT
            novel_chapter_id, project_id, chapter_index, volume_title,
            chapter_title, chapter_content, structured_event_json, event_status,
            error_reason, retry_count, created_at, updated_at
        FROM novel_chapters
        WHERE project_id = ?1
        ORDER BY chapter_index ASC, created_at ASC
        "#,
    )?;
    let rows = statement.query_map([project_id], row_to_chapter)?;
    rows.collect()
}

fn read_chapter(
    connection: &Connection,
    novel_chapter_id: &str,
) -> Result<Option<NovelChapterDto>, rusqlite::Error> {
    let mut statement = connection.prepare(
        r#"
        SELECT
            novel_chapter_id, project_id, chapter_index, volume_title,
            chapter_title, chapter_content, structured_event_json, event_status,
            error_reason, retry_count, created_at, updated_at
        FROM novel_chapters
        WHERE novel_chapter_id = ?1
        "#,
    )?;
    let mut rows = statement.query_map([novel_chapter_id], row_to_chapter)?;
    rows.next().transpose()
}

fn row_to_chapter(row: &Row<'_>) -> Result<NovelChapterDto, rusqlite::Error> {
    let structured_event_json: String = row.get(6)?;
    Ok(NovelChapterDto {
        novel_chapter_id: row.get(0)?,
        project_id: row.get(1)?,
        chapter_index: row.get::<_, i64>(2)? as u32,
        volume_title: row.get(3)?,
        chapter_title: row.get(4)?,
        chapter_content: row.get(5)?,
        structured_event: serde_json::from_str(&structured_event_json).unwrap_or_default(),
        event_status: row.get(7)?,
        error_reason: row.get(8)?,
        retry_count: row.get::<_, i64>(9)? as u32,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

fn create_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{prefix}_{nanos}")
}
