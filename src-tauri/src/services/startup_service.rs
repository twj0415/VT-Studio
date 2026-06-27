use crate::db::config_repository::ConfigRepository;
use crate::db::task_repository::TaskRepository;
use crate::db::Database;
use crate::domain::prompt::ListCreativeRulesRequest;
use crate::domain::template::ListTemplateManifestsRequest;
use crate::services::{
    ffmpeg_service, prompt_service, storage_service::StorageService, template_service,
};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartupInitReport {
    pub app_data_dir: PathBuf,
    pub workspace_root: PathBuf,
    pub database_path: PathBuf,
    pub workspace_initialized: bool,
    pub config_initialized: bool,
    pub template_count: usize,
    pub creative_rule_count: usize,
    pub ffmpeg_ready: bool,
    pub template_sidecar_ready: bool,
}

pub fn initialize_app_runtime(
    app_data_dir: &Path,
) -> Result<(Database, PathBuf, StartupInitReport), String> {
    fs::create_dir_all(app_data_dir).map_err(|error| error.to_string())?;

    let workspace_root = app_data_dir.join("workspace");
    let storage = StorageService::new(&workspace_root);
    storage.initialize_workspace()?;

    copy_packaged_sidecars_to_workspace(app_data_dir, &workspace_root)?;

    let database_path = app_data_dir.join("vt-ai-short-video-maker.sqlite3");
    let database = Database::open(&database_path).map_err(|error| error.to_string())?;

    ConfigRepository::new(&database).ensure_defaults()?;
    let template_count = template_service::list_template_manifests(
        &workspace_root,
        ListTemplateManifestsRequest {
            aspect_ratio: None,
            template_type: None,
            source_type: Some("builtin".to_string()),
        },
    )?
    .len();
    let creative_rule_count = prompt_service::list_creative_rules(
        &database,
        &workspace_root,
        ListCreativeRulesRequest {
            source_type: Some("builtin".to_string()),
            module: None,
        },
    )?
    .len();
    TaskRepository::new(&database)
        .scan_and_mark_recoverable_tasks()
        .map_err(|error| error.to_string())?;

    let ffmpeg_ready = ffmpeg_service::check_ffmpeg_sidecars(&workspace_root)
        .map(|status| status.ready)
        .unwrap_or(false);
    let template_sidecar_ready = template_service::check_template_sidecars(&workspace_root)
        .map(|status| status.ready)
        .unwrap_or(false);

    let report = StartupInitReport {
        app_data_dir: app_data_dir.to_path_buf(),
        workspace_root: workspace_root.clone(),
        database_path,
        workspace_initialized: true,
        config_initialized: true,
        template_count,
        creative_rule_count,
        ffmpeg_ready,
        template_sidecar_ready,
    };

    eprintln!(
        "startup.init: workspace={}, templates={}, creative_rules={}, ffmpeg_ready={}, template_sidecar_ready={}",
        report.workspace_root.display(),
        report.template_count,
        report.creative_rule_count,
        report.ffmpeg_ready,
        report.template_sidecar_ready
    );

    Ok((database, workspace_root, report))
}

fn copy_packaged_sidecars_to_workspace(
    app_data_dir: &Path,
    workspace_root: &Path,
) -> Result<(), String> {
    let resource_bin = app_data_dir.join("resources").join("bin");
    if !resource_bin.is_dir() {
        return Ok(());
    }

    let sidecar_root = workspace_root.join("sidecars");
    fs::create_dir_all(&sidecar_root).map_err(|error| error.to_string())?;
    for entry in fs::read_dir(&resource_bin).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let source = entry.path();
        let file_name = entry
            .file_name()
            .into_string()
            .map_err(|_| "packaged sidecar file name must be valid unicode.".to_string())?;
        if file_name == "README.md" {
            continue;
        }
        let target = sidecar_root.join(&file_name);
        copy_if_missing(&source, &target)?;
    }

    Ok(())
}

fn copy_if_missing(source: &Path, target: &Path) -> Result<(), String> {
    if target.exists() {
        return Ok(());
    }
    if source.is_dir() {
        copy_directory(source, target)
    } else if source.is_file() {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::copy(source, target).map_err(|error| error.to_string())?;
        Ok(())
    } else {
        Ok(())
    }
}

fn copy_directory(source: &Path, target: &Path) -> Result<(), String> {
    fs::create_dir_all(target).map_err(|error| error.to_string())?;
    for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let child_source = entry.path();
        let child_target = target.join(entry.file_name());
        copy_if_missing(&child_source, &child_target)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::initialize_app_runtime;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn startup_initializes_workspace_database_templates_and_rules() {
        let root = test_root("startup_init");
        let (_database, workspace_root, report) =
            initialize_app_runtime(&root).expect("startup init should complete");

        assert_eq!(workspace_root, root.join("workspace"));
        assert!(workspace_root.join("projects").is_dir());
        assert!(workspace_root.join("assets").is_dir());
        assert!(workspace_root.join("outputs").is_dir());
        assert!(workspace_root.join("templates").is_dir());
        assert!(workspace_root.join("prompts/builtin").is_dir());
        assert!(root.join("vt-ai-short-video-maker.sqlite3").is_file());
        assert!(report.workspace_initialized);
        assert!(report.config_initialized);
        assert!(report.template_count >= 3);
        assert!(report.creative_rule_count >= 1);
        assert!(!report.ffmpeg_ready);
        assert!(!report.template_sidecar_ready);

        let (_database, _workspace_root, second_report) =
            initialize_app_runtime(&root).expect("startup init should be idempotent");
        assert!(second_report.template_count >= 3);
        assert!(second_report.creative_rule_count >= 1);

        cleanup(root);
    }

    #[test]
    fn startup_copies_packaged_sidecars_without_overwriting_existing_runtime_files() {
        let root = test_root("startup_sidecar_copy");
        let resource_bin = root.join("resources/bin");
        fs::create_dir_all(resource_bin.join("chromium")).expect("resource dir should write");
        fs::write(resource_bin.join("ffmpeg.exe"), "packaged").expect("ffmpeg should write");
        fs::write(resource_bin.join("chromium/chrome.exe"), "chrome").expect("chrome should write");

        let runtime_ffmpeg = root.join("workspace/sidecars/ffmpeg.exe");
        fs::create_dir_all(runtime_ffmpeg.parent().expect("parent should exist"))
            .expect("runtime dir should write");
        fs::write(&runtime_ffmpeg, "existing").expect("runtime ffmpeg should write");

        let (_database, workspace_root, _report) =
            initialize_app_runtime(&root).expect("startup init should complete");
        assert_eq!(
            fs::read_to_string(workspace_root.join("sidecars/ffmpeg.exe"))
                .expect("runtime ffmpeg should read"),
            "existing"
        );
        assert!(workspace_root
            .join("sidecars/chromium/chrome.exe")
            .is_file());

        cleanup(root);
    }

    fn test_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vt-ai-short-video-maker-{name}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn cleanup(path: PathBuf) {
        let _ = fs::remove_dir_all(path);
    }
}
