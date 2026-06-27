use crate::db::Database;
use crate::domain::diagnostic::{AppReleaseInfoDto, RuntimeCheckItemDto, RuntimeSelfCheckDto};
use crate::domain::template::{
    ListTemplateManifestsRequest, PreviewTemplateRequest, TemplateRenderDataDto,
};
use crate::services::{ffmpeg_service, storage_service::StorageService, template_service};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_app_release_info() -> AppReleaseInfoDto {
    AppReleaseInfoDto {
        app_name: env!("CARGO_PKG_NAME").to_string(),
        product_name: "VT AI Short Video Maker".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        identifier: "com.vt-ai-short-video-maker.app".to_string(),
        target_platform: "windows-x64".to_string(),
        update_channel: "manual".to_string(),
        auto_update_enabled: false,
        update_feed_url: None,
        update_requires_https: true,
        update_signature_required: true,
        installer_signature_required: true,
        signed_installer_verified: false,
    }
}

pub fn run_runtime_self_check(
    database: &Database,
    workspace_root: &Path,
) -> Result<RuntimeSelfCheckDto, String> {
    let workspace = check_workspace(workspace_root);
    let sqlite = check_sqlite(database);
    let ffmpeg = ffmpeg_service::check_ffmpeg_sidecars(workspace_root)?;
    let template_sidecar = template_service::check_template_sidecars(workspace_root)?;
    let template_manifest = check_template_manifest(workspace_root);
    let template_preview = check_template_preview(
        workspace_root,
        template_sidecar.ready,
        template_manifest.ready,
    );
    let ready = workspace.ready
        && sqlite.ready
        && ffmpeg.ready
        && template_sidecar.ready
        && template_manifest.ready
        && template_preview.ready;

    Ok(RuntimeSelfCheckDto {
        ready,
        checked_at: current_timestamp_string(),
        workspace,
        sqlite,
        ffmpeg,
        template_sidecar,
        template_manifest,
        template_preview,
    })
}

fn check_workspace(workspace_root: &Path) -> RuntimeCheckItemDto {
    let result = (|| -> Result<(), String> {
        StorageService::new(workspace_root).initialize_workspace()?;
        let probe = workspace_root.join("temp").join(".runtime-self-check");
        fs::write(&probe, "ok").map_err(|error| error.to_string())?;
        let text = fs::read_to_string(&probe).map_err(|error| error.to_string())?;
        let _ = fs::remove_file(&probe);
        if text == "ok" {
            Ok(())
        } else {
            Err("workspace readback mismatch.".to_string())
        }
    })();
    item_from_result("workspace", result, "runtime.workspace_failed")
}

fn check_sqlite(database: &Database) -> RuntimeCheckItemDto {
    let result = database
        .with_connection(|connection| {
            connection.query_row("SELECT 1", [], |row| row.get::<_, i64>(0))?;
            Ok(())
        })
        .map_err(|error| error.to_string());
    item_from_result("sqlite", result, "runtime.sqlite_failed")
}

fn check_template_manifest(workspace_root: &Path) -> RuntimeCheckItemDto {
    let result = template_service::list_template_manifests(
        workspace_root,
        ListTemplateManifestsRequest {
            aspect_ratio: Some("vertical_9_16".to_string()),
            template_type: Some("cover".to_string()),
            source_type: Some("builtin".to_string()),
        },
    )
    .and_then(|items| {
        if items
            .iter()
            .any(|item| item.template_id == "knowledge_bold")
        {
            Ok(())
        } else {
            Err("builtin cover template knowledge_bold was not found.".to_string())
        }
    });
    item_from_result(
        "templateManifest",
        result,
        "runtime.template_manifest_failed",
    )
}

fn check_template_preview(
    workspace_root: &Path,
    template_sidecar_ready: bool,
    template_manifest_ready: bool,
) -> RuntimeCheckItemDto {
    if !template_manifest_ready {
        return RuntimeCheckItemDto {
            key: "templatePreview".to_string(),
            ready: false,
            skipped: true,
            error_code: Some("runtime.template_manifest_required".to_string()),
            message: Some("template manifest check failed.".to_string()),
        };
    }
    if !template_sidecar_ready {
        return RuntimeCheckItemDto {
            key: "templatePreview".to_string(),
            ready: false,
            skipped: true,
            error_code: Some("template.sidecar_missing".to_string()),
            message: Some("template sidecar is not ready.".to_string()),
        };
    }

    let request = PreviewTemplateRequest {
        template_id: "knowledge_bold".to_string(),
        aspect_ratio: "vertical_9_16".to_string(),
        template_type: "cover".to_string(),
        params: serde_json::json!({
            "cover_title": "Self Check",
            "accent_color": "#FFD54A",
            "position": "bottom"
        }),
        data: TemplateRenderDataDto {
            title: Some("Self Check".to_string()),
            narration: Some("Runtime template preview".to_string()),
            subtitle_chunks: None,
            image_path: None,
            video_frame_path: None,
            character_names: None,
        },
    };
    let result = template_service::preview_template(workspace_root, request).map(|_| ());
    item_from_result("templatePreview", result, "runtime.template_preview_failed")
}

fn item_from_result(
    key: &str,
    result: Result<(), String>,
    error_code: &str,
) -> RuntimeCheckItemDto {
    match result {
        Ok(()) => RuntimeCheckItemDto {
            key: key.to_string(),
            ready: true,
            skipped: false,
            error_code: None,
            message: None,
        },
        Err(error) => RuntimeCheckItemDto {
            key: key.to_string(),
            ready: false,
            skipped: false,
            error_code: Some(error_code.to_string()),
            message: Some(error),
        },
    }
}

fn current_timestamp_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

#[cfg(test)]
mod tests {
    use super::{get_app_release_info, run_runtime_self_check};
    use crate::services::startup_service;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn runtime_self_check_reports_missing_sidecars_without_blocking_core_checks() {
        let root = test_root("runtime_self_check");
        let (database, workspace_root, _report) =
            startup_service::initialize_app_runtime(&root).expect("startup should complete");

        let status =
            run_runtime_self_check(&database, &workspace_root).expect("self check should run");
        assert!(!status.ready);
        assert!(status.workspace.ready);
        assert!(status.sqlite.ready);
        assert!(status.template_manifest.ready);
        assert!(!status.ffmpeg.ready);
        assert!(!status.template_sidecar.ready);
        assert!(status.template_preview.skipped);
        assert_eq!(
            status.template_preview.error_code.as_deref(),
            Some("template.sidecar_missing")
        );

        cleanup(root);
    }

    #[test]
    fn app_release_info_keeps_updates_manual_and_signature_required() {
        let info = get_app_release_info();

        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(info.target_platform, "windows-x64");
        assert!(!info.auto_update_enabled);
        assert_eq!(info.update_feed_url, None);
        assert!(info.update_requires_https);
        assert!(info.update_signature_required);
        assert!(info.installer_signature_required);
        assert!(!info.signed_installer_verified);
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
