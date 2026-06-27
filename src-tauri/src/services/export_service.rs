use crate::db::asset_repository::{AssetRepository, NewAssetRecord, NewAssetReferenceRecord};
use crate::db::export_repository::{ExportRepository, NewExportRecord};
use crate::db::project_repository::ProjectRepository;
use crate::db::scene_repository::SceneRepository;
use crate::db::task_repository::TaskRepository;
use crate::db::Database;
use crate::domain::export::{
    BackupWorkspaceDto, BackupWorkspaceRequest, ExportDiagnosticPackageDto,
    ExportDiagnosticPackageRequest, ExportFinalVideoRequest, ExportProjectPackageRequest,
    ExportRecordDto, ImportProjectPackageDto, ImportProjectPackageRequest,
    ListExportRecordsRequest, OpenExportDirectoryDto, OpenExportDirectoryRequest,
    RestoreWorkspaceDto, RestoreWorkspaceRequest, RestoredProjectDto,
};
use crate::domain::project::{CreateProjectRequest, ListProjectsRequest, ProjectDetailDto};
use crate::domain::scene::StoryboardDto;
use crate::domain::task::CompositionTaskDto;
use crate::security::path_guard::PathGuard;
use crate::security::secret_guard::{reject_json_secrets, reject_secret_scan, SecretScanInput};
use crate::services::diagnostic_service;
use crate::services::ffmpeg_service;
use crate::services::log_service::{
    write_app_log, write_project_export_log, StructuredFileLogRecord,
};
use crate::services::storage_service::{FileAccessPolicy, FileBucket, StorageService};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const FINAL_VIDEO_EXPORT_KIND: &str = "final_video";
const PROJECT_PACKAGE_EXPORT_KIND: &str = "project_package";
const WORKSPACE_BACKUP_VERSION: i64 = 1;
const DIAGNOSTIC_PACKAGE_VERSION: i64 = 1;
const DIAGNOSTIC_LOG_TAIL_BYTES: usize = 64 * 1024;

pub fn export_final_video(
    database: &Database,
    workspace_root: &Path,
    request: ExportFinalVideoRequest,
) -> Result<ExportRecordDto, String> {
    let composition = latest_succeeded_composition(database, &request.project_id)?;
    let source_relative_path = validate_output_relative_path(&composition.output_path)?;
    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let source_path = resolve_existing_output_path(&storage, &source_relative_path)
        .map_err(|error| format!("export.final_missing: {error}"))?;
    let target_bucket_relative = next_export_target_path(
        &storage,
        &request.project_id,
        &composition,
        request.overwrite.unwrap_or(false),
    )?;
    let stored = storage
        .copy_into_bucket(
            &source_path,
            FileBucket::Output,
            &target_bucket_relative,
            FileAccessPolicy::ExportCopy,
        )
        .map_err(|error| format!("export.target_denied: {error}"))?;

    let record = ExportRepository::new(database).insert_export_record(NewExportRecord {
        export_id: create_export_id(),
        project_id: request.project_id,
        composition_task_id: Some(composition.task_id),
        export_kind: FINAL_VIDEO_EXPORT_KIND.to_string(),
        source_relative_path: Some(source_relative_path),
        target_relative_path: Some(stored.relative_path),
        status: "succeeded".to_string(),
        error_json: None,
        metadata_json: json!({
            "fileName": target_bucket_relative.rsplit('/').next().unwrap_or("final.mp4"),
            "overwrite": request.overwrite.unwrap_or(false),
            "controlledDirectory": "outputs/user_exports"
        }),
    })?;
    write_export_success_log(workspace_root, &record, "final video exported")?;
    Ok(record)
}

pub fn list_export_records(
    database: &Database,
    request: ListExportRecordsRequest,
) -> Result<Vec<ExportRecordDto>, String> {
    ExportRepository::new(database).list_project_exports(&request.project_id)
}

