use crate::db::{Database, Repository};
use crate::domain::project::{
    CreateProjectRequest, ListProjectsRequest, NamedProjectAssetDto, PageResult, ProjectBibleDto,
    ProjectDetailDto, ProjectDto, ProjectLatestTaskDto, ProjectSummaryDto, UpdateProjectFields,
};
use crate::domain::task::{
    initial_step_status, DIGITAL_HUMAN_PIPELINE_STEPS, DIGITAL_HUMAN_TASK_KIND,
    IMAGE_SLIDESHOW_PIPELINE_STEPS, IMAGE_SLIDESHOW_TASK_KIND, IMAGE_TO_VIDEO_PIPELINE_STEPS,
    IMAGE_TO_VIDEO_TASK_KIND, MATERIAL_EDIT_PIPELINE_STEPS, MATERIAL_EDIT_TASK_KIND,
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
        let (active_pack_id, rule_refs, executable_refs) = resolve_project_config_refs(&request);
        let rule_refs_json =
            serde_json::to_string(&rule_refs).map_err(|error| error.to_string())?;
        let executable_refs_json =
            serde_json::to_string(&executable_refs).map_err(|error| error.to_string())?;
        let input_process_mode = if request.input_type == "paste" {
            "fixed".to_string()
        } else {
            request.input_process_mode
        };
        let input_options = request.input_options.unwrap_or_else(|| json!({}));
        let input_options_json =
            serde_json::to_string(&input_options).map_err(|error| error.to_string())?;
        let style_data_json = serde_json::to_string(&json!({
            "style_prompt": request.style_prompt.clone().unwrap_or_default(),
            "color_palette": [],
            "lighting": "",
            "composition": "",
            "negative_prompt": "",
            "reference_image_path": null,
            "reference_images_json": [],
            "reference_images": []
        }))
        .map_err(|error| error.to_string())?;
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
        let workflow_type = request.workflow_type.clone();
        let (task_kind, current_step, summary, pipeline_steps) =
            project_workflow_task_plan(&workflow_type);

        self.database
            .transaction(|transaction| {
                transaction.execute(
                    r#"
                    INSERT INTO projects (
                        project_id, title, workflow_type, input_type, input_process_mode,
                        input_options_json, source_text, source_text_path, aspect_ratio,
                        target_scene_count, segment_duration_seconds, style_prompt, tone,
                        content_language, active_pack_id, rule_refs_json,
                        executable_refs_json, lifecycle
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, 'draft')
                    "#,
                    params![
                        project_id,
                        title,
                        workflow_type,
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
                        active_pack_id,
                        rule_refs_json,
                        executable_refs_json,
                    ],
                )?;

                transaction.execute(
                    "INSERT INTO project_bibles (project_id, summary) VALUES (?1, ?2)",
                    params![project_id, "第一阶段默认项目设定集"],
                )?;

                transaction.execute(
                    r#"
                    INSERT INTO style_bibles (style_bible_id, project_id, name, data_json)
                    VALUES (?1, ?2, ?3, ?4)
                    "#,
                    params![
                        format!("style_{project_id}"),
                        project_id,
                        "默认画风",
                        style_data_json,
                    ],
                )?;

                transaction.execute(
                    r#"
                    INSERT INTO tasks (task_id, project_id, task_kind, task_status, current_step, summary)
                    VALUES (?1, ?2, ?3, 'running', ?4, ?5)
                    "#,
                    params![task_id, project_id, task_kind, current_step, summary],
                )?;

                for (order_index, step_name) in pipeline_steps.iter().enumerate() {
                    transaction.execute(
                        r#"
                        INSERT INTO task_steps (step_id, task_id, step_name, status, output_json, order_index)
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                        "#,
                        params![
                            format!("{task_id}_{step_name}"),
                            task_id,
                            step_name,
                            initial_step_status(step_name),
                            if *step_name == "project_init" {
                                Some(json!({ "orderIndex": order_index, "source": "project_create" }).to_string())
                            } else {
                                None
                            },
                            order_index as i64,
                        ],
                    )?;
                }

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
        } else {
            projects.retain(|project| project.lifecycle != "deleted");
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

    pub fn update_cover(
        &self,
        project_id: &str,
        cover_path: &str,
        cover_title: &str,
        cover_template_id: &str,
        cover_source_item_id: Option<&str>,
    ) -> Result<ProjectDetailDto, String> {
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    UPDATE projects
                    SET
                        cover_path = ?1,
                        cover_title = ?2,
                        cover_template_id = ?3,
                        cover_source_item_id = ?4,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE project_id = ?5
                    "#,
                    params![
                        cover_path,
                        cover_title,
                        cover_template_id,
                        cover_source_item_id,
                        project_id
                    ],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get_detail(project_id)?
            .ok_or_else(|| format!("Project not found: {project_id}"))
    }

    pub fn update_basic_fields(
        &self,
        project_id: &str,
        fields: UpdateProjectFields,
    ) -> Result<ProjectDetailDto, String> {
        let input_options_json =
            serde_json::to_string(&fields.input_options).map_err(|error| error.to_string())?;
        let rule_refs_json =
            serde_json::to_string(&fields.rule_refs).map_err(|error| error.to_string())?;
        let executable_refs_json =
            serde_json::to_string(&fields.executable_refs).map_err(|error| error.to_string())?;

        let affected = self
            .database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    UPDATE projects
                    SET
                        title = ?1,
                        input_options_json = ?2,
                        source_text = ?3,
                        source_text_path = ?4,
                        aspect_ratio = ?5,
                        target_scene_count = ?6,
                        segment_duration_seconds = ?7,
                        style_prompt = ?8,
                        active_pack_id = ?9,
                        rule_refs_json = ?10,
                        executable_refs_json = ?11,
                        cover_title = ?12,
                        tone = ?13,
                        content_language = ?14,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE project_id = ?15
                    "#,
                    params![
                        fields.title,
                        input_options_json,
                        fields.source_text,
                        fields.source_text_path,
                        fields.aspect_ratio,
                        fields.target_scene_count,
                        fields.segment_duration_seconds,
                        fields.style_prompt,
                        fields.active_pack_id,
                        rule_refs_json,
                        executable_refs_json,
                        fields.cover_title,
                        fields.tone,
                        fields.content_language,
                        project_id
                    ],
                )
            })
            .map_err(|error| error.to_string())?;

        if affected == 0 {
            return Err(format!("Project not found: {project_id}"));
        }

        self.get_detail(project_id)?
            .ok_or_else(|| format!("Project not found: {project_id}"))
    }

    pub fn update_source_text_path(
        &self,
        project_id: &str,
        source_text_path: &str,
    ) -> Result<ProjectDetailDto, String> {
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    UPDATE projects
                    SET
                        source_text = NULL,
                        source_text_path = ?1,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE project_id = ?2
                    "#,
                    params![source_text_path, project_id],
                )?;
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        self.get_detail(project_id)?
            .ok_or_else(|| format!("Project not found: {project_id}"))
    }

    pub fn update_lifecycle(
        &self,
        project_id: &str,
        lifecycle: &str,
    ) -> Result<ProjectDetailDto, String> {
        let affected = self
            .database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    UPDATE projects
                    SET lifecycle = ?1, updated_at = CURRENT_TIMESTAMP
                    WHERE project_id = ?2
                    "#,
                    params![lifecycle, project_id],
                )
            })
            .map_err(|error| error.to_string())?;

        if affected == 0 {
            return Err(format!("Project not found: {project_id}"));
        }

        self.get_detail(project_id)?
            .ok_or_else(|| format!("Project not found: {project_id}"))
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
            content_language, active_pack_id, rule_refs_json, executable_refs_json,
            cover_path, cover_title, cover_template_id, cover_source_item_id,
            lifecycle, created_at, updated_at
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
                content_language, active_pack_id, rule_refs_json, executable_refs_json,
                cover_path, cover_title, cover_template_id, cover_source_item_id,
                lifecycle, created_at, updated_at
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
        let id: String = row.get(0)?;
        Ok(NamedProjectAssetDto {
            style_id: if table == "style_bibles" {
                Some(id.clone())
            } else {
                None
            },
            character_id: if table == "character_bibles" {
                Some(id.clone())
            } else {
                None
            },
            location_id: if table == "location_bibles" {
                Some(id.clone())
            } else {
                None
            },
            id,
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
    let rule_refs_json: String = row.get(15)?;
    let executable_refs_json: String = row.get(16)?;
    let rule_refs = serde_json::from_str::<Value>(&rule_refs_json).unwrap_or_else(|_| json!({}));
    let executable_refs =
        serde_json::from_str::<Value>(&executable_refs_json).unwrap_or_else(|_| json!({}));

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
        active_pack_id: row.get(14)?,
        rule_refs,
        executable_refs,
        cover_path: row.get(17)?,
        cover_title: row.get(18)?,
        cover_template_id: row.get(19)?,
        cover_source_item_id: row.get(20)?,
        tone: row.get(12)?,
        content_language: row.get(13)?,
        lifecycle: row.get(21)?,
        created_at: row.get(22)?,
        updated_at: row.get(23)?,
        latest_task,
    })
}

