mod commands;
mod core;
mod db;
mod domain;
mod providers;
mod security;
mod services;
mod utils;

use std::io;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().map_err(|error| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to resolve app data directory: {error}"),
                )
            })?;
            let database_path = app_data_dir.join("vt-ai-short-video-maker.sqlite3");
            let workspace_root = app_data_dir.join("workspace");
            std::fs::create_dir_all(&workspace_root)?;
            let database = db::Database::open(database_path).map_err(|error| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to initialize SQLite database: {error}"),
                )
            })?;
            app.manage(core::app_state::AppState::new(database, workspace_root));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::project::create_project,
            commands::project::list_projects,
            commands::project::get_project_detail,
            commands::project::update_project,
            commands::scene::get_storyboard,
            commands::scene::update_storyboard_item,
            commands::scene::batch_update_storyboard_items,
            commands::scene::reorder_storyboard_items,
            commands::scene::generate_image_prompts,
            commands::scene::start_image_generation,
            commands::scene::select_image_candidate,
            commands::scene::start_video_generation,
            commands::scene::select_video_segment,
            commands::task::create_task,
            commands::task::get_task_detail,
            commands::task::approve_task_step,
            commands::task::start_composition,
            commands::config::get_app_config,
            commands::config::update_app_config,
            commands::dictionary::get_dictionary,
            commands::dictionary::list_dictionaries,
            commands::media::list_executable_media_options,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