pub fn export_project_package(
    database: &Database,
    workspace_root: &Path,
    request: ExportProjectPackageRequest,
) -> Result<ExportRecordDto, String> {
    let project = ProjectRepository::new(database)
        .get_detail(&request.project_id)?
        .ok_or_else(|| {
            format!(
                "export.package_invalid: Project not found: {}",
                request.project_id
            )
        })?;
    let storyboard = SceneRepository::new(database)
        .get_storyboard(&request.project_id)?
        .ok_or_else(|| {
            format!(
                "export.package_invalid: Storyboard not found for project {}.",
                request.project_id
            )
        })?;
    let asset_repository = AssetRepository::new(database);
    let assets = asset_repository.list_assets(None, false)?;
    let asset_paths = asset_repository.collect_project_asset_paths(&request.project_id)?;
    let included_assets = assets
        .iter()
        .filter(|asset| asset_paths.iter().any(|path| path == &asset.relative_path))
        .cloned()
        .collect::<Vec<_>>();
    let asset_references = included_assets
        .iter()
        .map(|asset| {
            asset_repository
                .list_references(&asset.asset_id)
                .map(|references| json!({ "asset": asset, "references": references }))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let composition_task =
        TaskRepository::new(database).latest_composition_task_by_project(&request.project_id)?;
    let export_records =
        ExportRepository::new(database).list_project_exports(&request.project_id)?;
    let manifest = json!({
        "app": "vt-ai-short-video-maker",
        "packageVersion": 1,
        "projectId": request.project_id,
        "title": project.project.title,
        "createdAt": current_timestamp_string(),
        "containsSecrets": false,
        "entries": {
            "manifest": "manifest.json",
            "dbExport": "db_export.json",
            "projectSnapshot": format!("projects/{}/project.json", sanitize_file_segment(&project.project.project_id)),
        }
    });
    let db_export = json!({
        "schemaVersion": 1,
        "projectDetail": project,
        "storyboard": storyboard,
        "assets": included_assets,
        "assetReferences": asset_references,
        "compositionTask": composition_task,
        "exportRecords": export_records,
        "exportedAt": current_timestamp_string(),
    });
    let manifest_text = pretty_json(&manifest)?;
    let db_export_text = pretty_json(&db_export)?;
    reject_json_secrets(&db_export)
        .map_err(|error| format!("export.secret_detected: db_export.json: {error}"))?;
    reject_secret_scan(&[SecretScanInput {
        name: "manifest.json",
        content: &manifest_text,
    }])
    .map_err(|error| format!("export.secret_detected: {error}"))?;
    reject_absolute_paths_in_export_text("manifest.json", &manifest_text)?;
    reject_absolute_paths_in_export_text("db_export.json", &db_export_text)?;

    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let target_bucket_relative = next_project_package_target_path(
        &storage,
        &request.project_id,
        request.overwrite.unwrap_or(false),
    )?;
    let target_path = storage
        .resolver()
        .resolve_bucket_path_for_write(FileBucket::Output, &target_bucket_relative)
        .map_err(|error| format!("export.target_denied: {error}"))?;
    let mut entries = vec![
        ZipEntrySource::Bytes {
            entry_path: "manifest.json".to_string(),
            content: manifest_text.into_bytes(),
        },
        ZipEntrySource::Bytes {
            entry_path: "db_export.json".to_string(),
            content: db_export_text.into_bytes(),
        },
        ZipEntrySource::Bytes {
            entry_path: format!(
                "projects/{}/project.json",
                sanitize_file_segment(&request.project_id)
            ),
            content: pretty_json(&db_export["projectDetail"])?.into_bytes(),
        },
    ];
    append_project_package_file_entries(&storage, &asset_paths, &mut entries)?;
    if let Some(source_text_path) = db_export["projectDetail"]["project"]["sourceTextPath"].as_str()
    {
        append_workspace_file_if_exists(&storage, source_text_path, &mut entries)?;
    }
    if let Some(output_path) = db_export["compositionTask"]["outputPath"].as_str() {
        append_workspace_file_if_exists(&storage, output_path, &mut entries)?;
    }
    append_user_template_entries(&storage, &mut entries)?;
    write_stored_zip(&target_path, entries)
        .map_err(|error| format!("export.target_denied: {error}"))?;

    let record = ExportRepository::new(database).insert_export_record(NewExportRecord {
        export_id: create_export_id(),
        project_id: request.project_id,
        composition_task_id: db_export["compositionTask"]["taskId"]
            .as_str()
            .map(str::to_string),
        export_kind: PROJECT_PACKAGE_EXPORT_KIND.to_string(),
        source_relative_path: None,
        target_relative_path: Some(format!("outputs/{target_bucket_relative}")),
        status: "succeeded".to_string(),
        error_json: None,
        metadata_json: json!({
            "packageVersion": 1,
            "containsSecrets": false,
            "assetCount": asset_paths.len(),
            "controlledDirectory": "outputs/project_packages"
        }),
    })?;
    write_export_success_log(workspace_root, &record, "project package exported")?;
    Ok(record)
}

pub fn import_project_package(
    database: &Database,
    workspace_root: &Path,
    request: ImportProjectPackageRequest,
) -> Result<ImportProjectPackageDto, String> {
    let package_path = validate_project_package_path(&request.package_relative_path)?;
    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let package_bucket_path = package_path.strip_prefix("outputs/").ok_or_else(|| {
        "import.package_invalid: package path must be inside outputs.".to_string()
    })?;
    let absolute_package = storage
        .resolver()
        .resolve_existing_bucket_path(FileBucket::Output, package_bucket_path)
        .map_err(|error| format!("import.package_invalid: {error}"))?;
    let bytes = std::fs::read(&absolute_package)
        .map_err(|error| format!("import.package_invalid: {error}"))?;
    let entries = read_stored_zip_entries(&bytes)?;
    let manifest_text = read_zip_text_entry(&entries, "manifest.json")?;
    let db_export_text = read_zip_text_entry(&entries, "db_export.json")?;
    reject_secret_scan(&[
        SecretScanInput {
            name: "manifest.json",
            content: &manifest_text,
        },
        SecretScanInput {
            name: "db_export.json",
            content: &db_export_text,
        },
    ])
    .map_err(|error| format!("import.package_invalid: {error}"))?;
    let manifest = serde_json::from_str::<serde_json::Value>(&manifest_text)
        .map_err(|error| format!("import.package_invalid: invalid manifest: {error}"))?;
    validate_package_manifest(&manifest)?;
    let db_export = serde_json::from_str::<serde_json::Value>(&db_export_text)
        .map_err(|error| format!("import.package_invalid: invalid db_export: {error}"))?;
    if db_export["schemaVersion"].as_i64() != Some(1) {
        return Err("import.package_invalid: unsupported db_export schema version.".to_string());
    }
    let project_detail = serde_json::from_value::<ProjectDetailDto>(
        db_export
            .get("projectDetail")
            .cloned()
            .ok_or_else(|| "import.package_invalid: projectDetail is missing.".to_string())?,
    )
    .map_err(|error| format!("import.package_invalid: invalid projectDetail: {error}"))?;
    let storyboard = serde_json::from_value::<StoryboardDto>(
        db_export
            .get("storyboard")
            .cloned()
            .ok_or_else(|| "import.package_invalid: storyboard is missing.".to_string())?,
    )
    .map_err(|error| format!("import.package_invalid: invalid storyboard: {error}"))?;

    let source_project_id = project_detail.project.project_id.clone();
    let imported_project_id = create_imported_project_id();
    let imported_title = format!("{} 导入", project_detail.project.title);
    let detail = ProjectRepository::new(database).create_with_id(
        imported_project_id.clone(),
        CreateProjectRequest {
            title: imported_title.clone(),
            workflow_type: project_detail.project.workflow_type,
            input_type: project_detail.project.input_type,
            topic: None,
            source_text: project_detail.project.source_text,
            source_text_path: None,
            content_language: project_detail.project.content_language,
            tone: project_detail.project.tone,
            aspect_ratio: project_detail.project.aspect_ratio,
            target_scene_count: project_detail.project.target_scene_count,
            segment_duration_seconds: project_detail.project.segment_duration_seconds,
            style_prompt: project_detail.project.style_prompt,
            active_pack_id: project_detail.project.active_pack_id,
            rule_refs: Some(project_detail.project.rule_refs),
            executable_refs: Some(project_detail.project.executable_refs),
            input_process_mode: project_detail.project.input_process_mode,
            input_options: Some(project_detail.project.input_options),
        },
    )?;
    import_storyboard_items(database, &imported_project_id, storyboard)?;
    let imported_asset_count =
        import_package_assets(database, &storage, &entries, &imported_project_id)?;

    Ok(ImportProjectPackageDto {
        project_id: detail.project.project_id,
        source_project_id,
        title: imported_title,
        imported_asset_count,
    })
}

pub fn backup_workspace(
    database: &Database,
    workspace_root: &Path,
    request: BackupWorkspaceRequest,
) -> Result<BackupWorkspaceDto, String> {
    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;

    let projects = ProjectRepository::new(database).list(ListProjectsRequest {
        page: 1,
        page_size: u32::MAX,
        keyword: None,
        lifecycle: None,
        sort_by: None,
        sort_order: Some("asc".to_string()),
    })?;
    let project_repository = ProjectRepository::new(database);
    let scene_repository = SceneRepository::new(database);
    let task_repository = TaskRepository::new(database);
    let export_repository = ExportRepository::new(database);
    let asset_repository = AssetRepository::new(database);
    let assets = asset_repository.list_assets(None, false)?;
    let asset_references = assets
        .iter()
        .map(|asset| {
            asset_repository
                .list_references(&asset.asset_id)
                .map(|references| json!({ "asset": asset, "references": references }))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let project_exports = projects
        .items
        .iter()
        .map(|project| {
            let project_id = &project.project_id;
            let detail = project_repository
                .get_detail(project_id)?
                .ok_or_else(|| format!("backup.restore_failed: project missing: {project_id}"))?;
            let storyboard = scene_repository.get_storyboard(project_id)?;
            let composition_task =
                task_repository.latest_composition_task_by_project(project_id)?;
            let export_records = export_repository.list_project_exports(project_id)?;
            Ok(json!({
                "projectDetail": detail,
                "storyboard": storyboard,
                "compositionTask": composition_task,
                "exportRecords": export_records,
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let migrations = database
        .applied_migrations()
        .map_err(|error| format!("backup.restore_failed: {error}"))?;
    let backup_id = create_backup_id();
    let manifest = json!({
        "app": "vt-ai-short-video-maker",
        "backupVersion": WORKSPACE_BACKUP_VERSION,
        "backupId": backup_id,
        "createdAt": current_timestamp_string(),
        "containsSecrets": false,
        "requiresSecretReentry": true,
        "projectCount": project_exports.len(),
        "assetCount": assets.len(),
        "entries": {
            "manifest": "manifest.json",
            "dbExport": "db_export.json",
            "workspaceProjects": "projects/",
            "workspaceAssets": "assets/",
            "userTemplates": "templates/user/",
            "userPrompts": "prompts/user/"
        }
    });
    let db_export = json!({
        "schemaVersion": 1,
        "exportKind": "workspace_backup",
        "migrations": migrations,
        "projects": project_exports,
        "assets": assets,
        "assetReferences": asset_references,
        "exportedAt": current_timestamp_string(),
    });
    let manifest_text = pretty_json(&manifest)?;
    let db_export_text = pretty_json(&db_export)?;
    reject_json_secrets(&db_export)
        .map_err(|error| format!("export.secret_detected: db_export.json: {error}"))?;
    reject_secret_scan(&[SecretScanInput {
        name: "backup manifest.json",
        content: &manifest_text,
    }])
    .map_err(|error| format!("export.secret_detected: {error}"))?;
    reject_absolute_paths_in_export_text("backup manifest.json", &manifest_text)?;
    reject_absolute_paths_in_export_text("backup db_export.json", &db_export_text)?;

    let target_bucket_relative = next_workspace_backup_target_path(
        &storage,
        &backup_id,
        request.overwrite.unwrap_or(false),
    )?;
    let target_path = storage
        .resolver()
        .resolve_bucket_path_for_write(FileBucket::Output, &target_bucket_relative)
        .map_err(|error| format!("export.target_denied: {error}"))?;
    let mut entries = vec![
        ZipEntrySource::Bytes {
            entry_path: "manifest.json".to_string(),
            content: manifest_text.into_bytes(),
        },
        ZipEntrySource::Bytes {
            entry_path: "db_export.json".to_string(),
            content: db_export_text.into_bytes(),
        },
    ];
    append_backup_directory_if_exists(
        workspace_root,
        &storage.resolver().bucket_root(FileBucket::Project),
        "projects",
        &mut entries,
    )?;
    append_backup_directory_if_exists(
        workspace_root,
        &storage.resolver().bucket_root(FileBucket::Asset),
        "assets",
        &mut entries,
    )?;
    append_user_template_entries(&storage, &mut entries)?;
    append_backup_directory_if_exists(
        workspace_root,
        &workspace_root.join("prompts").join("user"),
        "prompts/user",
        &mut entries,
    )?;
    reject_secret_text_file_entries(&entries)?;
    write_stored_zip(&target_path, entries)
        .map_err(|error| format!("export.target_denied: {error}"))?;

    let dto = BackupWorkspaceDto {
        backup_id,
        target_relative_path: format!("outputs/{target_bucket_relative}"),
        project_count: project_exports.len(),
        asset_count: assets.len(),
        contains_secrets: false,
        requires_secret_reentry: true,
    };
    write_app_log(
        workspace_root,
        &StructuredFileLogRecord {
            trace_id: create_trace_id(),
            level: "info".to_string(),
            message: "workspace backup exported".to_string(),
            project_id: None,
            task_id: None,
            task_step_id: None,
            step_kind: Some("workspace_backup".to_string()),
            item_id: None,
            provider_id: None,
            provider_kind: None,
            vendor: None,
            model_name: None,
            error_code: None,
            duration_ms: None,
            retry_count: Some(0),
            relative_path: Some(dto.target_relative_path.clone()),
            metadata_json: json!({
                "backupId": dto.backup_id,
                "projectCount": dto.project_count,
                "assetCount": dto.asset_count,
                "containsSecrets": false,
            }),
        },
    )?;
    Ok(dto)
}

pub fn restore_workspace(
    database: &Database,
    workspace_root: &Path,
    request: RestoreWorkspaceRequest,
) -> Result<RestoreWorkspaceDto, String> {
    let backup_path = validate_workspace_backup_path(&request.backup_relative_path)?;
    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let backup_bucket_path = backup_path
        .strip_prefix("outputs/")
        .ok_or_else(|| "backup.restore_failed: backup path must be inside outputs.".to_string())?;
    let absolute_backup = storage
        .resolver()
        .resolve_existing_bucket_path(FileBucket::Output, backup_bucket_path)
        .map_err(|error| format!("backup.restore_failed: {error}"))?;
    let bytes = std::fs::read(&absolute_backup)
        .map_err(|error| format!("backup.restore_failed: {error}"))?;
    let entries =
        read_stored_zip_entries(&bytes).map_err(|error| error.replacen("import.", "backup.", 1))?;
    let manifest_text = read_backup_text_entry(&entries, "manifest.json")?;
    let db_export_text = read_backup_text_entry(&entries, "db_export.json")?;
    let manifest = serde_json::from_str::<serde_json::Value>(&manifest_text)
        .map_err(|error| format!("backup.restore_failed: invalid manifest: {error}"))?;
    validate_backup_manifest(&manifest)?;
    let db_export = serde_json::from_str::<serde_json::Value>(&db_export_text)
        .map_err(|error| format!("backup.restore_failed: invalid db_export: {error}"))?;
    reject_secret_scan(&[SecretScanInput {
        name: "backup manifest.json",
        content: &manifest_text,
    }])
    .map_err(|error| format!("backup.restore_failed: {error}"))?;
    reject_json_secrets(&db_export)
        .map_err(|error| format!("backup.restore_failed: backup db_export.json: {error}"))?;
    if db_export["schemaVersion"].as_i64() != Some(1)
        || db_export["exportKind"].as_str() != Some("workspace_backup")
    {
        return Err("backup.restore_failed: unsupported workspace backup schema.".to_string());
    }
    validate_backup_migrations(database, &db_export)?;

    let restore_id = create_restore_id();
    let backup_id = manifest["backupId"]
        .as_str()
        .unwrap_or("backup")
        .to_string();
    let projects = db_export
        .get("projects")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "backup.restore_failed: projects list is missing.".to_string())?;
    let mut restored_projects = Vec::new();
    for project in projects {
        let project_detail = serde_json::from_value::<ProjectDetailDto>(
            project
                .get("projectDetail")
                .cloned()
                .ok_or_else(|| "backup.restore_failed: projectDetail is missing.".to_string())?,
        )
        .map_err(|error| format!("backup.restore_failed: invalid projectDetail: {error}"))?;
        let storyboard = project
            .get("storyboard")
            .filter(|value| !value.is_null())
            .cloned()
            .map(serde_json::from_value::<StoryboardDto>)
            .transpose()
            .map_err(|error| format!("backup.restore_failed: invalid storyboard: {error}"))?;
        let source_project_id = project_detail.project.project_id.clone();
        let restored_project_id = create_restored_project_id();
        let restored_title = format!("{} 恢复", project_detail.project.title);
        let detail = ProjectRepository::new(database).create_with_id(
            restored_project_id.clone(),
            CreateProjectRequest {
                title: restored_title.clone(),
                workflow_type: project_detail.project.workflow_type,
                input_type: project_detail.project.input_type,
                topic: None,
                source_text: project_detail.project.source_text,
                source_text_path: None,
                content_language: project_detail.project.content_language,
                tone: project_detail.project.tone,
                aspect_ratio: project_detail.project.aspect_ratio,
                target_scene_count: project_detail.project.target_scene_count,
                segment_duration_seconds: project_detail.project.segment_duration_seconds,
                style_prompt: project_detail.project.style_prompt,
                active_pack_id: project_detail.project.active_pack_id,
                rule_refs: Some(project_detail.project.rule_refs),
                executable_refs: Some(project_detail.project.executable_refs),
                input_process_mode: project_detail.project.input_process_mode,
                input_options: Some(project_detail.project.input_options),
            },
        )?;
        if let Some(storyboard) = storyboard {
            import_storyboard_items(database, &restored_project_id, storyboard)?;
        }
        restored_projects.push(RestoredProjectDto {
            project_id: detail.project.project_id,
            source_project_id,
            title: restored_title,
        });
    }
    let restored_asset_count = import_backup_assets(database, &storage, &entries, &restore_id)?;
    let restored_template_file_count =
        restore_backup_template_and_prompt_files(workspace_root, &storage, &entries, &restore_id)?;

    Ok(RestoreWorkspaceDto {
        backup_id,
        restored_projects,
        restored_asset_count,
        restored_template_file_count,
        requires_secret_reentry: true,
    })
}

pub fn export_diagnostic_package(
    database: &Database,
    workspace_root: &Path,
    request: ExportDiagnosticPackageRequest,
) -> Result<ExportDiagnosticPackageDto, String> {
    if request.include_media.unwrap_or(false) {
        return Err(
            "diagnostic.media_permission_required: media attachment is not implemented yet."
                .to_string(),
        );
    }
    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let diagnostic_id = create_diagnostic_id();
    let migrations = database
        .applied_migrations()
        .map_err(|error| format!("diagnostic.export_failed: {error}"))?;
    let migration_summary = migrations
        .iter()
        .map(|migration| {
            json!({
                "version": migration.version,
                "checksum": migration.checksum,
            })
        })
        .collect::<Vec<_>>();
    let sidecars = ffmpeg_service::check_ffmpeg_sidecars(workspace_root)
        .map_err(|error| format!("diagnostic.export_failed: {error}"))?;
    let release_info = diagnostic_service::get_app_release_info();
    let diagnostic_release = json!({
        "appName": release_info.app_name,
        "productName": release_info.product_name,
        "version": release_info.version,
        "identifier": release_info.identifier,
        "targetPlatform": release_info.target_platform,
        "updateChannel": release_info.update_channel,
        "autoUpdateEnabled": release_info.auto_update_enabled,
        "updateFeedUrl": release_info.update_feed_url,
        "updateRequiresHttps": release_info.update_requires_https,
        "updatePackageSigningRequired": release_info.update_signature_required,
        "installerSigningRequired": release_info.installer_signature_required,
        "signedInstallerVerified": release_info.signed_installer_verified,
    });
    let projects = ProjectRepository::new(database).list(ListProjectsRequest {
        page: 1,
        page_size: 20,
        keyword: None,
        lifecycle: None,
        sort_by: None,
        sort_order: Some("desc".to_string()),
    })?;
    let summary = json!({
        "app": "vt-ai-short-video-maker",
        "appVersion": diagnostic_release["version"],
        "release": diagnostic_release,
        "diagnosticVersion": DIAGNOSTIC_PACKAGE_VERSION,
        "diagnosticId": diagnostic_id,
        "createdAt": current_timestamp_string(),
        "containsSensitiveData": false,
        "includesMedia": false,
        "system": {
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
        },
        "workspace": {
            "projectCountSample": projects.items.len(),
            "projects": projects.items.iter().map(|project| json!({
                "projectId": project.project_id,
                "title": project.title,
                "lifecycle": project.lifecycle,
                "updatedAt": project.updated_at,
                "latestTask": project.latest_task,
            })).collect::<Vec<_>>(),
        },
        "migrations": migration_summary,
        "migrationCount": migrations.len(),
        "sidecars": sidecars,
    });
    reject_json_secrets(&summary)
        .map_err(|error| format!("diagnostic.secret_detected: {error}"))?;
    let summary_text = pretty_json(&summary)?;
    reject_absolute_paths_in_export_text("diagnostics/summary.json", &summary_text)
        .map_err(|error| error.replacen("export.", "diagnostic.", 1))?;
    let mut entries = vec![ZipEntrySource::Bytes {
        entry_path: "summary.json".to_string(),
        content: summary_text.into_bytes(),
    }];
    let log_file_count = append_diagnostic_log_entries(workspace_root, &storage, &mut entries)?;
    for entry in &entries {
        if let ZipEntrySource::Bytes {
            entry_path,
            content,
        } = entry
        {
            if let Ok(text) = std::str::from_utf8(content) {
                reject_secret_scan(&[SecretScanInput {
                    name: entry_path,
                    content: text,
                }])
                .map_err(|error| format!("diagnostic.secret_detected: {error}"))?;
                reject_absolute_paths_in_export_text(entry_path, text)
                    .map_err(|error| error.replacen("export.", "diagnostic.", 1))?;
            }
        }
    }
    let target_bucket_relative =
        next_diagnostic_package_target_path(&storage, &diagnostic_id, false)?;
    let target_path = storage
        .resolver()
        .resolve_bucket_path_for_write(FileBucket::Output, &target_bucket_relative)
        .map_err(|error| format!("export.target_denied: {error}"))?;
    write_stored_zip(&target_path, entries)
        .map_err(|error| format!("export.target_denied: {error}"))?;

    Ok(ExportDiagnosticPackageDto {
        diagnostic_id,
        target_relative_path: format!("outputs/{target_bucket_relative}"),
        contains_secrets: false,
        includes_media: false,
        log_file_count,
    })
}

pub fn open_export_directory(
    database: &Database,
    workspace_root: &Path,
    request: OpenExportDirectoryRequest,
) -> Result<OpenExportDirectoryDto, String> {
    let record = ExportRepository::new(database)
        .get_export_record(&request.export_id)?
        .ok_or_else(|| {
            format!(
                "export.open_failed: export record not found: {}",
                request.export_id
            )
        })?;
    let target_relative_path = record
        .target_relative_path
        .as_deref()
        .ok_or_else(|| "export.open_failed: export target path is empty.".to_string())?;
    let normalized = validate_controlled_output_path(target_relative_path)?;
    let directory_relative_path = normalized
        .rsplit_once('/')
        .map(|(directory, _)| directory.to_string())
        .ok_or_else(|| "export.open_failed: export target directory is invalid.".to_string())?;

    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let output_bucket_path = directory_relative_path
        .strip_prefix("outputs/")
        .ok_or_else(|| "export.open_failed: export directory is outside outputs.".to_string())?;
    let directory = storage
        .resolver()
        .resolve_existing_bucket_path(FileBucket::Output, output_bucket_path)
        .map_err(|error| format!("export.open_failed: {error}"))?;
    if !directory.is_dir() {
        return Err("export.open_failed: export target parent is not a directory.".to_string());
    }

    Ok(OpenExportDirectoryDto {
        export_id: record.export_id,
        directory_relative_path,
    })
}

fn latest_succeeded_composition(
    database: &Database,
    project_id: &str,
) -> Result<CompositionTaskDto, String> {
    let composition = TaskRepository::new(database)
        .latest_composition_task_by_project(project_id)?
        .ok_or_else(|| {
            format!("export.composition_not_ready: Project {project_id} has no composition task.")
        })?;
    if composition.status != "succeeded" {
        return Err(format!(
            "export.composition_not_ready: Composition task {} is {}.",
            composition.task_id, composition.status
        ));
    }
    if composition.output_path.trim().is_empty() {
        return Err("export.final_missing: composition output path is empty.".to_string());
    }
    Ok(composition)
}

fn validate_output_relative_path(path: &str) -> Result<String, String> {
    let normalized = PathGuard::validate_relative_path(path)
        .map_err(|error| format!("export.target_denied: {error}"))?;
    if !normalized.starts_with("outputs/") {
        return Err("export.target_denied: final video must be inside outputs bucket.".to_string());
    }
    Ok(normalized)
}

fn validate_controlled_output_path(path: &str) -> Result<String, String> {
    let normalized = PathGuard::validate_relative_path(path)
        .map_err(|error| format!("export.target_denied: {error}"))?;
    let allowed = normalized.starts_with("outputs/user_exports/")
        || normalized.starts_with("outputs/project_packages/")
        || normalized.starts_with("outputs/backups/")
        || normalized.starts_with("outputs/diagnostics/");
    if !allowed {
        return Err(
            "export.target_denied: export path must be inside a controlled outputs directory."
                .to_string(),
        );
    }
    Ok(normalized)
}

fn validate_project_package_path(path: &str) -> Result<String, String> {
    let normalized = PathGuard::validate_relative_path(path)
        .map_err(|error| format!("import.package_invalid: {error}"))?;
    if !normalized.starts_with("outputs/project_packages/") || !normalized.ends_with(".zip") {
        return Err(
            "import.package_invalid: package must be a controlled outputs/project_packages zip."
                .to_string(),
        );
    }
    Ok(normalized)
}

fn validate_workspace_backup_path(path: &str) -> Result<String, String> {
    let normalized = PathGuard::validate_relative_path(path)
        .map_err(|error| format!("backup.restore_failed: {error}"))?;
    if !normalized.starts_with("outputs/backups/") || !normalized.ends_with(".zip") {
        return Err(
            "backup.restore_failed: backup must be a controlled outputs/backups zip.".to_string(),
        );
    }
    Ok(normalized)
}

fn resolve_existing_output_path(
    storage: &StorageService,
    workspace_relative_path: &str,
) -> Result<PathBuf, String> {
    let output_bucket_path = workspace_relative_path
        .strip_prefix("outputs/")
        .ok_or_else(|| "source path is outside outputs.".to_string())?;
    let source_path = storage
        .resolver()
        .resolve_existing_bucket_path(FileBucket::Output, output_bucket_path)?;
    if !source_path.is_file() {
        return Err("final video file is missing.".to_string());
    }
    Ok(source_path)
}

fn next_export_target_path(
    storage: &StorageService,
    project_id: &str,
    composition: &CompositionTaskDto,
    overwrite: bool,
) -> Result<String, String> {
    let project_segment = sanitize_file_segment(project_id);
    let composition_segment = sanitize_file_segment(&composition.task_id);
    let base_name = format!("{composition_segment}_final");
    let mut target = format!("user_exports/{project_segment}/{base_name}.mp4");

    if overwrite {
        return Ok(target);
    }

    for index in 1..=999 {
        let absolute_path = storage
            .resolver()
            .resolve_bucket_path_for_write(FileBucket::Output, &target)?;
        if !absolute_path.exists() {
            return Ok(target);
        }
        target = format!("user_exports/{project_segment}/{base_name}_{index:02}.mp4");
    }

    Err("export.target_denied: too many duplicate export files.".to_string())
}

fn next_project_package_target_path(
    storage: &StorageService,
    project_id: &str,
    overwrite: bool,
) -> Result<String, String> {
    let project_segment = sanitize_file_segment(project_id);
    let base_name = format!("{project_segment}_project_package");
    let mut target = format!("project_packages/{project_segment}/{base_name}.zip");

    if overwrite {
        return Ok(target);
    }

    for index in 1..=999 {
        let absolute_path = storage
            .resolver()
            .resolve_bucket_path_for_write(FileBucket::Output, &target)?;
        if !absolute_path.exists() {
            return Ok(target);
        }
        target = format!("project_packages/{project_segment}/{base_name}_{index:02}.zip");
    }

    Err("export.target_denied: too many duplicate project packages.".to_string())
}

fn next_workspace_backup_target_path(
    storage: &StorageService,
    backup_id: &str,
    overwrite: bool,
) -> Result<String, String> {
    let base_name = sanitize_file_segment(backup_id);
    let mut target = format!("backups/{base_name}.backup.zip");

    if overwrite {
        return Ok(target);
    }

    for index in 1..=999 {
        let absolute_path = storage
            .resolver()
            .resolve_bucket_path_for_write(FileBucket::Output, &target)?;
        if !absolute_path.exists() {
            return Ok(target);
        }
        target = format!("backups/{base_name}_{index:02}.backup.zip");
    }

    Err("export.target_denied: too many duplicate workspace backups.".to_string())
}

fn next_diagnostic_package_target_path(
    storage: &StorageService,
    diagnostic_id: &str,
    overwrite: bool,
) -> Result<String, String> {
    let base_name = sanitize_file_segment(diagnostic_id);
    let mut target = format!("diagnostics/{base_name}.diagnostic.zip");

    if overwrite {
        return Ok(target);
    }

    for index in 1..=999 {
        let absolute_path = storage
            .resolver()
            .resolve_bucket_path_for_write(FileBucket::Output, &target)?;
        if !absolute_path.exists() {
            return Ok(target);
        }
        target = format!("diagnostics/{base_name}_{index:02}.diagnostic.zip");
    }

    Err("export.target_denied: too many duplicate diagnostic packages.".to_string())
}

fn append_project_package_file_entries(
    storage: &StorageService,
    paths: &[String],
    entries: &mut Vec<ZipEntrySource>,
) -> Result<(), String> {
    for path in paths {
        append_workspace_file_if_exists(storage, path, entries)?;
    }
    Ok(())
}

fn append_workspace_file_if_exists(
    storage: &StorageService,
    workspace_relative_path: &str,
    entries: &mut Vec<ZipEntrySource>,
) -> Result<(), String> {
    let normalized = PathGuard::validate_relative_path(workspace_relative_path)
        .map_err(|error| format!("export.target_denied: {error}"))?;
    let Some((bucket, bucket_path)) = split_workspace_relative_path(&normalized) else {
        return Err(format!(
            "export.target_denied: unsupported package file path: {normalized}"
        ));
    };
    let absolute_path = match storage
        .resolver()
        .resolve_existing_bucket_path(bucket, bucket_path)
    {
        Ok(path) if path.is_file() => path,
        Ok(_) => return Ok(()),
        Err(_) => return Ok(()),
    };
    entries.push(ZipEntrySource::File {
        entry_path: normalized,
        source_path: absolute_path,
    });
    Ok(())
}

fn append_user_template_entries(
    storage: &StorageService,
    entries: &mut Vec<ZipEntrySource>,
) -> Result<(), String> {
    let template_root = storage
        .resolver()
        .bucket_root(FileBucket::Template)
        .join("user");
    if !template_root.is_dir() {
        return Ok(());
    }
    append_directory_entries(
        &template_root,
        "templates/user",
        storage.resolver().workspace_root(),
        entries,
    )
}

fn append_backup_directory_if_exists(
    workspace_root: &Path,
    source_dir: &Path,
    entry_prefix: &str,
    entries: &mut Vec<ZipEntrySource>,
) -> Result<(), String> {
    if !source_dir.is_dir() {
        return Ok(());
    }
    let metadata = std::fs::symlink_metadata(source_dir).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
        return Err("export.target_denied: backup source cannot include symlinks.".to_string());
    }
    append_directory_entries(source_dir, entry_prefix, workspace_root, entries)
}

fn append_directory_entries(
    current_dir: &Path,
    entry_prefix: &str,
    bucket_root: &Path,
    entries: &mut Vec<ZipEntrySource>,
) -> Result<(), String> {
    for entry in std::fs::read_dir(current_dir).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let metadata = entry.metadata().map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
            return Err(
                "export.target_denied: package source cannot include symlinks.".to_string(),
            );
        }
        if metadata.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            let next_prefix = format!("{entry_prefix}/{name}");
            PathGuard::validate_relative_path(&next_prefix)?;
            append_directory_entries(&path, &next_prefix, bucket_root, entries)?;
        } else if metadata.is_file() {
            let relative_to_bucket = path
                .strip_prefix(bucket_root)
                .map_err(|error| error.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            let entry_path = if relative_to_bucket.starts_with(entry_prefix) {
                relative_to_bucket
            } else {
                format!("{entry_prefix}/{relative_to_bucket}")
            };
            PathGuard::validate_relative_path(&entry_path)?;
            entries.push(ZipEntrySource::File {
                entry_path,
                source_path: path,
            });
        }
    }
    Ok(())
}

fn split_workspace_relative_path(path: &str) -> Option<(FileBucket, &str)> {
    path.strip_prefix("projects/")
        .map(|rest| (FileBucket::Project, rest))
        .or_else(|| {
            path.strip_prefix("assets/")
                .map(|rest| (FileBucket::Asset, rest))
        })
        .or_else(|| {
            path.strip_prefix("outputs/")
                .map(|rest| (FileBucket::Output, rest))
        })
        .or_else(|| {
            path.strip_prefix("templates/")
                .map(|rest| (FileBucket::Template, rest))
        })
}

fn reject_absolute_paths_in_export_text(name: &str, text: &str) -> Result<(), String> {
    let contains_windows_path = text.contains(":\\\\") || text.contains(":/");
    let contains_file_url = text.to_ascii_lowercase().contains("file://");
    if contains_windows_path || contains_file_url {
        return Err(format!(
            "export.target_denied: {name} contains an absolute or file URL path."
        ));
    }
    Ok(())
}

fn append_diagnostic_log_entries(
    workspace_root: &Path,
    storage: &StorageService,
    entries: &mut Vec<ZipEntrySource>,
) -> Result<usize, String> {
    let mut count = 0usize;
    for relative_path in ["logs/app.log", "logs/error.log", "logs/diagnostic.log"] {
        if append_diagnostic_log_if_exists(workspace_root, relative_path, entries)? {
            count += 1;
        }
    }
    let project_root = storage.resolver().bucket_root(FileBucket::Project);
    if project_root.is_dir() {
        for entry in std::fs::read_dir(&project_root).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let metadata = entry.metadata().map_err(|error| error.to_string())?;
            if !metadata.is_dir()
                || metadata.file_type().is_symlink()
                || is_reparse_point(&metadata)
            {
                continue;
            }
            let project_segment = entry.file_name().to_string_lossy().to_string();
            let relative_path = format!("projects/{project_segment}/exports/export.log");
            if append_diagnostic_log_if_exists(workspace_root, &relative_path, entries)? {
                count += 1;
            }
        }
    }
    Ok(count)
}

fn append_diagnostic_log_if_exists(
    workspace_root: &Path,
    workspace_relative_path: &str,
    entries: &mut Vec<ZipEntrySource>,
) -> Result<bool, String> {
    let safe_path = match PathGuard::new(workspace_root).safe_join_existing(workspace_relative_path)
    {
        Ok(path) => path,
        Err(_) => return Ok(false),
    };
    if !safe_path.absolute_path().is_file() {
        return Ok(false);
    }
    let content = read_tail_bytes(safe_path.absolute_path(), DIAGNOSTIC_LOG_TAIL_BYTES)?;
    let redacted = crate::security::secret_guard::redact_text(&content);
    reject_secret_scan(&[SecretScanInput {
        name: workspace_relative_path,
        content: &redacted,
    }])
    .map_err(|error| format!("diagnostic.secret_detected: {error}"))?;
    let entry_path = format!("logs/{}", workspace_relative_path.replace('/', "__"));
    PathGuard::validate_relative_path(&entry_path)
        .map_err(|error| format!("diagnostic.export_failed: {error}"))?;
    entries.push(ZipEntrySource::Bytes {
        entry_path,
        content: redacted.into_bytes(),
    });
    Ok(true)
}

fn read_tail_bytes(path: &Path, limit: usize) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    let start = bytes.len().saturating_sub(limit);
    Ok(String::from_utf8_lossy(&bytes[start..]).to_string())
}

fn write_export_success_log(
    workspace_root: &Path,
    record: &ExportRecordDto,
    message: &str,
) -> Result<(), String> {
    let log_record = StructuredFileLogRecord {
        trace_id: create_trace_id(),
        level: "info".to_string(),
        message: message.to_string(),
        project_id: Some(record.project_id.clone()),
        task_id: record.composition_task_id.clone(),
        task_step_id: None,
        step_kind: Some("export".to_string()),
        item_id: None,
        provider_id: None,
        provider_kind: None,
        vendor: None,
        model_name: None,
        error_code: None,
        duration_ms: None,
        retry_count: Some(0),
        relative_path: record.target_relative_path.clone(),
        metadata_json: json!({
            "exportId": record.export_id,
            "exportKind": record.export_kind,
            "status": record.status,
        }),
    };
    write_project_export_log(workspace_root, &record.project_id, &log_record)?;
    write_app_log(workspace_root, &log_record)
}

fn reject_secret_text_file_entries(entries: &[ZipEntrySource]) -> Result<(), String> {
    for entry in entries {
        let ZipEntrySource::File {
            entry_path,
            source_path,
        } = entry
        else {
            continue;
        };
        let lower = entry_path.to_ascii_lowercase();
        let is_text_like = lower.ends_with(".json")
            || lower.ends_with(".md")
            || lower.ends_with(".txt")
            || lower.ends_with(".yaml")
            || lower.ends_with(".yml")
            || lower.ends_with(".toml")
            || lower.ends_with(".js")
            || lower.ends_with(".ts");
        if !is_text_like {
            continue;
        }
        let content = std::fs::read_to_string(source_path)
            .map_err(|error| format!("export.target_denied: {error}"))?;
        reject_secret_scan(&[SecretScanInput {
            name: entry_path,
            content: &content,
        }])
        .map_err(|error| format!("export.secret_detected: {error}"))?;
        reject_absolute_paths_in_export_text(entry_path, &content)?;
    }
    Ok(())
}

fn read_backup_text_entry(entries: &[ZipReadEntry], entry_path: &str) -> Result<String, String> {
    read_zip_text_entry(entries, entry_path)
        .map_err(|error| error.replacen("import.", "backup.", 1))
}

fn validate_backup_manifest(manifest: &serde_json::Value) -> Result<(), String> {
    if manifest["app"].as_str() != Some("vt-ai-short-video-maker") {
        return Err("backup.restore_failed: backup app is not supported.".to_string());
    }
    if manifest["backupVersion"].as_i64() != Some(WORKSPACE_BACKUP_VERSION) {
        return Err("backup.restore_failed: backup version is not supported.".to_string());
    }
    if manifest["containsSecrets"].as_bool() != Some(false) {
        return Err("backup.restore_failed: backup declares secret content.".to_string());
    }
    Ok(())
}

fn validate_backup_migrations(
    database: &Database,
    db_export: &serde_json::Value,
) -> Result<(), String> {
    let current = database
        .applied_migrations()
        .map_err(|error| format!("backup.restore_failed: {error}"))?;
    let current_latest = current
        .iter()
        .map(|migration| migration.version)
        .max()
        .unwrap_or(0);
    let backup_latest = db_export
        .get("migrations")
        .and_then(serde_json::Value::as_array)
        .and_then(|items| {
            items
                .iter()
                .filter_map(|item| item.get("version").and_then(serde_json::Value::as_i64))
                .max()
        })
        .unwrap_or(0);
    if backup_latest > current_latest {
        return Err(format!(
            "backup.restore_failed: backup migration version {backup_latest} is newer than current {current_latest}."
        ));
    }
    Ok(())
}

fn import_backup_assets(
    database: &Database,
    storage: &StorageService,
    entries: &[ZipReadEntry],
    restore_id: &str,
) -> Result<usize, String> {
    let repository = AssetRepository::new(database);
    let restore_segment = sanitize_file_segment(restore_id);
    let mut count = 0usize;
    for entry in entries
        .iter()
        .filter(|entry| entry.entry_path.starts_with("assets/"))
    {
        let bucket_path = entry
            .entry_path
            .strip_prefix("assets/")
            .ok_or_else(|| "backup.restore_failed: invalid asset path.".to_string())?;
        let restored_relative = format!("restored/{restore_segment}/{bucket_path}");
        storage
            .write_bytes(
                FileBucket::Asset,
                &restored_relative,
                &entry.content,
                FileAccessPolicy::WriteProject,
            )
            .map_err(|error| format!("backup.restore_failed: {error}"))?;
        let asset_id = format!("asset_restored_{restore_segment}_{}", count + 1);
        repository.insert_asset(&NewAssetRecord {
            asset_id: asset_id.clone(),
            kind: "restored_workspace_asset".to_string(),
            relative_path: format!("assets/{restored_relative}"),
            source_kind: "workspace_backup_restore".to_string(),
            mime_type: None,
            size_bytes: Some(i64::try_from(entry.content.len()).unwrap_or(i64::MAX)),
            checksum: None,
            is_builtin: false,
            metadata: json!({
                "sourceEntryPath": entry.entry_path,
                "restoreId": restore_id,
            }),
        })?;
        repository.create_reference(&NewAssetReferenceRecord {
            reference_id: format!("asset_ref_restored_{restore_segment}_{}", count + 1),
            asset_id,
            owner_kind: "workspace_restore".to_string(),
            owner_id: restore_id.to_string(),
            usage_kind: "restored_backup_asset".to_string(),
        })?;
        count += 1;
    }
    Ok(count)
}

fn restore_backup_template_and_prompt_files(
    workspace_root: &Path,
    storage: &StorageService,
    entries: &[ZipReadEntry],
    restore_id: &str,
) -> Result<usize, String> {
    let restore_segment = sanitize_file_segment(restore_id);
    let mut count = 0usize;
    for entry in entries.iter().filter(|entry| {
        entry.entry_path.starts_with("templates/user/")
            || entry.entry_path.starts_with("prompts/user/")
    }) {
        if let Some(template_path) = entry.entry_path.strip_prefix("templates/user/") {
            let restored_relative = format!("restored/{restore_segment}/{template_path}");
            storage
                .write_bytes(
                    FileBucket::Template,
                    &restored_relative,
                    &entry.content,
                    FileAccessPolicy::WriteProject,
                )
                .map_err(|error| format!("backup.restore_failed: {error}"))?;
            count += 1;
        } else if let Some(prompt_path) = entry.entry_path.strip_prefix("prompts/user/") {
            let relative = PathGuard::validate_relative_path(&format!(
                "prompts/restored/{restore_segment}/{prompt_path}"
            ))
            .map_err(|error| format!("backup.restore_failed: {error}"))?;
            let target = PathGuard::new(workspace_root)
                .safe_join_for_write(&relative)
                .map_err(|error| format!("backup.restore_failed: {error}"))?;
            std::fs::write(target.absolute_path(), &entry.content)
                .map_err(|error| format!("backup.restore_failed: {error}"))?;
            count += 1;
        }
    }
    Ok(count)
}

fn pretty_json(value: &serde_json::Value) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|error| error.to_string())
}

fn sanitize_file_segment(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    if sanitized.is_empty() {
        "untitled".to_string()
    } else {
        sanitized
    }
}

fn create_export_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("export_{nanos}")
}