fn resolve_project_config_refs(request: &CreateProjectRequest) -> (Option<String>, Value, Value) {
    (
        request
            .active_pack_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        request.rule_refs.clone().unwrap_or_else(|| json!({})),
        request.executable_refs.clone().unwrap_or_else(|| json!({})),
    )
}

fn project_workflow_task_plan(
    workflow_type: &str,
) -> (
    &'static str,
    &'static str,
    &'static str,
    &'static [&'static str],
) {
    if workflow_type == DIGITAL_HUMAN_TASK_KIND {
        return (
            DIGITAL_HUMAN_TASK_KIND,
            "script_review",
            "等待确认口播文案",
            DIGITAL_HUMAN_PIPELINE_STEPS,
        );
    }
    if workflow_type == MATERIAL_EDIT_TASK_KIND {
        return (
            MATERIAL_EDIT_TASK_KIND,
            "material_import",
            "等待导入素材",
            MATERIAL_EDIT_PIPELINE_STEPS,
        );
    }
    if workflow_type == IMAGE_SLIDESHOW_TASK_KIND {
        return (
            IMAGE_SLIDESHOW_TASK_KIND,
            "storyboard_review",
            "等待确认分镜",
            IMAGE_SLIDESHOW_PIPELINE_STEPS,
        );
    }

    (
        IMAGE_TO_VIDEO_TASK_KIND,
        "storyboard_review",
        "等待确认分镜",
        IMAGE_TO_VIDEO_PIPELINE_STEPS,
    )
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
