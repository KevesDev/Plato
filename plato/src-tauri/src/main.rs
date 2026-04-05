#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod commands;
mod engine;

use engine::PlatoEngineState;

fn main() {
    tauri::Builder::default()
        .manage(PlatoEngineState::default())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::workspace::scan_workspace_directory,
            commands::onboarding::check_model_status,
            commands::onboarding::start_model_download,
            commands::lifecycle::initialize_engine,
            commands::chat::stream_chat_completion
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}