fn current_timestamp_string() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    seconds.to_string()
}

fn create_imported_project_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("project_imported_{nanos}")
}

fn create_restored_project_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("project_restored_{nanos}")
}

fn create_backup_id() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    format!("workspace_backup_{seconds}")
}

fn create_restore_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("restore_{nanos}")
}

fn create_diagnostic_id() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    format!("diagnostic_{seconds}")
}

fn create_trace_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("trace_{nanos}")
}

enum ZipEntrySource {
    Bytes {
        entry_path: String,
        content: Vec<u8>,
    },
    File {
        entry_path: String,
        source_path: PathBuf,
    },
}

struct ZipReadEntry {
    entry_path: String,
    content: Vec<u8>,
}

struct ZipCentralRecord {
    entry_path: String,
    crc32: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    local_header_offset: u32,
}

fn read_stored_zip_entries(bytes: &[u8]) -> Result<Vec<ZipReadEntry>, String> {
    let mut entries = Vec::new();
    let mut offset = 0usize;
    while offset + 4 <= bytes.len() {
        let signature = read_u32_at(bytes, offset)?;
        if signature == 0x0201_4b50 || signature == 0x0605_4b50 {
            break;
        }
        if signature != 0x0403_4b50 {
            return Err("import.package_invalid: invalid zip local header.".to_string());
        }
        if offset + 30 > bytes.len() {
            return Err("import.package_invalid: truncated zip local header.".to_string());
        }
        let method = read_u16_at(bytes, offset + 8)?;
        if method != 0 {
            return Err(
                "import.package_invalid: compressed zip entries are not supported yet.".to_string(),
            );
        }
        let compressed_size = read_u32_at(bytes, offset + 18)? as usize;
        let uncompressed_size = read_u32_at(bytes, offset + 22)? as usize;
        if compressed_size != uncompressed_size {
            return Err("import.package_invalid: invalid stored zip size.".to_string());
        }
        let name_len = read_u16_at(bytes, offset + 26)? as usize;
        let extra_len = read_u16_at(bytes, offset + 28)? as usize;
        let name_start = offset + 30;
        let content_start = name_start
            .checked_add(name_len)
            .and_then(|value| value.checked_add(extra_len))
            .ok_or_else(|| "import.package_invalid: zip entry is too large.".to_string())?;
        let content_end = content_start
            .checked_add(compressed_size)
            .ok_or_else(|| "import.package_invalid: zip entry is too large.".to_string())?;
        if content_end > bytes.len() {
            return Err("import.package_invalid: truncated zip entry.".to_string());
        }
        let entry_path = std::str::from_utf8(&bytes[name_start..name_start + name_len])
            .map_err(|error| format!("import.package_invalid: invalid zip entry name: {error}"))?
            .to_string();
        PathGuard::validate_relative_path(&entry_path)
            .map_err(|error| format!("import.zip_slip_detected: {error}"))?;
        entries.push(ZipReadEntry {
            entry_path,
            content: bytes[content_start..content_end].to_vec(),
        });
        offset = content_end;
    }
    if entries.is_empty() {
        return Err("import.package_invalid: zip has no entries.".to_string());
    }
    Ok(entries)
}

