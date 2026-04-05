// Prevents additional console window on Windows in release, do not remove.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod engine;
mod models;

use commands::*;
use engine::PlatoEngineState;

fn main() -> Result<(), tauri::Error> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(PlatoEngineState::default())
        .invoke_handler(tauri::generate_handler![
            check_model_status,
            start_model_download,
            initialize_engine,
            stream_chat_completion,
            abort_inference,
            select_and_scan_workspace,
            start_workspace_ingestion
        ])
        .run(tauri::generate_context!())?;
    
    Ok(())
}