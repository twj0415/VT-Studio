mod commands;
mod core;
mod db;
mod domain;
mod providers;
mod security;
mod services;
mod utils;

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().map_err(|error| {
                Box::<dyn std::error::Error>::from(format!(
                    "Failed to resolve app data directory: {error}"
                ))
            })?;
            let (database, workspace_root, _report) =
                services::startup_service::initialize_app_runtime(&app_data_dir).map_err(
                    |error| {
                        Box::<dyn std::error::Error>::from(format!(
                            "Failed to initialize application runtime: {error}"
                        ))
                    },
                )?;
            app.manage(core::app_state::AppState::new(
                database,
                workspace_root,
                services::keyring_service::KeyringService::system(),
            ));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::project::create_project,
            commands::project::list_projects,
            commands::project::get_project_detail,
            commands::project::update_project,
            commands::project::generate_project_cover,
            commands::project::replace_project_cover_image,
            commands::novel::import_novel,
            commands::novel::list_novel_chapters,
            commands::novel::update_novel_chapter_event,
            commands::novel::mark_novel_chapter_event_failed,
            commands::novel::retry_novel_chapter_event,
            commands::long_content::save_long_content_plan,
            commands::long_content::list_long_content_plans,
            commands::long_content::approve_long_content_plan,
            commands::long_content::reject_long_content_plan,
            commands::digital_human::get_digital_human_project_state,
            commands::digital_human::mark_digital_human_tts_succeeded,
            commands::digital_human::mark_digital_human_tts_failed,
            commands::digital_human::start_digital_human_video,
            commands::material_edit::get_material_edit_project_state,
            commands::material_edit::save_material_analysis_suggestion,
            commands::material_edit::approve_material_analysis_suggestion,
            commands::material_edit::reject_material_analysis_suggestion,
            commands::material_edit::bind_storyboard_material,
            commands::material_edit::mark_storyboard_no_material_needed,
            commands::material_edit::validate_material_storyboard_coverage,
            commands::local_memory::upsert_local_memory_entry,
            commands::local_memory::list_local_memory_entries,
            commands::local_memory::create_local_memory_retrieval,
            commands::local_memory::approve_local_memory_candidate,
            commands::local_memory::reject_local_memory_candidate,
            commands::local_memory::build_local_memory_context,
            commands::image_slideshow::get_image_slideshow_project_state,
            commands::image_slideshow::register_template_motion_segment,
            commands::image_slideshow::validate_image_slideshow_segments,
            commands::canvas_edit::create_canvas_edit_candidate,
            commands::scene::get_storyboard,
            commands::scene::update_storyboard_item,
            commands::scene::batch_update_storyboard_items,
            commands::scene::apply_script_draft,
            commands::scene::reorder_storyboard_items,
            commands::scene::generate_image_prompts,
            commands::scene::start_image_generation,
            commands::scene::build_character_resource_plan,
            commands::scene::start_image_asset_generation,
            commands::scene::select_image_candidate,
            commands::scene::clear_historical_image_candidates,
            commands::scene::start_tts_generation,
            commands::scene::replace_storyboard_audio,
            commands::scene::probe_storyboard_audio,
            commands::scene::generate_subtitles,
            commands::scene::update_storyboard_subtitles,
            commands::scene::start_video_generation,
            commands::scene::select_video_segment,
            commands::scene::clear_historical_video_segments,
            commands::task::create_task,
            commands::task::start_task,
            commands::task::cancel_task,
            commands::task::resume_task,
            commands::task::retry_task_step,
            commands::task::get_task_detail,
            commands::task::list_tasks,
            commands::task::approve_task_step,
            commands::task::start_composition,
            commands::template::list_template_manifests,
            commands::template::validate_template_params,
            commands::template::preview_template,
            commands::template::render_template,
            commands::template::check_template_sidecars,
            commands::export::export_final_video,
            commands::export::export_project_package,
            commands::export::import_project_package,
            commands::export::backup_workspace,
            commands::export::restore_workspace,
            commands::export::export_diagnostic_package,
            commands::export::list_export_records,
            commands::export::open_export_directory,
            commands::diagnostic::get_app_release_info,
            commands::diagnostic::run_runtime_self_check,
            commands::config::get_app_config,
            commands::config::update_app_config,
            commands::config::save_provider_secret,
            commands::config::delete_provider_secret,
            commands::config::has_provider_secret,
            commands::config::list_provider_configs,
            commands::config::upsert_provider_config,
            commands::config::delete_provider_config,
            commands::config::list_provider_models,
            commands::config::upsert_provider_model,
            commands::config::delete_provider_model,
            commands::config::list_workflow_presets,
            commands::config::upsert_workflow_preset,
            commands::config::delete_workflow_preset,
            commands::provider::provider_dry_run,
            commands::provider::provider_generation_test,
            commands::dictionary::get_dictionary,
            commands::dictionary::list_dictionaries,
            commands::media::list_executable_media_options,
            commands::media::import_asset,
            commands::media::list_assets,
            commands::media::delete_asset,
            commands::media::create_asset_reference,
            commands::media::list_asset_references,
            commands::media::delete_asset_reference,
            commands::media::collect_project_asset_paths,
            commands::media::read_asset_preview,
            commands::media::probe_media,
            commands::ffmpeg::check_ffmpeg_sidecars,
            commands::style::get_project_style_bible,
            commands::style::list_style_presets,
            commands::style::upsert_project_style_bible,
            commands::style::apply_style_preset,
            commands::style::bind_style_reference_asset,
            commands::style::analyze_style_reference_image,
            commands::style::build_image_prompt_preview,
            commands::character::list_project_character_bibles,
            commands::character::upsert_project_character_bible,
            commands::character::delete_project_character_bible,
            commands::character::bind_character_reference_asset,
            commands::location::list_project_location_bibles,
            commands::location::upsert_project_location_bible,
            commands::location::delete_project_location_bible,
            commands::location::bind_location_reference_asset,
            commands::prompt::list_creative_rules,
            commands::prompt::get_creative_rule,
            commands::prompt::clone_creative_rule_to_user,
            commands::prompt::save_user_creative_rule,
            commands::prompt::set_user_creative_rule_enabled,
            commands::prompt::delete_user_creative_rule,
            commands::prompt::validate_structured_output,
            commands::video_pack::list_video_packs,
            commands::video_pack::get_video_pack,
            commands::video_pack::clone_video_pack_to_user,
            commands::video_pack::upsert_user_video_pack,
            commands::video_pack::set_video_pack_enabled,
            commands::video_pack::delete_user_video_pack,
            commands::video_pack::save_project_config_as_video_pack,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