fn read_zip_text_entry(entries: &[ZipReadEntry], entry_path: &str) -> Result<String, String> {
    let entry = entries
        .iter()
        .find(|entry| entry.entry_path == entry_path)
        .ok_or_else(|| format!("import.package_invalid: {entry_path} is missing."))?;
    String::from_utf8(entry.content.clone())
        .map_err(|error| format!("import.package_invalid: {entry_path} is not utf-8: {error}"))
}

fn validate_package_manifest(manifest: &serde_json::Value) -> Result<(), String> {
    if manifest["app"].as_str() != Some("vt-ai-short-video-maker") {
        return Err("import.package_invalid: package app is not supported.".to_string());
    }
    if manifest["packageVersion"].as_i64() != Some(1) {
        return Err("import.package_invalid: package version is not supported.".to_string());
    }
    if manifest["containsSecrets"].as_bool() != Some(false) {
        return Err("import.package_invalid: package declares secret content.".to_string());
    }
    Ok(())
}

fn import_storyboard_items(
    database: &Database,
    project_id: &str,
    storyboard: StoryboardDto,
) -> Result<(), String> {
    let items = storyboard
        .items
        .into_iter()
        .map(|mut item| {
            item.item_id = format!("{}_{}", project_id, sanitize_file_segment(&item.item_id));
            item.project_id = project_id.to_string();
            item.selected_image_id = None;
            item.selected_video_segment_id = None;
            item.image_candidates = vec![];
            item.video_segments = vec![];
            item.image_status = "pending".to_string();
            item.video_status = "pending".to_string();
            item.render_status = "pending".to_string();
            item.segment_status = "pending".to_string();
            item.downstream_reset_records = None;
            item
        })
        .collect::<Vec<_>>();
    SceneRepository::new(database).upsert_storyboard_items(items)?;
    Ok(())
}

