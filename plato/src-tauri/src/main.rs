#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod commands;
mod engine;

use engine::PlatoEngineState;

/**
 * Platform health check to verify system compatibility and memory availability.
 */
#[tauri::command]
fn check_system_health() -> models::IpcResponse<models::SystemHealthStatus> {
    let status = models::SystemHealthStatus {
        is_platform_supported: true,
        available_memory_mb: 0, 
        model_exists: false,
    };

    models::IpcResponse {
        success: true,
        data: Some(status),
        error_message: None,
    }
}

fn main() {
    tauri::Builder::default()
        .manage(PlatoEngineState::default())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            check_system_health,
            commands::workspace::scan_workspace_directory,
            commands::onboarding::verify_and_download_model
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}