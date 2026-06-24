use crate::db::{Database, Repository};
use crate::domain::project::{
    CreateProjectRequest, ListProjectsRequest, NamedProjectAssetDto, PageResult, ProjectBibleDto,
    ProjectDetailDto, ProjectDto, ProjectLatestTaskDto, ProjectSummaryDto,
};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ProjectRepository<'db> {
    database: &'db Database,
}

impl<'db> ProjectRepository<'db> {
    pub fn new(database: &'db Database) -> Self {
        Self { database }
    }

    pub fn create_project_id() -> String {
        create_id("project")
    }

    pub fn create_with_id(
        &self,
        project_id: String,
        request: CreateProjectRequest,
    ) -> Result<ProjectDetailDto, String> {
        let task_id = create_id("task");
        let input_process_mode = if request.input_type == "paste" {
            "fixed".to_string()
        } else {
            request.input_process_mode
        };
        let input_options = request.input_options.unwrap_or_else(|| json!({}));
        let input_options_json =
            serde_json::to_string(&input_options).map_err(|error| error.to_string())?;
        let (source_text, source_text_path) = resolve_source_text_fields(
            request.topic.clone(),
            request.source_text,
            request.source_text_path,
        );
        let title = if request.title.trim().is_empty() {
            default_title(&request.input_type, request.topic.as_deref())
        } else {
            request.title
        };

        self.database
            .transaction(|transaction| {
                transaction.execute(
                    r#"
                    INSERT INTO projects (
                        project_id, title, workflow_type, input_type, input_process_mode,
                        input_options_json, source_text, source_text_path, aspect_ratio,
                        target_scene_count, segment_duration_seconds, style_prompt, tone,
                        content_language, lifecycle
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, 'draft')
                    "#,
                    params![
                        project_id,
                        title,
                        request.workflow_type,
                        request.input_type,
                        input_process_mode,
                        input_options_json,
                        source_text,
                        source_text_path,
                        request.aspect_ratio,
                        request.target_scene_count,
                        request.segment_duration_seconds,
                        request.style_prompt,
                        request.tone,
                        request.content_language,
                    ],
                )?;

                transaction.execute(
                    "INSERT INTO project_bibles (project_id, summary) VALUES (?1, ?2)",
                    params![project_id, "第一阶段默认项目设定集"],
                )?;

                transaction.execute(
                    r#"
                    INSERT INTO tasks (task_id, project_id, task_kind, task_status, current_step, summary)
                    VALUES (?1, ?2, 'image_to_video', 'waiting_user', 'storyboard_review', ?3)
                    "#,
                    params![task_id, project_id, "等待确认分镜"],
                )?;

                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get_detail(&project_id)?
            .ok_or_else(|| format!("Project {project_id} was created but cannot be read back."))
    }

    pub fn list(
        &self,
        request: ListProjectsRequest,
    ) -> Result<PageResult<ProjectSummaryDto>, String> {
        let mut projects = self
            .database
            .with_connection(read_all_projects)
            .map_err(|error| error.to_string())?;

        if let Some(keyword) = request.keyword.as_deref() {
            let keyword = keyword.trim();
            if !keyword.is_empty() {
                projects.retain(|project| project.title.contains(keyword));
            }
        }

        if let Some(lifecycle) = request.lifecycle.as_deref() {
            projects.retain(|project| project.lifecycle == lifecycle);
        }

        if matches!(request.sort_by.as_deref(), Some("title")) {
            projects.sort_by(|left, right| left.title.cmp(&right.title));
        } else {
            projects.sort_by(|left, right| left.updated_at.cmp(&right.updated_at));
        }

        if matches!(request.sort_order.as_deref(), Some("desc")) || request.sort_order.is_none() {
            projects.reverse();
        }

        let total = projects.len() as u32;
        let page = request.page.max(1);
        let page_size = request.page_size.max(1);
        let start = ((page - 1) * page_size) as usize;
        let end = (start + page_size as usize).min(projects.len());
        let items = if start >= projects.len() {
            vec![]
        } else {
            projects[start..end].to_vec()
        };

        Ok(PageResult {
            items,
            total,
            page,
            page_size,
        })
    }

    pub fn get_detail(&self, project_id: &str) -> Result<Option<ProjectDetailDto>, String> {
        self.database
            .with_connection(|connection| read_project_detail(connection, project_id))
            .map_err(|error| error.to_string())
    }
}

impl Repository for ProjectRepository<'_> {
    fn database(&self) -> &Database {
        self.database
    }
}

fn read_all_projects(connection: &Connection) -> Result<Vec<ProjectDto>, rusqlite::Error> {
    let mut statement = connection.prepare(
        r#"
        SELECT
            project_id, title, workflow_type, input_type, input_process_mode,
            input_options_json, source_text, source_text_path, aspect_ratio,
            target_scene_count, segment_duration_seconds, style_prompt, tone,
            content_language, lifecycle, created_at, updated_at
        FROM projects
        "#,
    )?;
    let rows = statement.query_map([], |row| row_to_project(row, None))?;
    let mut projects = rows.collect::<Result<Vec<_>, _>>()?;
    drop(statement);

    for project in &mut projects {
        project.latest_task = read_latest_task(connection, &project.project_id)?;
    }

    Ok(projects)
}

fn read_project_detail(
    connection: &Connection,
    project_id: &str,
) -> Result<Option<ProjectDetailDto>, rusqlite::Error> {
    let Some(mut project) = read_project(connection, project_id)? else {
        return Ok(None);
    };
    project.latest_task = read_latest_task(connection, project_id)?;

    Ok(Some(ProjectDetailDto {
        project_bible: read_project_bible(connection, project_id)?,
        project,
        style_bible: read_named_asset(connection, "style_bibles", "style_bible_id", project_id)?,
        character_bibles: read_named_assets(
            connection,
            "character_bibles",
            "character_bible_id",
            project_id,
        )?,
        location_bibles: read_named_assets(
            connection,
            "location_bibles",
            "location_bible_id",
            project_id,
        )?,
    }))
}

fn read_project(
    connection: &Connection,
    project_id: &str,
) -> Result<Option<ProjectDto>, rusqlite::Error> {
    connection
        .query_row(
            r#"
            SELECT
                project_id, title, workflow_type, input_type, input_process_mode,
                input_options_json, source_text, source_text_path, aspect_ratio,
                target_scene_count, segment_duration_seconds, style_prompt, tone,
                content_language, lifecycle, created_at, updated_at
            FROM projects
            WHERE project_id = ?1
            "#,
            [project_id],
            |row| row_to_project(row, None),
        )
        .optional()
}

fn read_latest_task(
    connection: &Connection,
    project_id: &str,
) -> Result<Option<ProjectLatestTaskDto>, rusqlite::Error> {
    connection
        .query_row(
            r#"
            SELECT task_id, task_status, summary
            FROM tasks
            WHERE project_id = ?1
            ORDER BY updated_at DESC, created_at DESC
            LIMIT 1
            "#,
            [project_id],
            |row| {
                Ok(ProjectLatestTaskDto {
                    task_id: row.get(0)?,
                    task_status: row.get(1)?,
                    summary: row.get(2)?,
                })
            },
        )
        .optional()
}

fn read_project_bible(
    connection: &Connection,
    project_id: &str,
) -> Result<ProjectBibleDto, rusqlite::Error> {
    connection
        .query_row(
            "SELECT project_id, summary FROM project_bibles WHERE project_id = ?1",
            [project_id],
            |row| {
                Ok(ProjectBibleDto {
                    project_id: row.get(0)?,
                    summary: row.get(1)?,
                })
            },
        )
        .optional()
        .map(|value| {
            value.unwrap_or_else(|| ProjectBibleDto {
                project_id: project_id.to_string(),
                summary: String::new(),
            })
        })
}

fn read_named_asset(
    connection: &Connection,
    table: &str,
    id_column: &str,
    project_id: &str,
) -> Result<Option<NamedProjectAssetDto>, rusqlite::Error> {
    let mut assets = read_named_assets(connection, table, id_column, project_id)?;
    Ok(assets.pop())
}

fn read_named_assets(
    connection: &Connection,
    table: &str,
    id_column: &str,
    project_id: &str,
) -> Result<Vec<NamedProjectAssetDto>, rusqlite::Error> {
    let sql = format!(
        "SELECT {id_column}, name FROM {table} WHERE project_id = ?1 ORDER BY created_at ASC"
    );
    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map([project_id], |row| {
        Ok(NamedProjectAssetDto {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;

    rows.collect()
}

fn row_to_project(
    row: &Row<'_>,
    latest_task: Option<ProjectLatestTaskDto>,
) -> Result<ProjectDto, rusqlite::Error> {
    let input_options_json: String = row.get(5)?;
    let input_options =
        serde_json::from_str::<Value>(&input_options_json).unwrap_or_else(|_| json!({}));

    Ok(ProjectDto {
        project_id: row.get(0)?,
        title: row.get(1)?,
        workflow_type: row.get(2)?,
        input_type: row.get(3)?,
        input_process_mode: row.get(4)?,
        input_options,
        source_text: row.get(6)?,
        source_text_path: row.get(7)?,
        aspect_ratio: row.get(8)?,
        target_scene_count: row.get(9)?,
        segment_duration_seconds: row.get(10)?,
        style_prompt: row.get(11)?,
        tone: row.get(12)?,
        content_language: row.get(13)?,
        lifecycle: row.get(14)?,
        created_at: row.get(15)?,
        updated_at: row.get(16)?,
        latest_task,
    })
}

fn resolve_source_text_fields(
    topic: Option<String>,
    source_text: Option<String>,
    source_text_path: Option<String>,
) -> (Option<String>, Option<String>) {
    let inline_text = source_text.or(topic);
    let Some(text) = inline_text else {
        return (None, source_text_path);
    };

    (Some(text), None)
}

fn default_title(input_type: &str, topic: Option<&str>) -> String {
    if input_type == "topic" {
        return topic.unwrap_or("新项目").chars().take(15).collect();
    }

    if input_type == "paste" {
        return "固定文案项目".to_string();
    }

    if input_type == "article" {
        return "文章项目".to_string();
    }

    "新项目".to_string()
}

fn create_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{prefix}_{nanos}")
}