fn import_package_assets(
    database: &Database,
    storage: &StorageService,
    entries: &[ZipReadEntry],
    project_id: &str,
) -> Result<usize, String> {
    let repository = AssetRepository::new(database);
    let mut count = 0usize;
    for entry in entries
        .iter()
        .filter(|entry| entry.entry_path.starts_with("assets/"))
    {
        let bucket_path = entry
            .entry_path
            .strip_prefix("assets/")
            .ok_or_else(|| "import.package_invalid: invalid asset path.".to_string())?;
        let imported_relative = format!(
            "imported/{}/{}",
            sanitize_file_segment(project_id),
            bucket_path
        );
        storage
            .write_bytes(
                FileBucket::Asset,
                &imported_relative,
                &entry.content,
                FileAccessPolicy::WriteProject,
            )
            .map_err(|error| format!("import.package_invalid: {error}"))?;
        let asset_id = format!(
            "asset_imported_{}_{}",
            sanitize_file_segment(project_id),
            count + 1
        );
        repository.insert_asset(&NewAssetRecord {
            asset_id: asset_id.clone(),
            kind: "imported_project_asset".to_string(),
            relative_path: format!("assets/{imported_relative}"),
            source_kind: "project_package_import".to_string(),
            mime_type: None,
            size_bytes: Some(i64::try_from(entry.content.len()).unwrap_or(i64::MAX)),
            checksum: None,
            is_builtin: false,
            metadata: json!({
                "sourceEntryPath": entry.entry_path,
                "importedProjectId": project_id,
            }),
        })?;
        repository.create_reference(&NewAssetReferenceRecord {
            reference_id: format!(
                "asset_ref_imported_{}_{}",
                sanitize_file_segment(project_id),
                count + 1
            ),
            asset_id,
            owner_kind: "project".to_string(),
            owner_id: project_id.to_string(),
            usage_kind: "imported_package_asset".to_string(),
        })?;
        count += 1;
    }
    Ok(count)
}

fn read_u16_at(bytes: &[u8], offset: usize) -> Result<u16, String> {
    let slice = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| "import.package_invalid: truncated zip integer.".to_string())?;
    Ok(u16::from_le_bytes([slice[0], slice[1]]))
}

fn read_u32_at(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let slice = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| "import.package_invalid: truncated zip integer.".to_string())?;
    Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

fn write_stored_zip(target_path: &Path, entries: Vec<ZipEntrySource>) -> Result<(), String> {
    let mut file = std::fs::File::create(target_path).map_err(|error| error.to_string())?;
    let mut central_records = Vec::new();
    let mut offset = 0u64;
    for entry in entries {
        let (entry_path, content) = match entry {
            ZipEntrySource::Bytes {
                entry_path,
                content,
            } => (entry_path, content),
            ZipEntrySource::File {
                entry_path,
                source_path,
            } => (
                entry_path,
                std::fs::read(source_path).map_err(|error| error.to_string())?,
            ),
        };
        PathGuard::validate_relative_path(&entry_path)?;
        let entry_name = entry_path.as_bytes();
        let crc32 = crc32(&content);
        let size =
            u32::try_from(content.len()).map_err(|_| "zip entry is too large.".to_string())?;
        let local_header_offset =
            u32::try_from(offset).map_err(|_| "zip file is too large.".to_string())?;
        let mut local = Vec::new();
        write_u32(&mut local, 0x0403_4b50);
        write_u16(&mut local, 20);
        write_u16(&mut local, 0);
        write_u16(&mut local, 0);
        write_u16(&mut local, 0);
        write_u16(&mut local, 33);
        write_u32(&mut local, crc32);
        write_u32(&mut local, size);
        write_u32(&mut local, size);
        write_u16(
            &mut local,
            u16::try_from(entry_name.len())
                .map_err(|_| "zip entry name is too long.".to_string())?,
        );
        write_u16(&mut local, 0);
        local.extend_from_slice(entry_name);
        use std::io::Write;
        file.write_all(&local).map_err(|error| error.to_string())?;
        file.write_all(&content)
            .map_err(|error| error.to_string())?;
        offset += local.len() as u64 + content.len() as u64;
        central_records.push(ZipCentralRecord {
            entry_path,
            crc32,
            compressed_size: size,
            uncompressed_size: size,
            local_header_offset,
        });
    }

    let central_offset = u32::try_from(offset).map_err(|_| "zip file is too large.".to_string())?;
    let mut central = Vec::new();
    for record in &central_records {
        let entry_name = record.entry_path.as_bytes();
        write_u32(&mut central, 0x0201_4b50);
        write_u16(&mut central, 20);
        write_u16(&mut central, 20);
        write_u16(&mut central, 0);
        write_u16(&mut central, 0);
        write_u16(&mut central, 0);
        write_u16(&mut central, 33);
        write_u32(&mut central, record.crc32);
        write_u32(&mut central, record.compressed_size);
        write_u32(&mut central, record.uncompressed_size);
        write_u16(
            &mut central,
            u16::try_from(entry_name.len())
                .map_err(|_| "zip entry name is too long.".to_string())?,
        );
        write_u16(&mut central, 0);
        write_u16(&mut central, 0);
        write_u16(&mut central, 0);
        write_u16(&mut central, 0);
        write_u32(&mut central, 0);
        write_u32(&mut central, record.local_header_offset);
        central.extend_from_slice(entry_name);
    }
    let central_size = u32::try_from(central.len())
        .map_err(|_| "zip central directory is too large.".to_string())?;
    use std::io::Write;
    file.write_all(&central)
        .map_err(|error| error.to_string())?;
    let mut end = Vec::new();
    write_u32(&mut end, 0x0605_4b50);
    write_u16(&mut end, 0);
    write_u16(&mut end, 0);
    write_u16(
        &mut end,
        u16::try_from(central_records.len())
            .map_err(|_| "zip has too many entries.".to_string())?,
    );
    write_u16(
        &mut end,
        u16::try_from(central_records.len())
            .map_err(|_| "zip has too many entries.".to_string())?,
    );
    write_u32(&mut end, central_size);
    write_u32(&mut end, central_offset);
    write_u16(&mut end, 0);
    file.write_all(&end).map_err(|error| error.to_string())
}

fn write_u16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn write_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

#[cfg(windows)]
fn is_reparse_point(metadata: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_reparse_point(_metadata: &std::fs::Metadata) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::{
        backup_workspace, crc32, export_diagnostic_package, export_final_video,
        export_project_package, import_project_package, list_export_records, open_export_directory,
        restore_workspace, write_stored_zip, write_u16, write_u32, ZipEntrySource,
    };
    use crate::db::asset_repository::{AssetRepository, NewAssetRecord, NewAssetReferenceRecord};
    use crate::db::project_repository::ProjectRepository;
    use crate::db::scene_repository::SceneRepository;
    use crate::db::task_repository::{NewCompositionTaskRecord, TaskRepository};
    use crate::db::Database;
    use crate::domain::export::{
        BackupWorkspaceRequest, ExportDiagnosticPackageRequest, ExportFinalVideoRequest,
        ExportProjectPackageRequest, ImportProjectPackageRequest, ListExportRecordsRequest,
        OpenExportDirectoryRequest, RestoreWorkspaceRequest,
    };
    use crate::services::storage_service::{FileBucket, StorageService};
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn export_final_video_rejects_missing_composition() {
        let root = test_root("missing_composition");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        insert_project(&database, "project_missing_composition");

        let error = export_final_video(
            &database,
            &root.join("workspace"),
            ExportFinalVideoRequest {
                project_id: "project_missing_composition".to_string(),
                overwrite: None,
            },
        )
        .expect_err("missing composition should fail");

        assert!(error.starts_with("export.composition_not_ready:"));
        cleanup(root);
    }

    #[test]
    fn export_final_video_rejects_non_succeeded_composition() {
        let root = test_root("non_succeeded");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        insert_project(&database, "project_non_succeeded");
        upsert_composition(
            &database,
            "project_non_succeeded",
            "composition_pending",
            "pending",
            "outputs/exports/project_non_succeeded/final.mp4",
        );

        let error = export_final_video(
            &database,
            &root.join("workspace"),
            ExportFinalVideoRequest {
                project_id: "project_non_succeeded".to_string(),
                overwrite: None,
            },
        )
        .expect_err("pending composition should fail");

        assert!(error.starts_with("export.composition_not_ready:"));
        cleanup(root);
    }

    #[test]
    fn export_final_video_rejects_missing_final_file() {
        let root = test_root("missing_final");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        insert_project(&database, "project_missing_final");
        upsert_composition(
            &database,
            "project_missing_final",
            "composition_done",
            "succeeded",
            "outputs/exports/project_missing_final/final.mp4",
        );

        let error = export_final_video(
            &database,
            &root.join("workspace"),
            ExportFinalVideoRequest {
                project_id: "project_missing_final".to_string(),
                overwrite: None,
            },
        )
        .expect_err("missing final file should fail");

        assert!(error.starts_with("export.final_missing:"));
        assert!(!error.contains(&root.display().to_string()));
        cleanup(root);
    }

    #[test]
    fn export_final_video_rejects_escaped_output_path() {
        let root = test_root("escaped_output");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        insert_project(&database, "project_escaped");
        let result =
            TaskRepository::new(&database).upsert_composition_task(&NewCompositionTaskRecord {
                task_id: "composition_escaped".to_string(),
                project_id: "project_escaped".to_string(),
                segment_ids: vec![],
                output_path: "../escape.mp4".to_string(),
                enhancements: json!({}),
                status: "succeeded".to_string(),
                progress: 100,
                error_json: None,
            });

        assert!(result.is_err());
        cleanup(root);
    }

    #[test]
    fn export_final_video_copies_to_controlled_directory_and_records_history() {
        let root = test_root("copy_success");
        let workspace = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        insert_project(&database, "project_copy");
        let final_path = workspace.join("outputs/exports/project_copy/final.mp4");
        fs::create_dir_all(final_path.parent().expect("parent")).expect("output dir");
        fs::write(&final_path, "video").expect("final video should write");
        upsert_composition(
            &database,
            "project_copy",
            "composition_done",
            "succeeded",
            "outputs/exports/project_copy/final.mp4",
        );

        let record = export_final_video(
            &database,
            &workspace,
            ExportFinalVideoRequest {
                project_id: "project_copy".to_string(),
                overwrite: None,
            },
        )
        .expect("export should succeed");

        assert_eq!(record.status, "succeeded");
        assert_eq!(
            record.composition_task_id.as_deref(),
            Some("composition_done")
        );
        assert_eq!(
            record.source_relative_path.as_deref(),
            Some("outputs/exports/project_copy/final.mp4")
        );
        let target = record.target_relative_path.as_deref().expect("target path");
        assert!(target.starts_with("outputs/user_exports/project_copy/"));
        assert!(workspace.join(target).is_file());
        let export_log =
            fs::read_to_string(workspace.join("projects/project_copy/exports/export.log"))
                .expect("export log should read");
        assert!(export_log.contains("final_video"));
        assert!(export_log.contains("outputs/user_exports/project_copy/"));

        let records = list_export_records(
            &database,
            ListExportRecordsRequest {
                project_id: "project_copy".to_string(),
            },
        )
        .expect("records should list");
        assert_eq!(records.len(), 1);

        cleanup(root);
    }

    #[test]
    fn duplicate_export_uses_safe_suffix_without_overwrite() {
        let root = test_root("duplicate_suffix");
        let workspace = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        insert_project(&database, "project_dup");
        let final_path = workspace.join("outputs/exports/project_dup/final.mp4");
        fs::create_dir_all(final_path.parent().expect("parent")).expect("output dir");
        fs::write(&final_path, "video").expect("final video should write");
        upsert_composition(
            &database,
            "project_dup",
            "composition_dup",
            "succeeded",
            "outputs/exports/project_dup/final.mp4",
        );

        let first = export_final_video(
            &database,
            &workspace,
            ExportFinalVideoRequest {
                project_id: "project_dup".to_string(),
                overwrite: None,
            },
        )
        .expect("first export should succeed");
        let second = export_final_video(
            &database,
            &workspace,
            ExportFinalVideoRequest {
                project_id: "project_dup".to_string(),
                overwrite: None,
            },
        )
        .expect("second export should succeed");

        assert_ne!(first.target_relative_path, second.target_relative_path);
        assert!(second
            .target_relative_path
            .as_deref()
            .expect("target")
            .ends_with("_01.mp4"));

        cleanup(root);
    }

    #[test]
    fn open_export_directory_returns_only_controlled_parent() {
        let root = test_root("open_directory");
        let workspace = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        insert_project(&database, "project_open");
        let final_path = workspace.join("outputs/exports/project_open/final.mp4");
        fs::create_dir_all(final_path.parent().expect("parent")).expect("output dir");
        fs::write(&final_path, "video").expect("final video should write");
        upsert_composition(
            &database,
            "project_open",
            "composition_open",
            "succeeded",
            "outputs/exports/project_open/final.mp4",
        );
        let record = export_final_video(
            &database,
            &workspace,
            ExportFinalVideoRequest {
                project_id: "project_open".to_string(),
                overwrite: None,
            },
        )
        .expect("export should succeed");

        let directory = open_export_directory(
            &database,
            &workspace,
            OpenExportDirectoryRequest {
                export_id: record.export_id.clone(),
            },
        )
        .expect("directory should resolve");

        assert_eq!(directory.export_id, record.export_id);
        assert_eq!(
            directory.directory_relative_path,
            "outputs/user_exports/project_open"
        );

        cleanup(root);
    }

    #[test]
    fn export_project_package_writes_manifest_db_export_assets_and_record() {
        let root = test_root("package_success");
        let workspace = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        insert_project(&database, "project_package");
        insert_storyboard_item(&database, "project_package", "item_package");
        fs::create_dir_all(workspace.join("assets/source_material")).expect("asset dir");
        fs::write(workspace.join("assets/source_material/ref.png"), "asset").expect("asset");
        let asset_repository = AssetRepository::new(&database);
        asset_repository
            .insert_asset(&NewAssetRecord {
                asset_id: "asset_package".to_string(),
                kind: "source_material".to_string(),
                relative_path: "assets/source_material/ref.png".to_string(),
                source_kind: "user_import".to_string(),
                mime_type: Some("image/png".to_string()),
                size_bytes: Some(5),
                checksum: None,
                is_builtin: false,
                metadata: json!({ "displayName": "ref" }),
            })
            .expect("asset should save");
        asset_repository
            .create_reference(&NewAssetReferenceRecord {
                reference_id: "asset_ref_package".to_string(),
                asset_id: "asset_package".to_string(),
                owner_kind: "storyboard_item".to_string(),
                owner_id: "item_package".to_string(),
                usage_kind: "source_material".to_string(),
            })
            .expect("reference should save");

        let record = export_project_package(
            &database,
            &workspace,
            ExportProjectPackageRequest {
                project_id: "project_package".to_string(),
                overwrite: None,
            },
        )
        .expect("package export should succeed");

        assert_eq!(record.export_kind, "project_package");
        let target = record.target_relative_path.as_deref().expect("target path");
        assert!(target.starts_with("outputs/project_packages/project_package/"));
        let bytes = fs::read(workspace.join(target)).expect("zip should read");
        assert!(zip_contains_entry(&bytes, "manifest.json"));
        assert!(zip_contains_entry(&bytes, "db_export.json"));
        assert!(zip_contains_entry(
            &bytes,
            "projects/project_package/project.json"
        ));
        assert!(zip_contains_entry(&bytes, "assets/source_material/ref.png"));
        let export_log =
            fs::read_to_string(workspace.join("projects/project_package/exports/export.log"))
                .expect("export log should read");
        assert!(export_log.contains("project_package"));

        let records = list_export_records(
            &database,
            ListExportRecordsRequest {
                project_id: "project_package".to_string(),
            },
        )
        .expect("records should list");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].export_kind, "project_package");

        cleanup(root);
    }

    #[test]
    fn export_project_package_rejects_secret_like_project_snapshot() {
        let root = test_root("package_secret");
        let workspace = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        insert_project(&database, "project_secret");
        insert_storyboard_item(&database, "project_secret", "item_secret");
        database
            .with_connection(|connection| {
                connection.execute(
                    "UPDATE projects SET source_text = 'api_key = sk-abcdefghijklmnopqrstuvwxyz012345' WHERE project_id = 'project_secret'",
                    [],
                )
            })
            .expect("secret fixture should save");

        let error = export_project_package(
            &database,
            &workspace,
            ExportProjectPackageRequest {
                project_id: "project_secret".to_string(),
                overwrite: None,
            },
        )
        .expect_err("secret-like snapshot should block package export");

        assert!(error.starts_with("export.secret_detected:"));
        assert!(!workspace.join("outputs/project_packages").exists());
        cleanup(root);
    }

    #[test]
    fn duplicate_project_package_uses_safe_suffix() {
        let root = test_root("package_duplicate");
        let workspace = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        insert_project(&database, "project_package_dup");
        insert_storyboard_item(&database, "project_package_dup", "item_package_dup");

        let first = export_project_package(
            &database,
            &workspace,
            ExportProjectPackageRequest {
                project_id: "project_package_dup".to_string(),
                overwrite: None,
            },
        )
        .expect("first package should export");
        let second = export_project_package(
            &database,
            &workspace,
            ExportProjectPackageRequest {
                project_id: "project_package_dup".to_string(),
                overwrite: None,
            },
        )
        .expect("second package should export");

        assert_ne!(first.target_relative_path, second.target_relative_path);
        assert!(second
            .target_relative_path
            .as_deref()
            .expect("target")
            .ends_with("_01.zip"));

        cleanup(root);
    }

    #[test]
    fn import_project_package_creates_new_project_and_imports_assets() {
        let root = test_root("import_success");
        let workspace = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        insert_project(&database, "project_import_source");
        insert_storyboard_item(&database, "project_import_source", "item_import_source");
        fs::create_dir_all(workspace.join("assets/source_material")).expect("asset dir");
        fs::write(workspace.join("assets/source_material/ref.png"), "asset").expect("asset");
        let asset_repository = AssetRepository::new(&database);
        asset_repository
            .insert_asset(&NewAssetRecord {
                asset_id: "asset_import_source".to_string(),
                kind: "source_material".to_string(),
                relative_path: "assets/source_material/ref.png".to_string(),
                source_kind: "user_import".to_string(),
                mime_type: Some("image/png".to_string()),
                size_bytes: Some(5),
                checksum: None,
                is_builtin: false,
                metadata: json!({}),
            })
            .expect("asset should save");
        asset_repository
            .create_reference(&NewAssetReferenceRecord {
                reference_id: "asset_ref_import_source".to_string(),
                asset_id: "asset_import_source".to_string(),
                owner_kind: "storyboard_item".to_string(),
                owner_id: "item_import_source".to_string(),
                usage_kind: "source_material".to_string(),
            })
            .expect("reference should save");
        let exported = export_project_package(
            &database,
            &workspace,
            ExportProjectPackageRequest {
                project_id: "project_import_source".to_string(),
                overwrite: None,
            },
        )
        .expect("package export should succeed");

        let imported = import_project_package(
            &database,
            &workspace,
            ImportProjectPackageRequest {
                package_relative_path: exported.target_relative_path.expect("package path"),
            },
        )
        .expect("package import should succeed");

        assert_ne!(imported.project_id, "project_import_source");
        assert_eq!(imported.source_project_id, "project_import_source");
        assert_eq!(imported.imported_asset_count, 1);
        let imported_detail = ProjectRepository::new(&database)
            .get_detail(&imported.project_id)
            .expect("imported project should read")
            .expect("imported project should exist");
        assert!(imported_detail.project.title.ends_with("导入"));
        let imported_storyboard = SceneRepository::new(&database)
            .get_storyboard(&imported.project_id)
            .expect("storyboard should read")
            .expect("storyboard should exist");
        assert_eq!(imported_storyboard.items.len(), 1);
        assert!(workspace
            .join(format!(
                "assets/imported/{}/source_material/ref.png",
                imported.project_id
            ))
            .is_file());

        cleanup(root);
    }

    #[test]
    fn import_project_package_rejects_zip_slip_entry() {
        let root = test_root("import_zip_slip");
        let workspace = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let storage = StorageService::new(&workspace);
        storage
            .initialize_workspace()
            .expect("workspace should init");
        let target = storage
            .resolver()
            .resolve_bucket_path_for_write(
                FileBucket::Output,
                "project_packages/bad/bad_project_package.zip",
            )
            .expect("target should resolve");
        write_stored_zip(
            &target,
            vec![ZipEntrySource::Bytes {
                entry_path: "../manifest.json".to_string(),
                content: b"{}".to_vec(),
            }],
        )
        .expect_err("writer itself should reject unsafe entry");
        let unsafe_zip = build_unsafe_zip_for_test("../manifest.json", b"{}");
        fs::write(&target, unsafe_zip).expect("unsafe zip should write");

        let error = import_project_package(
            &database,
            &workspace,
            ImportProjectPackageRequest {
                package_relative_path: "outputs/project_packages/bad/bad_project_package.zip"
                    .to_string(),
            },
        )
        .expect_err("zip slip should be rejected");

        assert!(error.starts_with("import.zip_slip_detected:"));
        cleanup(root);
    }

    #[test]
    fn import_project_package_rejects_invalid_manifest() {
        let root = test_root("import_invalid_manifest");
        let workspace = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let storage = StorageService::new(&workspace);
        storage
            .initialize_workspace()
            .expect("workspace should init");
        let target = storage
            .resolver()
            .resolve_bucket_path_for_write(
                FileBucket::Output,
                "project_packages/bad/invalid_project_package.zip",
            )
            .expect("target should resolve");
        write_stored_zip(
            &target,
            vec![
                ZipEntrySource::Bytes {
                    entry_path: "manifest.json".to_string(),
                    content: br#"{"app":"other","packageVersion":1,"containsSecrets":false}"#
                        .to_vec(),
                },
                ZipEntrySource::Bytes {
                    entry_path: "db_export.json".to_string(),
                    content: br#"{"schemaVersion":1}"#.to_vec(),
                },
            ],
        )
        .expect("zip should write");

        let error = import_project_package(
            &database,
            &workspace,
            ImportProjectPackageRequest {
                package_relative_path: "outputs/project_packages/bad/invalid_project_package.zip"
                    .to_string(),
            },
        )
        .expect_err("invalid manifest should fail");

        assert!(error.starts_with("import.package_invalid:"));
        cleanup(root);
    }

    #[test]
    fn backup_workspace_writes_safe_package_with_projects_assets_templates_and_migrations() {
        let root = test_root("backup_success");
        let workspace = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        insert_project(&database, "project_backup");
        insert_storyboard_item(&database, "project_backup", "item_backup");
        fs::create_dir_all(workspace.join("assets/source_material")).expect("asset dir");
        fs::write(workspace.join("assets/source_material/ref.png"), "asset").expect("asset");
        fs::create_dir_all(workspace.join("templates/user/caption")).expect("template dir");
        fs::write(
            workspace.join("templates/user/caption/template.json"),
            r#"{"name":"caption"}"#,
        )
        .expect("template");
        let asset_repository = AssetRepository::new(&database);
        asset_repository
            .insert_asset(&NewAssetRecord {
                asset_id: "asset_backup".to_string(),
                kind: "source_material".to_string(),
                relative_path: "assets/source_material/ref.png".to_string(),
                source_kind: "user_import".to_string(),
                mime_type: Some("image/png".to_string()),
                size_bytes: Some(5),
                checksum: None,
                is_builtin: false,
                metadata: json!({}),
            })
            .expect("asset should save");

        let backup = backup_workspace(
            &database,
            &workspace,
            BackupWorkspaceRequest { overwrite: None },
        )
        .expect("backup should succeed");

        assert!(backup.target_relative_path.starts_with("outputs/backups/"));
        assert_eq!(backup.project_count, 1);
        assert_eq!(backup.asset_count, 1);
        assert!(!backup.contains_secrets);
        assert!(backup.requires_secret_reentry);
        let bytes = fs::read(workspace.join(&backup.target_relative_path)).expect("backup read");
        assert!(zip_contains_entry(&bytes, "manifest.json"));
        assert!(zip_contains_entry(&bytes, "db_export.json"));
        assert!(zip_contains_entry(&bytes, "assets/source_material/ref.png"));
        assert!(zip_contains_entry(
            &bytes,
            "templates/user/caption/template.json"
        ));
        let app_log = fs::read_to_string(workspace.join("logs/app.log")).expect("app log");
        assert!(app_log.contains("workspace backup exported"));
        assert!(app_log.contains("outputs/backups/"));

        cleanup(root);
    }

    #[test]
    fn restore_workspace_creates_new_projects_and_imports_assets_without_overwriting() {
        let root = test_root("restore_success");
        let workspace = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        insert_project(&database, "project_restore_source");
        insert_storyboard_item(&database, "project_restore_source", "item_restore_source");
        fs::create_dir_all(workspace.join("assets/source_material")).expect("asset dir");
        fs::write(workspace.join("assets/source_material/ref.png"), "asset").expect("asset");
        let asset_repository = AssetRepository::new(&database);
        asset_repository
            .insert_asset(&NewAssetRecord {
                asset_id: "asset_restore_source".to_string(),
                kind: "source_material".to_string(),
                relative_path: "assets/source_material/ref.png".to_string(),
                source_kind: "user_import".to_string(),
                mime_type: Some("image/png".to_string()),
                size_bytes: Some(5),
                checksum: None,
                is_builtin: false,
                metadata: json!({}),
            })
            .expect("asset should save");
        let backup = backup_workspace(
            &database,
            &workspace,
            BackupWorkspaceRequest { overwrite: None },
        )
        .expect("backup should succeed");

        let restored = restore_workspace(
            &database,
            &workspace,
            RestoreWorkspaceRequest {
                backup_relative_path: backup.target_relative_path,
            },
        )
        .expect("restore should succeed");

        assert_eq!(restored.restored_projects.len(), 1);
        let restored_project = &restored.restored_projects[0];
        assert_ne!(restored_project.project_id, "project_restore_source");
        assert_eq!(restored_project.source_project_id, "project_restore_source");
        assert_eq!(restored.restored_asset_count, 1);
        assert!(restored.requires_secret_reentry);
        let detail = ProjectRepository::new(&database)
            .get_detail(&restored_project.project_id)
            .expect("restored project should read")
            .expect("restored project should exist");
        assert!(detail.project.title.ends_with("恢复"));
        let storyboard = SceneRepository::new(&database)
            .get_storyboard(&restored_project.project_id)
            .expect("restored storyboard should read")
            .expect("restored storyboard should exist");
        assert_eq!(storyboard.items.len(), 1);
        assert!(workspace.join("assets/restored").is_dir());

        cleanup(root);
    }

    #[test]
    fn backup_workspace_rejects_secret_like_user_template() {
        let root = test_root("backup_secret");
        let workspace = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        insert_project(&database, "project_backup_secret");
        fs::create_dir_all(workspace.join("templates/user/leak")).expect("template dir");
        fs::write(
            workspace.join("templates/user/leak/template.json"),
            r#"{"api_key":"sk-abcdefghijklmnopqrstuvwxyz012345"}"#,
        )
        .expect("secret template");

        let error = backup_workspace(
            &database,
            &workspace,
            BackupWorkspaceRequest { overwrite: None },
        )
        .expect_err("secret-like template should block backup");

        assert!(error.starts_with("export.secret_detected:"));
        cleanup(root);
    }

    #[test]
    fn restore_workspace_rejects_zip_slip_backup() {
        let root = test_root("restore_zip_slip");
        let workspace = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let storage = StorageService::new(&workspace);
        storage
            .initialize_workspace()
            .expect("workspace should init");
        let target = storage
            .resolver()
            .resolve_bucket_path_for_write(FileBucket::Output, "backups/bad.backup.zip")
            .expect("target should resolve");
        let unsafe_zip = build_unsafe_zip_for_test("../manifest.json", b"{}");
        fs::write(&target, unsafe_zip).expect("unsafe zip should write");

        let error = restore_workspace(
            &database,
            &workspace,
            RestoreWorkspaceRequest {
                backup_relative_path: "outputs/backups/bad.backup.zip".to_string(),
            },
        )
        .expect_err("zip slip backup should fail");

        assert!(error.starts_with("backup.zip_slip_detected:"));
        cleanup(root);
    }

    #[test]
    fn export_diagnostic_package_writes_summary_and_redacted_logs() {
        let root = test_root("diagnostic_success");
        let workspace = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        insert_project(&database, "project_diagnostic");
        fs::create_dir_all(workspace.join("logs")).expect("logs dir");
        fs::write(
            workspace.join("logs/app.log"),
            "Authorization: Bearer sk-live-secret-token-123456\nok",
        )
        .expect("app log");

        let diagnostic = export_diagnostic_package(
            &database,
            &workspace,
            ExportDiagnosticPackageRequest {
                include_media: None,
            },
        )
        .expect("diagnostic should export");

        assert!(diagnostic
            .target_relative_path
            .starts_with("outputs/diagnostics/"));
        assert!(!diagnostic.contains_secrets);
        assert!(!diagnostic.includes_media);
        assert_eq!(diagnostic.log_file_count, 1);
        let bytes = fs::read(workspace.join(&diagnostic.target_relative_path))
            .expect("diagnostic zip should read");
        assert!(zip_contains_entry(&bytes, "summary.json"));
        assert!(zip_contains_entry(&bytes, "logs/logs__app.log"));
        let text = String::from_utf8_lossy(&bytes);
        assert!(!text.contains("sk-live"));
        assert!(text.contains("***REDACTED***"));
        assert!(text.contains("\"appVersion\""));
        assert!(text.contains(env!("CARGO_PKG_VERSION")));
        assert!(text.contains("\"autoUpdateEnabled\": false"));
        assert!(text.contains("\"updatePackageSigningRequired\": true"));
        assert!(text.contains("\"installerSigningRequired\": true"));

        cleanup(root);
    }

    #[test]
    fn export_diagnostic_package_rejects_media_without_permission_flow() {
        let root = test_root("diagnostic_media");
        let workspace = root.join("workspace");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");

        let error = export_diagnostic_package(
            &database,
            &workspace,
            ExportDiagnosticPackageRequest {
                include_media: Some(true),
            },
        )
        .expect_err("media diagnostic should require explicit flow");

        assert!(error.starts_with("diagnostic.media_permission_required:"));
        cleanup(root);
    }

    fn insert_project(database: &Database, project_id: &str) {
        database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    INSERT INTO projects (
                        project_id, title, workflow_type, input_type, input_process_mode,
                        aspect_ratio, target_scene_count, segment_duration_seconds,
                        content_language, lifecycle
                    )
                    VALUES (?1, ?1, 'image_to_video', 'topic', 'generate', '9:16', 1, 4, 'zh-CN', 'draft')
                    "#,
                    [project_id],
                )
            })
            .expect("project fixture should save");
    }

    fn insert_storyboard_item(database: &Database, project_id: &str, item_id: &str) {
        database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    INSERT INTO storyboard_items (
                        item_id, project_id, item_index, source_text, narration_text,
                        visual_goal, visual_description, characters_json,
                        character_ids_json, scene_description, image_prompt,
                        negative_prompt, video_prompt, duration_seconds,
                        status, image_status, audio_status, video_status,
                        subtitle_status, render_status, segment_status
                    )
                    VALUES (
                        ?1, ?2, 1, 'source', 'narration', 'goal',
                        'visual', '[]', '[]', 'scene', 'image prompt',
                        '', 'video prompt', 4, 'pending', 'pending',
                        'pending', 'pending', 'pending', 'pending', 'pending'
                    )
                    "#,
                    (item_id, project_id),
                )
            })
            .expect("storyboard item fixture should save");
    }

    fn zip_contains_entry(bytes: &[u8], entry_name: &str) -> bool {
        bytes
            .windows(entry_name.len())
            .any(|window| window == entry_name.as_bytes())
    }

    fn build_unsafe_zip_for_test(entry_path: &str, content: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        let entry_name = entry_path.as_bytes();
        write_u32(&mut bytes, 0x0403_4b50);
        write_u16(&mut bytes, 20);
        write_u16(&mut bytes, 0);
        write_u16(&mut bytes, 0);
        write_u16(&mut bytes, 0);
        write_u16(&mut bytes, 33);
        write_u32(&mut bytes, crc32(content));
        write_u32(&mut bytes, content.len() as u32);
        write_u32(&mut bytes, content.len() as u32);
        write_u16(&mut bytes, entry_name.len() as u16);
        write_u16(&mut bytes, 0);
        bytes.extend_from_slice(entry_name);
        bytes.extend_from_slice(content);
        bytes
    }

    fn upsert_composition(
        database: &Database,
        project_id: &str,
        task_id: &str,
        status: &str,
        output_path: &str,
    ) {
        TaskRepository::new(database)
            .upsert_composition_task(&NewCompositionTaskRecord {
                task_id: task_id.to_string(),
                project_id: project_id.to_string(),
                segment_ids: vec!["segment_1".to_string()],
                output_path: output_path.to_string(),
                enhancements: json!({}),
                status: status.to_string(),
                progress: if status == "succeeded" { 100 } else { 50 },
                error_json: (status != "succeeded").then(|| json!({ "code": "pending" })),
            })
            .expect("composition fixture should save");
    }

    fn test_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vt-ai-short-video-maker-export-{name}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn cleanup(path: PathBuf) {
        let _ = fs::remove_dir_all(path);
    }
}
